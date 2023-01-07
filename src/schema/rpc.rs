#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcData {
    #[prost(string, tag = "1")]
    pub method: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub body: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "RpcDirection", tag = "3")]
    pub direction: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RpcDirection {
    Unspecified = 0,
    Request = 1,
    Response = 2,
}
impl RpcDirection {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            RpcDirection::Unspecified => "RPC_DIRECTION_UNSPECIFIED",
            RpcDirection::Request => "RPC_DIRECTION_REQUEST",
            RpcDirection::Response => "RPC_DIRECTION_RESPONSE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RPC_DIRECTION_UNSPECIFIED" => Some(Self::Unspecified),
            "RPC_DIRECTION_REQUEST" => Some(Self::Request),
            "RPC_DIRECTION_RESPONSE" => Some(Self::Response),
            _ => None,
        }
    }
}
