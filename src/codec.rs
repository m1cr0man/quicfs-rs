use async_std::io;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::request_response::codec::{ProtocolName, RequestResponseCodec};

use crate::schema::rpc::RpcData;
use prost::Message;

#[derive(Debug, Clone)]
pub struct QuicfsProtocol();

impl ProtocolName for QuicfsProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/quicfs/1.0.0".as_bytes()
    }
}

#[derive(Clone)]
pub struct QuicfsCodec();

#[derive(Debug)]
pub struct Test(Vec<u8>);

#[async_trait::async_trait]
impl RequestResponseCodec for QuicfsCodec {
    type Protocol = QuicfsProtocol;

    // TODO
    type Request = RpcData;

    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1_048_576).await?;

        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Self::Request::decode(data.as_slice()).map_err(|_| io::ErrorKind::InvalidInput.into())
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut out = Vec::new();
        let _ = io.read_to_end(&mut out).await;
        Ok(out)
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
        let data = req.encode_to_vec();
        write_length_prefixed(io, data).await
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
        io.write_all(&res).await
    }
}
