use crate::cli::{Commands, QuicFSCli};
use bytes::Bytes;
use log::LevelFilter;
use qp2p::{Config, Endpoint};
use std::{
    iter::FromIterator,
    net::{Ipv4Addr, SocketAddr},
    process::exit,
    str::FromStr,
    time::Duration,
};

#[macro_use]
extern crate log;

use clap::Parser;

mod cli;
mod error;

// TODO qp2p peer and node
#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

#[tokio::main]
async fn main() -> Result<(), ()> {
    let out = "Hello world!";
    println!("{}", out);

    env_logger::builder()
        .filter(None, LevelFilter::Debug)
        .init();

    // Create an endpoint
    let (node, mut incoming_conns, _contact) = Endpoint::new_peer(
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        &[],
        Config {
            idle_timeout: Duration::from_secs(60 * 60).into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let cli = QuicFSCli::parse();

    match &cli.command {
        Commands::Client { server, src, dest } => {
            let addr = match server {
                Some(a) => match SocketAddr::from_str(a) {
                    Ok(b) => b,
                    Err(err) => {
                        println!("Failed to parse address: {:?}", err);
                        exit(2);
                    }
                },
                None => {
                    println!("Missing server address");
                    exit(1);
                }
            };

            let src = src.as_deref().unwrap();
            let dest = dest.as_deref().unwrap();
            println!(
                "Downloading {} -> {} from {:?}",
                src.clone(),
                dest.clone(),
                addr,
            );

            let (conn, mut incoming) = node.connect_to(&addr).await.unwrap();

            conn.send((Bytes::new(), Bytes::new(), Bytes::from_iter(src.bytes())))
                .await
                .unwrap();

            let reply = incoming.next().await.unwrap();
            println!("Received from {:?} -> {:?}", addr, reply);
        }

        Commands::Server { listen, serve } => {
            println!(
                "Serving {} on {:?}",
                serve.as_deref().unwrap(),
                node.public_addr(),
            );

            while let Some((connection, mut incoming_messages)) = incoming_conns.next().await {
                let src = connection.remote_address();

                while let Some((_, _, bytes)) = incoming_messages.next().await.unwrap() {
                    println!("Received from {:?} -> {:?}", src, bytes);

                    // Echo back the bytes
                    let reply = [Bytes::from("Here is your "), bytes].concat();

                    connection
                        .send((Bytes::new(), Bytes::new(), Bytes::from(reply.clone())))
                        .await
                        .unwrap();
                    println!("Sent to {:?} -> {:?}", src, reply);
                }
            }
        }
    }
    Ok(())
}
