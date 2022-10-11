# The quest for Zero Copy

Rust? No GC? No data copy? What is this, the future or something?

## Flatbuffers

So HTTP everywhere kinda sucks...

With Flatbuffers, it's really easy to define my own protocol.
Flatbuffers are memory mapped byte arrays, no parsing. Perfect
for what I'm trying to do.

Gonna need this:
https://docs.rs/tokio/0.1.22/tokio/fs/index.html

Example flatbuffer:
https://github.com/google/flexbuffers/blob/master/samples/sample_flatbuffers.rs
https://docs.rs/flatbuffers/latest/flatbuffers/

Soooon
https://github.com/Sherlock-Holo/fuse3/blob/master/examples/src/path_memfs/main.rs

Info on flexbuffers:
https://google.github.io/flatbuffers/flexbuffers.html

`FlexBuffers is still slower than regular FlatBuffers though, so we recommend to only use it if you need it.`

## Capnproto

See section on malicious input:
https://capnproto.org/news/2014-06-17-capnproto-flatbuffers-sbe.html

And this:
https://github.com/dvidelabs/flatcc/blob/master/doc/security.md

TL;DR you can't really trust a flatbuffer. To quote:

```
If a buffer cannot be trusted, such as when receiving it over a public network, it may be the case that buffer type is known, but it is not known if someone uses an incorrect implementation of FlatBuffers, or if the buffer has somehow been corrupted in transit, or someone intentionally tampered with the buffer.
```

you can verify it but that only tells you it's safe, not
that it's correct.

I'm gonna use capnproto plus its RPC feature.
