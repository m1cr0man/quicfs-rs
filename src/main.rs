use codec::QuicfsCodec;
use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::stream::StreamExt;
use libp2p::request_response::ProtocolSupport;
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{
    core::muxing::StreamMuxerBox, identity, ping, quic, request_response, Multiaddr, PeerId,
    Transport,
};
use prost::bytes::Bytes;
use schema::quicfs::{QuicfsRequest, QuicfsResponse};
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::schema::quicfs::ReaddirRequest;
use crate::server::QuicfsServer;
mod codec;
mod schema;
mod schema_helpers;
mod server;
// mod sharing;

#[derive(NetworkBehaviour)]
#[behaviour(inject_event = true)]
struct QuicfsPeerBehaviour {
    ping: ping::Behaviour,
    request_response: request_response::RequestResponse<QuicfsCodec>,
}

struct SwarmHandler {
    swarm: Swarm<QuicfsPeerBehaviour>,
}

impl SwarmHandler {
    async fn run_client(
        mut self,
        request_tx: mpsc::Sender<QuicfsRequest>,
        mut request_rx: mpsc::Receiver<QuicfsRequest>,
        mut response_tx: mpsc::Sender<QuicfsResponse>,
    ) {
        let mut peers = Vec::new();
        let waiters: Arc<Mutex<Vec<oneshot::Sender<()>>>> = Arc::new(Mutex::new(Vec::new()));

        loop {
            futures::select! {
                event = self.swarm.select_next_some() => match event {
                    // Handle new listening addresses
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address)
                    }

                    // Handle new connections
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connection with {} established", peer_id);
                        peers.push(peer_id);

                        loop {
                            match waiters.lock().unwrap().pop() {
                                None => break,
                                Some(tx) => {
                                    if !tx.is_canceled() {
                                        if let Err(err) = tx.send(()) {
                                            println!("Error waking thread waiting for peer: {:?}", err)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Handle Request/Response events
                    SwarmEvent::Behaviour(QuicfsPeerBehaviourEvent::RequestResponse(
                        request_response::RequestResponseEvent::Message { peer, message },
                    )) => match message {
                        // Request
                        request_response::RequestResponseMessage::Request {
                            request_id,
                            request,
                            ..
                        } => {
                            println!(
                                "{:?} has sent RPC request: {} {:?}",
                                peer, request_id, request
                            );
                        }

                        // Response
                        request_response::RequestResponseMessage::Response {
                            request_id,
                            response,
                        } => {
                            println!(
                                "{:?} has sent RPC response: {} {:?}",
                                peer, request_id, response,
                            );
                            response_tx.send(response).await.unwrap();
                        }
                    },

                    // All other requests
                    evt => {
                        println!("{:?}", evt)
                    }
                },
                request = request_rx.select_next_some() => {
                    match peers.get(0) {
                        Some(peer) => {
                            self.swarm.behaviour_mut().request_response.send_request(peer, request);
                        },
                        None => {
                            let mut request_tx = request_tx.clone();
                            let waiters = waiters.clone();
                            async_std::task::spawn(async move {
                                let (tx, rx) = oneshot::channel();
                                waiters.lock().unwrap().push(tx);
                                println!("Waiting for peer");
                                rx.await.unwrap();
                                request_tx.send(request).await.unwrap();
                            });
                        }
                    }
                }
            }
        }
    }

    async fn run_server(mut self) {
        let (mut request_tx, request_rx) = mpsc::channel(1024);
        let (response_tx, mut response_rx) = mpsc::channel(1024);

        async_std::task::spawn(QuicfsServer::new(request_rx, response_tx).run(Some(64)));

        loop {
            futures::select! {
                event = self.swarm.select_next_some() => match event {
                    // Handle new listening addresses
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address)
                    }

                    // Handle new connections
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connection with {} established", peer_id)
                    }

                    // Handle Request/Response events
                    SwarmEvent::Behaviour(QuicfsPeerBehaviourEvent::RequestResponse(
                        request_response::RequestResponseEvent::Message { peer, message },
                    )) => match message {
                        // Request
                        request_response::RequestResponseMessage::Request {
                            request_id,
                            request,
                            channel,
                        } => {
                            println!(
                                "{:?} has sent RPC request: {} {:?}",
                                peer, request_id, request
                            );

                            if let Err(err) = request_tx.send((channel, request)).await {
                                println!("Failed to send query to server: {:?}", err);
                            }
                        }

                        // Response
                        request_response::RequestResponseMessage::Response {
                            request_id,
                            response,
                        } => {
                            println!(
                                "{:?} has sent RPC response: {} {:?}",
                                peer, request_id, response,
                            );
                        }
                    },

                    // All other requests
                    evt => {
                        println!("{:?}", evt)
                    }
                },

                (channel, response) = response_rx.select_next_some() => match response {
                    Ok(resp) => {
                        if let Err(err) = self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, resp)
                        {
                            println!("Failed to send response: {:?}", err);
                        };
                    }
                    Err(err) => {
                        println!("Error generating response: {:?}", err)
                    }
                },
            }
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer ID: {:?}", local_peer_id);

    let quic_config = quic::Config::new(&local_key);
    // Has to be put in muxer manually. See https://github.com/libp2p/rust-libp2p/issues/3179#issuecomment-1331718484
    // or https://github.com/libp2p/rust-libp2p/blob/be0b62a78fe9d72811b9eda742137cc8ddc4da35/transports/quic/tests/smoke.rs#L310-L319
    let transport = quic::async_std::Transport::new(quic_config)
        .map(|(p, c), _| (p, StreamMuxerBox::new(c)))
        .boxed();

    let behaviour = ping::Behaviour::new(ping::Config::new());

    let qcodec = QuicfsCodec {};

    let behaviour = QuicfsPeerBehaviour {
        request_response: request_response::RequestResponse::new(
            qcodec,
            [(codec::QuicfsProtocol {}, ProtocolSupport::Full)],
            request_response::RequestResponseConfig::default(),
        ),
        ping: behaviour,
    };

    let mut swarm = Swarm::with_async_std_executor(transport, behaviour, local_peer_id);

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {}", addr);
        let handler = SwarmHandler { swarm };

        let (mut request_tx, request_rx) = mpsc::channel(64);
        let (response_tx, mut response_rx) = mpsc::channel(64);

        async_std::task::spawn(handler.run_client(request_tx.clone(), request_rx, response_tx));

        request_tx
            .send(QuicfsRequest::Readdir(ReaddirRequest {
                handle_id: Bytes::from(b"1".to_vec()),
            }))
            .await
            .expect("Failed to send request");
        let resp = response_rx.next().await.expect("Failed to get response");

        println!("Got response!!1!1! {:?}", resp);
    } else {
        let handler = SwarmHandler { swarm };
        handler.run_server().await;
    }

    // The swarm is not sendable - we need to manage it in one place.
    // If we tried to fork on each event we'd have a bad time, so instead keep all
    // code to manage the swarm together in one task.
    Ok(())
}
