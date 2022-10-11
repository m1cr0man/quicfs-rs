@0xeb2cf27e974ce07f;
using Errors = import "./errors.capnp";

struct Timestamp {
    seconds @0 :UInt64;
    nseconds @1 :UInt64;
}

struct Result(T) {
    union {
        value @0 :T;
        error @1 :Errors.Error;
    }
}

struct BooleanResult {
    union {
        value @0 :Bool;
        error @1 :Errors.Error;
    }
}
