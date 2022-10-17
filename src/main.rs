use crate::schema::server_capnp;
use clap::Parser;
use futures_util::{FutureExt, StreamExt};
use quinn::{ClientConfig, Endpoint, NewConnection, ServerConfig, TransportConfig};
use rustls::RootCertStore;
use rustls_pemfile::Item;
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::BufReader};

mod cli;

mod schema;
mod server;

const RPC_THREADS: usize = 4;

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

async fn handle_stream((write, read): (quinn::SendStream, quinn::RecvStream)) {
    let network = capnp_rpc::twoparty::VatNetwork::new(
        read,
        write,
        capnp_rpc::rpc_twoparty_capnp::Side::Server,
        Default::default(),
    );

    // Create a capnp client
    let quicfs_client: server_capnp::server::Client =
        capnp_rpc::new_client(server::QuicfsServer {});

    let rpc_system = capnp_rpc::RpcSystem::new(Box::new(network), Some(quicfs_client.client));

    if let Err(error) = rpc_system.await {
        println!("Error encountered in RPC system: {}", error);
    };

    println!("Stream closing");
}

async fn server(listen_addr: &String) {
    let (certs, key) =
        read_certs_from_file("/var/lib/acme/unimog.m1cr0man.com".to_string()).unwrap();

    let transport_config = get_transport_config();

    let mut server_config = ServerConfig::with_single_cert(certs, key).unwrap();
    server_config.transport = Arc::new(transport_config);

    let (_endpoint, mut listener) =
        Endpoint::server(server_config, convert_addr(listen_addr)).unwrap();

    // This loop allows us to accept multiple connections
    while let Some(conn) = listener.next().await {
        let mut connection: NewConnection = conn.await.unwrap();
        let addr = connection.connection.remote_address().clone();
        println!("{} connected", addr);

        tokio::spawn(async move {
            println!("{} waiting for bidirectional streams", addr);

            loop {
                tokio::select! {
                    result = connection.bi_streams.next() => {
                        if let Some(result_val) = result {
                            match result_val {

                                Ok(streams) => {
                                    println!("{} new bidirectional stream, starting RPC", addr);
                                    tokio::spawn(async move {
                                        let local_pool = tokio_util::task::LocalPoolHandle::new(1);
                                        local_pool.spawn_pinned(|| { handle_stream(streams) }).await.unwrap();
                                        println!("Bidirectional stream handler finished");
                                    });
                                },

                                Err(error) => {
                                    // FYI: Removing the else allows you to auto-fill all the types
                                    match error {
                                        quinn::ConnectionError::TimedOut => {
                                        },
                                        quinn::ConnectionError::ApplicationClosed(_) => {
                                            println!("{} disconnected", addr);
                                            break;
                                        },
                                        _ => {
                                            println!("{} unhandled stream error {}", addr, error);
                                            break;
                                        }
                                    }
                                },
                            }
                        } else {
                            println!("huh?");
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

    let NewConnection { connection, .. } = conn.await.unwrap();

    println!("Connected to {}", server_addr);

    let (write, read) = connection.open_bi().await.unwrap();
    let mut writer = Box::new(write);

    {
        let network = capnp_rpc::twoparty::VatNetwork::new(
            read,
            writer,
            capnp_rpc::rpc_twoparty_capnp::Side::Client,
            Default::default(),
        );

        let mut rpc_system = capnp_rpc::RpcSystem::new(Box::new(network), None);

        let disconnector = rpc_system.get_disconnector();

        let quicfs_client: server_capnp::server::Client =
            rpc_system.bootstrap(capnp_rpc::rpc_twoparty_capnp::Side::Server);

        let local = tokio::task::LocalSet::new();

        local
            .run_until(async move {
                tokio::task::spawn_local(Box::pin(rpc_system));

                let mut request = quicfs_client.null_request();
                request.get();

                let reply = request.send().promise.await.unwrap();

                reply.get().unwrap();
                println!("Got a response from null_request!");
            })
            .await;

        // Clean shutdown
        println!("Shutting down");
        disconnector.await.unwrap();
    }

    println!("Bye!");
    connection.close(quinn::VarInt::from_u32(0), "bye".as_bytes());
}

#[tokio::main]
async fn main() {
    // let out = "Hello world!";
    // println!("{}", out);

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
