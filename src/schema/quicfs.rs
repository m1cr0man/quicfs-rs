use prost::Message;
use crate::schema::rpc::RpcData;
use crate::{encode_rpc, schema_helpers::{RpcCodec, decode_rpc}};
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
            "Quicfs::FSStat" => QuicfsMethod::FSStat,
            "Quicfs::Mount" => QuicfsMethod::Mount,
            "Quicfs::Getattr" => QuicfsMethod::Getattr,
            "Quicfs::Readdir" => QuicfsMethod::Readdir,
            "Quicfs::Read" => QuicfsMethod::Read,
            &_ => QuicfsMethod::Undefined,
        }
    }
}
#[derive(Debug)]
pub enum QuicfsRequest {
    FSStat(GenericNodeRequest),
    Mount(MountRequest),
    Getattr(GenericNodeRequest),
    Readdir(ReaddirRequest),
    Read(ReadRequest),
}
#[derive(Debug)]
pub enum QuicfsResponse {
    FSStat(FsStatResponse),
    Mount(MountResponse),
    Getattr(GetattrResponse),
    Readdir(ReaddirResponse),
    Read(ReadResponse),
}
impl RpcCodec<QuicfsRequest> for QuicfsRequest {
    fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {
        let method: QuicfsMethod = rpc.method.clone().into();
        match method {
            QuicfsMethod::FSStat => decode_rpc(rpc).map(|v| Self::FSStat(v)),
            QuicfsMethod::Mount => decode_rpc(rpc).map(|v| Self::Mount(v)),
            QuicfsMethod::Getattr => decode_rpc(rpc).map(|v| Self::Getattr(v)),
            QuicfsMethod::Readdir => decode_rpc(rpc).map(|v| Self::Readdir(v)),
            QuicfsMethod::Read => decode_rpc(rpc).map(|v| Self::Read(v)),
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
            Self::FSStat(v) => encode_rpc!(QuicfsMethod::FSStat, v),
            Self::Mount(v) => encode_rpc!(QuicfsMethod::Mount, v),
            Self::Getattr(v) => encode_rpc!(QuicfsMethod::Getattr, v),
            Self::Readdir(v) => encode_rpc!(QuicfsMethod::Readdir, v),
            Self::Read(v) => encode_rpc!(QuicfsMethod::Read, v),
        }
    }
}
impl RpcCodec<QuicfsResponse> for QuicfsResponse {
    fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {
        let method: QuicfsMethod = rpc.method.clone().into();
        match method {
            QuicfsMethod::FSStat => decode_rpc(rpc).map(|v| Self::FSStat(v)),
            QuicfsMethod::Mount => decode_rpc(rpc).map(|v| Self::Mount(v)),
            QuicfsMethod::Getattr => decode_rpc(rpc).map(|v| Self::Getattr(v)),
            QuicfsMethod::Readdir => decode_rpc(rpc).map(|v| Self::Readdir(v)),
            QuicfsMethod::Read => decode_rpc(rpc).map(|v| Self::Read(v)),
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
            Self::FSStat(v) => encode_rpc!(QuicfsMethod::FSStat, v),
            Self::Mount(v) => encode_rpc!(QuicfsMethod::Mount, v),
            Self::Getattr(v) => encode_rpc!(QuicfsMethod::Getattr, v),
            Self::Readdir(v) => encode_rpc!(QuicfsMethod::Readdir, v),
            Self::Read(v) => encode_rpc!(QuicfsMethod::Read, v),
        }
    }
}
