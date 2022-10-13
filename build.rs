extern crate capnpc;
use glob::glob;

const SCHEMADIR: &str = "schema";

fn main() {
    let mut compiler = ::capnpc::CompilerCommand::new();
    compiler.src_prefix(SCHEMADIR);
    compiler.output_path("src/".to_owned() + SCHEMADIR);

    for entry in glob(("".to_owned() + SCHEMADIR + "/**/*.capnp").as_str())
        .expect("Failed to parse glob pattern")
    {
        match entry {
            Ok(path) => {
                // Ignore rust.capnp
                if !path.ends_with("rust.capnp") {
                    println!("{}", path.display());
                    compiler.file(path);
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }

    compiler.run().expect("schema compiler command");
}
