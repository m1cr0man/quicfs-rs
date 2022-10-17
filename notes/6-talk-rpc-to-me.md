# Talk RPC to me

Figuring out the capnp RPC, and how to implement it for real.

https://dev.to/kushalj/capn-proto-rpc-at-the-speed-of-rust-part-2-of-2-3n3a

This single dev.to article is like the only resource on how to do this.

Demo of seralisation:

```rust
pub fn write_to_stream() -> ::capnp::Result<()> {
    let mut message = ::capnp::message::Builder::new_default();
    let mut ts = message.init_root::<models_capnp::timestamp::Builder>();

    ts.set_nseconds(5);
    ts.set_seconds(10);

    serialize_packed::write_message(&mut ::std::io::stdout(), &message)
}
```

Run that in a main.rs and you get a bunch of binary. You can print the result with:

```bash
$ ./target/debug/quicfs-rs server --listen 127.0.0.1:8022 | capnp convert 'packed:text' schema/core/models.capnp Timestamp
{"seconds": "10", "nseconds": "5"}
```

## Oh god, it's !Send

Turns out that the rpc_system returns a future that does not
implement Send. This is important because Tokio does async in
a multi threaded fashion and needs Send-able futures for this.
As a result, I can't just `.await` the rpc_sytem in any old
async function (well, any initiaed from `tokio::spawn`). Instead
I have to use a [LocalSet](https://docs.rs/tokio/0.2.18/tokio/task/struct.LocalSet.html).

Some Googling led me to [this issue](https://github.com/tokio-rs/tokio/issues/2545#issuecomment-753689610)
and discussions resulted in the creation of tokio_util::task::LocalPoolHandle.
With this I was able to create a thread to handle the rpc_system.
What I might do in the end is change the server function in main.rs
to use a LocalSet for all connection handling (rpc system too) and
work out how to use a separate pool of threads for IO handling.
