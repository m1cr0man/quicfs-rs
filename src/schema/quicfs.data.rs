#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileAttributes {
    #[prost(bytes = "vec", tag = "1")]
    pub handle_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub parent_handle_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration = "FileType", tag = "4")]
    pub file_type: i32,
    #[prost(uint32, tag = "5")]
    pub mode: u32,
    #[prost(uint32, tag = "6")]
    pub nlink: u32,
    #[prost(uint32, tag = "7")]
    pub uid: u32,
    #[prost(uint32, tag = "8")]
    pub gid: u32,
    #[prost(uint64, tag = "9")]
    pub size: u64,
    #[prost(uint64, tag = "10")]
    pub used: u64,
    #[prost(uint64, tag = "11")]
    pub offset: u64,
    #[prost(message, optional, tag = "12")]
    pub mtime: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "13")]
    pub ctime: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "14")]
    pub atime: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsStat {
    #[prost(uint64, tag = "1")]
    pub tbytes: u64,
    #[prost(uint64, tag = "2")]
    pub fbytes: u64,
    #[prost(uint64, tag = "3")]
    pub abytes: u64,
    #[prost(uint64, tag = "4")]
    pub tfiles: u64,
    #[prost(uint64, tag = "5")]
    pub ffiles: u64,
    #[prost(uint64, tag = "6")]
    pub afiles: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FileType {
    Unspecified = 0,
    Regular = 1,
    Directory = 2,
    Block = 3,
    Character = 4,
    Link = 5,
    Socket = 6,
    Fifo = 7,
}
impl FileType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FileType::Unspecified => "FILE_TYPE_UNSPECIFIED",
            FileType::Regular => "FILE_TYPE_REGULAR",
            FileType::Directory => "FILE_TYPE_DIRECTORY",
            FileType::Block => "FILE_TYPE_BLOCK",
            FileType::Character => "FILE_TYPE_CHARACTER",
            FileType::Link => "FILE_TYPE_LINK",
            FileType::Socket => "FILE_TYPE_SOCKET",
            FileType::Fifo => "FILE_TYPE_FIFO",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FILE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "FILE_TYPE_REGULAR" => Some(Self::Regular),
            "FILE_TYPE_DIRECTORY" => Some(Self::Directory),
            "FILE_TYPE_BLOCK" => Some(Self::Block),
            "FILE_TYPE_CHARACTER" => Some(Self::Character),
            "FILE_TYPE_LINK" => Some(Self::Link),
            "FILE_TYPE_SOCKET" => Some(Self::Socket),
            "FILE_TYPE_FIFO" => Some(Self::Fifo),
            _ => None,
        }
    }
}
