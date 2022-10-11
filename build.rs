extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .output_path("src/schema")
        .file("schema/server.capnp")
        .run()
        .expect("schema compiler command");
}
