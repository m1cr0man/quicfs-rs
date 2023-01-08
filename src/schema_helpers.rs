use crate::schema::rpc::RpcData;

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

#[inline]
pub fn decode_rpc<T: prost::Message + Default>(rpc: RpcData) -> Result<T, prost::DecodeError> {
    T::decode(rpc.body)
}
