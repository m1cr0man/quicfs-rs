#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericNodeRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub handle_id: ::prost::alloc::vec::Vec<u8>,
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
        HandleId(::prost::alloc::vec::Vec<u8>),
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
    #[prost(bytes = "vec", tag = "1")]
    pub handle_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub response_id: ::prost::alloc::vec::Vec<u8>,
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
    #[prost(bytes = "vec", tag = "1")]
    pub handle_id: ::prost::alloc::vec::Vec<u8>,
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
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub offset: u64,
    #[prost(uint64, tag = "4")]
    pub size: u64,
    #[prost(bool, tag = "5")]
    pub eof: bool,
}
