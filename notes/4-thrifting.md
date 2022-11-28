# Before I leave Cap n' Proto...

I got a bit fed up with capnp...

The need to handle exceptions at the read of _every individual attribute_
was getting painful. I had a function definition that looked something
like this:

```capnp
interface Node {
    readdir @5 () -> CM.Result(List(Node));
}
```

And that required this Rust code to parse client-side:

```rust
let mut request = client.readdir_request();
let reply = request.send().promise.await.unwrap();
let reply = reply.get().unwrap();
match reply.which().unwrap() {
    models_capnp::result::Which::Value(val) => match val {
        Ok(reader) => {
            let nodes = reader.into_iter();
            for node in nodes {
                let mut request = node.unwrap().getattr_request();
                let reply = request.send().promise.await.unwrap();
                match reply.get()?.which()? {
                    models_capnp::result::Which::Value(attrs) => attrs?.get_atime(),
                    models_capnp::result::Which::Error(_) => todo!(),
                }
            }
        }
        Err(_) => todo!(),
    },
    models_capnp::result::Which::Error(err) => todo!(),
};
```

Like.. holy shit. I am only reading one attribute of a file and not doing
anything with it. I would need to read out each individual component of the
file attributes for each file into my own struct to make some use of it.

Also, Capnp doesn't natively have a way to handle errors. I had to write my own
`Result` struct (see the `match` statement above) to handle application
level errors which involved another layer of exception handling hell.

What I've learned is that zero-copy protocols are good.. to a degree. If
the data I was reading was a big blob of data (file contents.. which I will
be reading at some point) and writing it to disk that would work great.
For reading lots of integer attributes that I want to copy into a nice
struct anyway, it's kind of pointless.

If capnp was able to parse the tree into a struct with safe read
operations up front that would work grand... but it doesn't.

# The grass is always greener...

Thrift seems like an interesting option. It was built to be an RPC IDL,
was made before gRPC was open sourced, and seems to have a nice
Rust binding. It was between Thrift and [Prost](https://crates.io/crates/prost),
but because I actually want an RPC IDL too I chose Thrift for now.
It seems Prost is maintained by the tokio team, which is a huge plus,
but before I give up and write my own RPC, I want to give Thrift a try.
Also, Apache foundation > the big Goog.
