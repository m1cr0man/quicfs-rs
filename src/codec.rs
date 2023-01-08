use async_std::io;
use futures::{AsyncRead, AsyncWrite};
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::request_response::codec::{ProtocolName, RequestResponseCodec};

use crate::schema::quicfs::{QuicfsRequest, QuicfsResponse};
use crate::schema::rpc::RpcData;
use crate::schema_helpers::RpcCodec;
use prost::Message;

#[derive(Debug, Clone)]
pub struct QuicfsProtocol();

impl QuicfsProtocol {
    async fn decode<M, T>(io: &mut T) -> io::Result<M>
    where
        M: RpcCodec<M>,
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1_048_576).await?;

        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let rpc: io::Result<RpcData> =
            RpcData::decode(data.as_slice()).map_err(|_| io::ErrorKind::InvalidInput.into());
        M::from_rpc(rpc?).map_err(|_| io::ErrorKind::InvalidInput.into())
    }

    async fn encode<M, T>(msg: M, io: &mut T) -> io::Result<()>
    where
        M: RpcCodec<M>,
        T: AsyncWrite + Unpin + Send,
    {
        let data = msg.to_rpc().encode_to_vec();
        write_length_prefixed(io, data).await
    }
}

impl ProtocolName for QuicfsProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/quicfs/1.0.0".as_bytes()
    }
}

#[derive(Clone)]
pub struct QuicfsCodec();

#[async_trait::async_trait]
impl RequestResponseCodec for QuicfsCodec {
    type Protocol = QuicfsProtocol;

    type Request = QuicfsRequest;

    type Response = QuicfsResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        Self::Protocol::decode(io).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        Self::Protocol::decode(io).await
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        Self::Protocol::encode(req, io).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        Self::Protocol::encode(res, io).await
    }
}
