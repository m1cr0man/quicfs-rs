@0xae7b7cd10f869e4d;

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
    const Read :UInt16 = 1;
    const Lookup :UInt16 = 2;
    const Modify :UInt16 = 4;
    const Extend :UInt16 = 8;
    const Delete :UInt16 = 16;
    const Execute :UInt16 = 32;
}

struct FileAttributes {
    using import "models.capnp".Timestamp;
    type @0   :FileType;
    mode @1   :UInt32;
    nlink @2  :UInt32;
    uid @3    :UInt32;
    gid @4    :UInt32;
    size @5   :UInt64;
    used @6   :UInt64;
    offset @7 :UInt64;
    mtime @8  :Timestamp;
    ctime @9  :Timestamp;
    atime @10 :Timestamp;
}

struct FileMode {
    const OtherExec  :UInt32 = 1;
    const OtherWrite :UInt32 = 2;
    const OtherRead  :UInt32 = 4;
    const GroupExec  :UInt32 = 8;
    const GroupWrite :UInt32 = 16;
    const GroupRead  :UInt32 = 32;
    const OwnerExec  :UInt32 = 64;
    const OwnerWrite :UInt32 = 128;
    const OwnerRead  :UInt32 = 256;
    const Sticky     :UInt32 = 512;
    const SetGid     :UInt32 = 1024;
    const SetUid     :UInt32 = 2048;
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
