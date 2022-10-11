@0xb71c5dea9b89b729;

using Files = import "../core/files.capnp";
using CM = import "../core/models.capnp";
using import "node.capnp".Node;
using import "file.capnp".File;

interface Directory extends(Node) {
    lookup @0 LookupRequest -> CM.Result(Node);

    # TODO advanced mode arguments
    create @1 CreateRequest -> CM.Result(File);

    mkdir @2 CreateRequest -> CM.Result(Directory);

    rmdir @3 (recursive :Bool = false) -> CM.BooleanResult;

    symlink @4 SymlinkRequest -> CM.Result(File);

    # TODO cookie, seeking, return EOF bool in result
    readdir @5 () -> CM.Result(List(Node));

    struct LookupRequest {
        name @0 :Text;
    }

    struct CreateRequest {
        name @0 :Text;
        mode @1 :Files.CreateMode;
    }

    struct SymlinkRequest {
        name @0   :Text;
        mode @1   :Files.CreateMode;
        target @2 :Text;
    }
}
