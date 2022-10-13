@0xed3d46f9a6980e4a;

using Rust = import "rust.capnp";
$Rust.parentModule("schema");

using CM = import "core/models.capnp";
using import "nodes/directory.capnp".Directory;

interface Server {
    null @0 () -> ();

    mount @1 (path: Text) -> CM.Result(Directory);
}
