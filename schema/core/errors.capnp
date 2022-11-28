@0xc4a18e843f4e4951;

using Rust = import "../rust.capnp";
$Rust.parentModule("schema::core");

struct Error {
    code @0 :UInt16;
    message @1 :Text;
}

enum ErrorCode {
    unknown @0;
    noEntity @1;
    exists @2;
}
