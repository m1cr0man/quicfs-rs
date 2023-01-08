use crate::schema::rpc::RpcData;

#[macro_export]
macro_rules! decode_rpc {
    ( $x:ident, $r:expr ) => {
        Ok(Self::$x($x::decode($r.body)?))
    };
}

#[macro_export]
macro_rules! encode_rpc {
    ( $m:expr, $x:expr ) => {
        RpcData {
            method: $m.into(),
            body: $x.encode_to_vec().into(),
        }
    };
}

pub trait RpcCodec<T> {
    fn from_rpc(rpc: RpcData) -> Result<T, prost::DecodeError>;
    fn to_rpc(&self) -> RpcData;
}
