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

# A revelation

I'm sitting here going over why I'm deciding to use RPC over HTTP, and I had
the realisation that I'm not comparing apples to apples. I could totally use
capnp messages over HTTP, and infact I would _need_ some sort of metadata
encapsulation around my capnp messages to identify what they are on the
receiving end if there is > 1 type (which there would be..if I wasn't using
capnp-rpc). Also, a
big reason I'm using QUIC is I want to operate like HTTP/3 where each
request/response stream is in its own.. stream. I do not want to run the RPC
over a single stream, maybe not on a single connection either. I've gone
about this all wrong.

I need to be able to define my own mechanism for how the QUIC protocol
will be used with my RPC. I also rely on some sort of service abstraction
such that it can identify and deserialise each wire message on the receiving
end. Each QUIC stream will have its own instance of the service operating
on it, and I think I will tie each file handle to a stream plus have some
metadata streams for handling mounts and other misc. stuff. This is
definitely sounding like HTTP/3 now, but alas I want to have as few parts
moving around in this stack as possible.

# The grass is always greener...

Thrift seems like an interesting option. It was built to be an RPC IDL,
was made before gRPC was open sourced, and seems to have a nice
Rust binding. It was between Thrift and [Prost](https://crates.io/crates/prost),
but because I actually want an RPC IDL too I chose Thrift for now.
It seems Prost is maintained by the tokio team, which is a huge plus,
but before I give up and write my own RPC, I want to give Thrift a try.
Also, Apache foundation > the big Goog.

Thrift is [very explicit](https://thrift-tutorial.readthedocs.io/en/latest/thrift-stack.html)
about how its architecture is split into multiple parts and you can
implement most parts yourself or use an existing one. Specifically,
it will allow me to define my own Transport layer which I'll need
to do if I don't want to use simple single-stream QUIC connections
between the client and server.

# Revelation two

I need a streaming RPC protocol, and gRPC is the only option.

However I can't hack HTTP/2 out of gRPC.

For now I think my best option is going to be using thrift but not its
RPC system. It's still a really nice serialiser.

NO WAIT

TWO SERVICES

# Thrift service handlers aren't mutable, and aren't async

Man, I don't want to conform to the norm and just use Prost, but given the
amount of other things im putting in my way, I think it's acceptable.

I don't want to rewrite thrift's code generator for mutable self on handlers,
as I feel I'm just missing something Rust-wise (Mutex? That's what [the example](https://github.com/apache/thrift/blob/9c0de2d1eb343910213c62325f73e3bb72361c22/tutorial/rs/src/bin/tutorial_server.rs#L83)
did).

Prost's code generator interface seems pretty straight forward so I'm gonna try that

# The plan is still not to use streaming RPC calls

There's some amazing freedom derived from having the client and server
independently establish Quic streams with each other. I could potentially
have 1:N connections between clients and servers.

The big notion I had was, imagine you sent a request to server A, but
it was distributed across server A, B and C. All 3 could connect to the
client and send responses.
