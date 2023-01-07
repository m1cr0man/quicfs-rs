use futures::prelude::*;
use libp2p::request_response::ProtocolSupport;
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{
    core::muxing::StreamMuxerBox, identity, ping, quic, request_response, Multiaddr, PeerId,
    Transport,
};
use std::error::Error;

use codec::QuicfsCodec;
use schema::rpc::RpcData;
mod codec;
mod schema;
// mod sharing;

#[derive(NetworkBehaviour)]
#[behaviour(inject_event = true)]
struct QuicfsPeer {
    ping: ping::Behaviour,
    // kademlia: Kademlia<MemoryStore>,
    request_response: request_response::RequestResponse<QuicfsCodec>,
}

// This is done automatically if behaviour(out_event) is not set
// #[derive(Debug)]
// enum QuicfsPeerEvent {
//     Kademlia(KademliaEvent),
//     Ping(ping::Event),
// }

// impl From<KademliaEvent> for QuicfsPeerEvent {
//     fn from(event: KademliaEvent) -> Self {
//         Self::Kademlia(event)
//     }
// }

// impl From<ping::Event> for QuicfsPeerEvent {
//     fn from(event: ping::Event) -> Self {
//         Self::Ping(event)
//     }
// }

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

    let behaviour = QuicfsPeer {
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
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connection with {} established", peer_id);
                swarm.behaviour_mut().request_response.send_request(
                    &peer_id,
                    RpcData {
                        method: "testmethod".into(),
                        body: "oh hi mark".into(),
                    },
                );
            }
            SwarmEvent::Behaviour(QuicfsPeerEvent::RequestResponse(
                request_response::RequestResponseEvent::Message { peer, message },
            )) => match message {
                request_response::RequestResponseMessage::Request {
                    request_id,
                    request,
                    channel,
                } => {
                    println!(
                        "{:?} has sent RPC request: {} {}",
                        peer, request_id, request.method
                    );
                    // Unfortunately request_response uses a oneshot queue
                    // internally so I can't use it to queue up multiple responses to the same
                    // request
                    swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, "oh hi mark".into())
                        .expect("Failed to respond")
                }
                request_response::RequestResponseMessage::Response {
                    request_id,
                    response,
                } => {
                    println!(
                        "{:?} has sent RPC response: {} {:?}",
                        peer,
                        request_id,
                        std::str::from_utf8(&response).expect("Invalid UTF8 response"),
                    );
                }
            },
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            _ => {}
        }
    }
}
