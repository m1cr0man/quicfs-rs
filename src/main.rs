use std::error::Error;

use libp2p::{identity, quic, PeerId};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let quic_config = quic::Config::new(&local_key);
    let mut transport = quic::async_std::Transport::new(quic_config);

    println!("Local peer ID: {:?}", local_peer_id);
    Ok(())
}
