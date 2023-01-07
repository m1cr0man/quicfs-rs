use libp2p::request_response::{Codec, ProtocolName};

#[derive(Debug, Clone)]
struct QuicfsProtocol();

impl ProtocolName for QuicfsProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/quicfs/1.0.0".as_bytes()
    }
}

#[derive(Clone)]
struct QuicfsCodec();

impl Codec for QuicfsCodec {
    type Protocol = QuicfsProtocol
}
