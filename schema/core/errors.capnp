@0xc4a18e843f4e4951;

using Rust = import "../rust.capnp";
$Rust.parentModule("schema::core");

struct Error {
    message @0 :Text;
}
