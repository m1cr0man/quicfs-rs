use prost::Message;
use crate::schema::rpc::RpcData;
use crate::{encode_rpc, decode_rpc, schema_helpers::RpcCodec};
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericNodeRequest {
    #[prost(bytes = "bytes", tag = "1")]
    pub handle_id: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AsyncResponse {
    #[prost(oneof = "async_response::AsyncData", tags = "1, 2")]
    pub async_data: ::core::option::Option<async_response::AsyncData>,
}
/// Nested message and enum types in `AsyncResponse`.
pub mod async_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum AsyncData {
        #[prost(string, tag = "1")]
        Error(::prost::alloc::string::String),
        /// Will be used in the future for distributed responses
        #[prost(uint32, tag = "2")]
        Servers(u32),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MountRequest {
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MountResponse {
    #[prost(oneof = "mount_response::MountData", tags = "1, 2")]
    pub mount_data: ::core::option::Option<mount_response::MountData>,
}
/// Nested message and enum types in `MountResponse`.
pub mod mount_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MountData {
        #[prost(string, tag = "1")]
        Error(::prost::alloc::string::String),
        #[prost(bytes, tag = "2")]
        HandleId(::prost::bytes::Bytes),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsStatResponse {
    #[prost(oneof = "fs_stat_response::FsstatData", tags = "1, 2")]
    pub fsstat_data: ::core::option::Option<fs_stat_response::FsstatData>,
}
/// Nested message and enum types in `FSStatResponse`.
pub mod fs_stat_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FsstatData {
        #[prost(string, tag = "1")]
        Error(::prost::alloc::string::String),
        #[prost(message, tag = "2")]
        Fsstat(super::data::FsStat),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetattrResponse {
    #[prost(oneof = "getattr_response::GetattrData", tags = "1, 2")]
    pub getattr_data: ::core::option::Option<getattr_response::GetattrData>,
}
/// Nested message and enum types in `GetattrResponse`.
pub mod getattr_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum GetattrData {
        #[prost(string, tag = "1")]
        Error(::prost::alloc::string::String),
        #[prost(message, tag = "2")]
        Attributes(super::data::FileAttributes),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReaddirRequest {
    #[prost(bytes = "bytes", tag = "1")]
    pub handle_id: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReaddirResponse {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<data::FileAttributes>,
    #[prost(uint64, tag = "3")]
    pub offset: u64,
    #[prost(uint64, tag = "4")]
    pub size: u64,
    #[prost(bool, tag = "5")]
    pub eof: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadRequest {
    #[prost(bytes = "bytes", tag = "1")]
    pub handle_id: ::prost::bytes::Bytes,
    #[prost(uint64, tag = "2")]
    pub offset: u64,
    #[prost(uint64, tag = "3")]
    pub size: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadResponse {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: ::prost::bytes::Bytes,
    #[prost(uint64, tag = "3")]
    pub offset: u64,
    #[prost(uint64, tag = "4")]
    pub size: u64,
    #[prost(bool, tag = "5")]
    pub eof: bool,
}
#[derive(Debug)]
pub enum QuicfsMethod {
    Undefined,
    FSStat,
    Mount,
    Getattr,
    Readdir,
    Read,
}
impl From<QuicfsMethod> for String {
    fn from(method: QuicfsMethod) -> Self {
        (match method {
            QuicfsMethod::Undefined => "Undefined",
            QuicfsMethod::FSStat => "Quicfs::FSStat",
            QuicfsMethod::Mount => "Quicfs::Mount",
            QuicfsMethod::Getattr => "Quicfs::Getattr",
            QuicfsMethod::Readdir => "Quicfs::Readdir",
            QuicfsMethod::Read => "Quicfs::Read",
        })
            .to_string()
    }
}
impl From<String> for QuicfsMethod {
    fn from(method_str: String) -> Self {
        match method_str.as_str() {
            &_ => QuicfsMethod::Undefined,
            "Quicfs::FSStat" => QuicfsMethod::FSStat,
            "Quicfs::Mount" => QuicfsMethod::Mount,
            "Quicfs::Getattr" => QuicfsMethod::Getattr,
            "Quicfs::Readdir" => QuicfsMethod::Readdir,
            "Quicfs::Read" => QuicfsMethod::Read,
        }
    }
}
#[derive(Debug)]
pub enum QuicfsRequest {
    GenericNodeRequest(GenericNodeRequest),
    ReaddirRequest(ReaddirRequest),
    MountRequest(MountRequest),
    ReadRequest(ReadRequest),
}
#[derive(Debug)]
pub enum QuicfsResponse {
    GetattrResponse(GetattrResponse),
    FsStatResponse(FsStatResponse),
    ReaddirResponse(ReaddirResponse),
    ReadResponse(ReadResponse),
    MountResponse(MountResponse),
}
impl RpcCodec<QuicfsRequest> for QuicfsRequest {
    fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {
        let method: QuicfsMethod = rpc.method.clone().into();
        match method {
            QuicfsMethod::FSStat => decode_rpc!(GenericNodeRequest, rpc),
            QuicfsMethod::Mount => decode_rpc!(MountRequest, rpc),
            QuicfsMethod::Getattr => decode_rpc!(GenericNodeRequest, rpc),
            QuicfsMethod::Readdir => decode_rpc!(ReaddirRequest, rpc),
            QuicfsMethod::Read => decode_rpc!(ReadRequest, rpc),
            QuicfsMethod::Undefined => {
                Err(
                    prost::DecodeError::new(
                        format!("Unrecognised RPC method {}", rpc.method),
                    ),
                )
            }
        }
    }
    fn to_rpc(&self) -> RpcData {
        match self {
            Self::GenericNodeRequest(v) => encode_rpc!(QuicfsMethod::FSStat, v),
            Self::MountRequest(v) => encode_rpc!(QuicfsMethod::Mount, v),
            Self::GenericNodeRequest(v) => encode_rpc!(QuicfsMethod::Getattr, v),
            Self::ReaddirRequest(v) => encode_rpc!(QuicfsMethod::Readdir, v),
            Self::ReadRequest(v) => encode_rpc!(QuicfsMethod::Read, v),
        }
    }
}
impl RpcCodec<QuicfsResponse> for QuicfsResponse {
    fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {
        let method: QuicfsMethod = rpc.method.clone().into();
        match method {
            QuicfsMethod::FSStat => decode_rpc!(FsStatResponse, rpc),
            QuicfsMethod::Mount => decode_rpc!(MountResponse, rpc),
            QuicfsMethod::Getattr => decode_rpc!(GetattrResponse, rpc),
            QuicfsMethod::Readdir => decode_rpc!(ReaddirResponse, rpc),
            QuicfsMethod::Read => decode_rpc!(ReadResponse, rpc),
            QuicfsMethod::Undefined => {
                Err(
                    prost::DecodeError::new(
                        format!("Unrecognised RPC method {}", rpc.method),
                    ),
                )
            }
        }
    }
    fn to_rpc(&self) -> RpcData {
        match self {
            Self::FsStatResponse(v) => encode_rpc!(QuicfsMethod::FSStat, v),
            Self::MountResponse(v) => encode_rpc!(QuicfsMethod::Mount, v),
            Self::GetattrResponse(v) => encode_rpc!(QuicfsMethod::Getattr, v),
            Self::ReaddirResponse(v) => encode_rpc!(QuicfsMethod::Readdir, v),
            Self::ReadResponse(v) => encode_rpc!(QuicfsMethod::Read, v),
        }
    }
}
