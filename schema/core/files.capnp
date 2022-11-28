@0xae7b7cd10f869e4d;

using Rust = import "../rust.capnp";
$Rust.parentModule("schema::core");

enum CreateMode {
    unchecked @0;
    guarded @1;
    exclusive @2;
}

enum FileType {
    regular @0;
    directory @1;
    block @2;
    character @3;
    link @4;
    socket @5;
    fifo @6;
}

struct AccessRights {
    const read :UInt16 = 1;
    const lookup :UInt16 = 2;
    const modify :UInt16 = 4;
    const extend :UInt16 = 8;
    const delete :UInt16 = 16;
    const execute :UInt16 = 32;
}

struct FileAttributes {
    using import "models.capnp".Timestamp;
    name @0   :Text;
    type @1   :FileType;
    mode @2   :UInt32;
    nlink @3  :UInt32;
    uid @4    :UInt32;
    gid @5    :UInt32;
    size @6   :UInt64;
    used @7   :UInt64;
    offset @8 :UInt64;
    mtime @9  :Timestamp;
    ctime @10 :Timestamp;
    atime @11 :Timestamp;
}

struct FileMode {
    const otherExec  :UInt32 = 1;
    const otherWrite :UInt32 = 2;
    const otherRead  :UInt32 = 4;
    const groupExec  :UInt32 = 8;
    const groupWrite :UInt32 = 16;
    const groupRead  :UInt32 = 32;
    const ownerExec  :UInt32 = 64;
    const ownerWrite :UInt32 = 128;
    const ownerRead  :UInt32 = 256;
    const sticky     :UInt32 = 512;
    const setGid     :UInt32 = 1024;
    const setUid     :UInt32 = 2048;
}

struct FSStat {
    tbytes   @0 :UInt64;
    fbytes   @1 :UInt64;
    abytes   @2 :UInt64;
    tfiles   @3 :UInt64;
    ffiles   @4 :UInt64;
    afiles   @5 :UInt64;
    invarsec @6 :UInt32;
}

using FileHandle = UInt64;
