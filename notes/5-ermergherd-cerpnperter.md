# Ermergherd, Cerpn Perter

I thought I was being smart with main.capnp files
in each subdirectory, like so, for easier imports:

```
using import "directory.capnp".Directory;
using import "file.capnp".File;
using import "node.capnp".Node;
```

This gave me this compiler error:

```
$ cargo build
   Compiling quicfs-rs v0.1.0 (/home/m1cr0man/quicfs-rs)
error: failed to run custom build command for `quicfs-rs v0.1.0 (/home/m1cr0man/quicfs-rs)`

Caused by:
  process didn't exit successfully: `/home/m1cr0man/quicfs-rs/target/debug/build/quicfs-rs-4a1374e16c11b1b7/build-script-build` (exit status: 101)
  --- stderr
  thread 'main' panicked at 'no entry found for key', /home/m1cr0man/.cargo/registry/src/github.com-1ecc6299db9ec823/capnpc-0.14.9/src/codegen_types.rs:183:29
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

Also, I was really confused by the build instructions. The actual steps were:

- Create a build.rs in the root of the repo.
- Write the required bits into it, make sure to call output_path.
- Update the repo's Cargo.toml to set the build script.
- Run `cargo build`.

## But wait, there's more!

Had to do some annotation shite and add mod.rs files everywhere to actually get
the generated code to be importable.
