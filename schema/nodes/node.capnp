@0xac3b9e27f56a9719;

using Rust = import "../rust.capnp";
$Rust.parentModule("schema::nodes");

using Files = import "../core/files.capnp";
using CM = import "../core/models.capnp";

interface Node {
    getattr @0 () -> CM.Result(Files.FileAttributes);
    access  @1 () -> CM.Result(AccessResult);
    rename  @2 (name: Text) -> CM.Result(Files.FileAttributes);
    fsstat  @3 () -> CM.Result(Files.FSStat);
    commit  @4 () -> CM.BooleanResult;

    struct AccessResult {
        rights @0 :UInt16;
    }
}
