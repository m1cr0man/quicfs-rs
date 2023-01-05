use futures::prelude::*;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{core::muxing::StreamMuxerBox, identity, ping, quic, Multiaddr, PeerId, Transport};
use std::error::Error;

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

    let behaviour = ping::Behaviour::new(ping::Config::new().with_keep_alive(true));

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
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            _ => {}
        }
    }
}
