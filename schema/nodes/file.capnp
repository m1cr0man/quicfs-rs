@0x812aaa6c4bfa343e;

using Files = import "../core/files.capnp";
using CM = import "../core/models.capnp";
using import "node.capnp".Node;
using import "./directory.capnp".Directory;

interface File extends(Node) {
    readlink @0 () -> CM.Result(Text);

    # TODO return EOF bool in result
    read @1 (offset :UInt64, count :UInt64) -> CM.Result(ReadResult);

    # TODO write stability in result
    write @2 (offset :UInt64, data :Data) -> CM.Result(WriteResult);

    remove @3 () -> CM.BooleanResult;

    link @4 (directory :Directory, name :Text) -> CM.Result(File);

    struct ReadResult {
        data @0 :Data;
        eof @1 :Bool;
    }

    struct WriteResult {
        attributes @0 :Files.FileAttributes;
    }
}
