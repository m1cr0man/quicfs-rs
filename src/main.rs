use clap::Parser;
use futures_util::StreamExt;
use quinn::{ClientConfig, Endpoint, NewConnection, ServerConfig, TransportConfig};
use rustls::RootCertStore;
use rustls_pemfile::Item;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use tokio::{
    io::{stdin, AsyncBufReadExt},
    sync::broadcast,
};
mod cli;

#[path = "./schema/server_capnp.rs"]
mod server_capnp;

pub fn read_certs_from_file(
    dir: String,
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(dir.clone() + "/fullchain.pem")?);
    let certs = rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open(dir + "/key.pem")?);
    let key = match rustls_pemfile::read_one(&mut key_reader)? {
        Some(Item::ECKey(k)) => k,
        _ => panic!("Unrecognised key"),
    };

    let key = rustls::PrivateKey(key);

    return Ok((certs, key));
}

fn convert_addr(addr: &String) -> SocketAddr {
    return addr.parse().unwrap();
}

fn get_transport_config() -> TransportConfig {
    let mut transport_config = TransportConfig::default();
    transport_config
        .keep_alive_interval(Some(Duration::from_secs(5)))
        .max_idle_timeout(Some(Duration::from_secs(35).try_into().unwrap()));
    transport_config
}

async fn server(listen_addr: &String) {
    let (certs, key) =
        read_certs_from_file("/var/lib/acme/unimog.m1cr0man.com".to_string()).unwrap();

    let transport_config = get_transport_config();

    let mut server_config = ServerConfig::with_single_cert(certs, key).unwrap();
    server_config.transport = Arc::new(transport_config);

    let (_endpoint, mut listener) =
        Endpoint::server(server_config, convert_addr(listen_addr)).unwrap();

    let (tx, _rx) = broadcast::channel(10);

    // This loop allows us to accept multiple connections
    while let Some(conn) = listener.next().await {
        let mut connection: NewConnection = conn.await.unwrap();
        let addr = connection.connection.remote_address().clone();
        println!("{} connected", addr);

        let tx = tx.clone();

        let (shut_tx, mut shut_rx) = broadcast::channel(1);

        tokio::spawn(async move {
            println!("{} opening unidirectional stream", addr);

            let mut write = connection.connection.open_uni().await.unwrap();

            let tx = tx.clone();
            // Quirk: Clone the rx from the tx, rather than the original rx
            let mut rx = tx.subscribe();

            loop {
                tokio::select! {
                    result = connection.uni_streams.next() => {
                        if let Some(result_val) = result {
                            match result_val {

                                Ok(read) => {
                                    println!("{} new unidirectional stream", addr);

                                    let tx = tx.clone();
                                    let shut_tx = shut_tx.clone();

                                    tokio::spawn(async move {
                                        let mut reader = tokio::io::BufReader::new(read);
                                        let mut line = String::new();

                                        while let Ok(read_size) = reader.read_line(&mut line).await {
                                            if read_size == 0 {
                                                break;
                                            }

                                            print!("{} {}", addr, line);
                                            tx.send((line.clone(), addr)).unwrap();

                                            // Wipe the line buffer each time
                                            line.clear();
                                        }

                                        println!("{} unidirectional stream closed by peer", addr);
                                        shut_tx.send(1).unwrap();
                                    });
                                },

                                Err(error) => {
                                    // FYI: Removing the else allows you to auto-fill all the types
                                    match error {
                                        quinn::ConnectionError::TimedOut => {
                                        },
                                        _ => {
                                            println!("{} unhandled stream error {}", addr, error);
                                        }
                                    }
                                },
                            }
                        } else {
                            break;
                        }
                    }

                    result = rx.recv() => {
                        let (msg, recv_addr) = result.unwrap();
                        // Don't repeat what the current connection sent
                        if recv_addr != addr {
                            // line.as_bytes -> provides underlying bytes
                            write.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }

                    result = shut_rx.recv() => {
                        if let Ok(1) = result {
                            println!("{} Disconnecting", addr);
                            // Everything implements the appropriate out-of-scope handlers.
                            break;
                        }
                    }
                }
            }

            println!("{} disconnected", addr);
        });
    }
}

async fn client(server_addr: &String) {
    let (certs, _key) =
        read_certs_from_file("/var/lib/acme/unimog.m1cr0man.com".to_string()).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(certs.last().unwrap()).unwrap();

    let transport_config = get_transport_config();

    let mut client_config = ClientConfig::with_root_certificates(roots);
    client_config.transport = Arc::new(transport_config);

    let mut endpoint = Endpoint::client(convert_addr(&"0.0.0.0:0".to_string())).unwrap();
    endpoint.set_default_client_config(client_config);

    let conn = endpoint
        .connect(convert_addr(server_addr), "unimog.m1cr0man.com")
        .unwrap();

    let NewConnection {
        connection,
        mut uni_streams,
        ..
    } = conn.await.unwrap();

    println!("Connected to {}", server_addr);

    let mut write = connection.open_uni().await.unwrap();

    // We can read stdin like any other object implementing AsyncRead
    let input = stdin();
    let mut input_reader = tokio::io::BufReader::new(input);
    let mut msg = String::new();
    let mut written;
    let addr = connection.remote_address().clone();

    loop {
        tokio::select! {
            result = uni_streams.next() => {
                if let Some(result_val) = result {
                    match result_val {

                        Ok(read) => {
                            println!("{} new unidirectional stream", addr);

                            tokio::spawn(async move {
                                let mut reader = tokio::io::BufReader::new(read);
                                let mut line = String::new();

                                while let Ok(read_size) = reader.read_line(&mut line).await {
                                    if read_size == 0 {
                                        break;
                                    }

                                    print!("{}", line);

                                    line.clear();
                                }

                                println!("{} unidirectional stream closed by peer", addr);
                            });
                        },

                        Err(error) => {
                            // FYI: Removing the else allows you to auto-fill all the types
                            match error {
                                quinn::ConnectionError::TimedOut => {
                                },
                                _ => {
                                    println!("{} unhandled stream error {}", addr, error);
                                }
                            }
                        },
                    }
                } else {
                    break;
                }
            }

            result = input_reader.read_line(&mut msg) => {
                // ctrl+d
                let datalen = result.unwrap();
                if datalen == 0 {
                    println!("{} Disconnecting", addr);
                    write.finish().await.unwrap();
                    break;
                }

                written = 0;
                while written < datalen {
                    written = written + write.write(msg[written..].as_bytes()).await.unwrap();
                    println!("{} written", written);
                }

                msg.clear();
            }
        }
    }

    // Clean shutdown
    println!("Bye!");
}

#[tokio::main]
async fn main() {
    let out = "Hello world!";
    println!("{}", out);

    let cli = cli::QuicFSCli::parse();

    match &cli.command {
        cli::Commands::Server { listen, serve: _ } => {
            return server(listen).await;
        }
        cli::Commands::Client {
            server: server_addr,
            src: _,
            dest: _,
        } => {
            return client(server_addr).await;
        }
    }
}
