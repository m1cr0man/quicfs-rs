use clap::Parser;
use futures_util::StreamExt;
use quinn::{Endpoint, NewConnection, ServerConfig};
use rustls_pemfile::Item;
use std::error::Error;
use std::net::SocketAddr;
use std::{fs::File, io::BufReader};
use tokio::{
    io::{stdin, AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast,
};
mod cli;

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

async fn server(listen_addr: &String) {
    let (certs, key) =
        read_certs_from_file("/var/lib/acme/unimog.m1cr0man.com".to_string()).unwrap();
    let server_config = ServerConfig::with_single_cert(certs, key).unwrap();
    let (_endpoint, mut listener) =
        Endpoint::server(server_config, convert_addr(listen_addr)).unwrap();

    let (tx, _rx) = broadcast::channel(10);

    // This loop allows us to accept multiple connections
    while let Some(conn) = listener.next().await {
        let mut connection: NewConnection = conn.await.unwrap();
        let addr = connection.connection.remote_address().clone();
        println!("{} connected", addr);

        let tx = tx.clone();
        // Quirk: Clone the rx from the tx, rather than the original rx
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            while let Some(Ok((mut write, read))) = connection.bi_streams.next().await {
                println!("{} new bidirectional stream", addr);

                // Note: BufReader owns the entire socket if we didn't split.
                // Hence we only pass the relevant half.
                let mut reader = tokio::io::BufReader::new(read);
                let mut line = String::new();

                loop {
                    // Use select so we can read + write at the same time
                    tokio::select! {
                        // Note: await is implicit
                        result = reader.read_line(&mut line) => {
                            if result.unwrap() == 0 {
                                break;
                            }

                            tx.send((line.clone(), addr)).unwrap();

                            // Wipe the line buffer each time
                            line.clear();
                        }

                        result = rx.recv() => {
                            let (msg, recv_addr) = result.unwrap();
                            // Don't repeat what the current connection sent
                            if recv_addr != addr {
                                // line.as_bytes -> provides underlying bytes
                                write.write_all(msg.as_bytes()).await.unwrap();
                            }
                        }
                    }
                }
            }
            println!("{} disconnected", addr);
        });
    }
}

async fn client(server_addr: &String) {
    let mut conn = TcpStream::connect(server_addr).await.unwrap();

    // No real need to split but we do it anyway in case things get more complex.
    let (read, mut write) = conn.split();

    let mut reader = tokio::io::BufReader::new(read);
    let mut line = String::new();

    // We can read stdin like any other object implementing AsyncRead
    let input = stdin();
    let mut input_reader = tokio::io::BufReader::new(input);
    let mut msg = String::new();

    loop {
        tokio::select! {
            _result = reader.read_line(&mut line) => {
                print!("{}", line);

                line.clear();
            }

            result = input_reader.read_line(&mut msg) => {
                // ctrl+d
                if result.unwrap() == 0 {
                    break;
                }

                write.write(msg.as_bytes()).await.unwrap();

                msg.clear();
            }
        }
    }

    // Clean shutdown
    println!("Bye!");
    conn.shutdown().await.unwrap();
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
