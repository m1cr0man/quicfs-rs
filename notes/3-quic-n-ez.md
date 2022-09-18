# QUIC n' EZ

## Quiche or Quinn?

I want to use Quinn, but it doesn't implement H3 which is a pain..

I'll need to use Quiche if I want h3...

But there seems to be some [big issues](https://github.com/cloudflare/quiche/issues/1273)
and [delayed PRs](https://github.com/cloudflare/quiche/pull/1121) all over
the place on Quiche. Also, my own last experience trying to set it up
was horrific.

Quinn has more contributors, less open issues, more closed PRs.

I'll implement H3 myself if I need to I think.

If I ever try quiche, note to self about UDP socket not having a split() method:

https://github.com/tokio-rs/tokio/discussions/3755

Had to set up cert parsing myself. Because im using ed25519 keys and not RSA,
had to figure out how to parse correctly. Best example was in the
[rustls_pemfile docs](https://docs.rs/rustls-pemfile/latest/rustls_pemfile/#functions).

After that, appropriately mapped/replaced the tcpstream with the quinn connection
and converted server. Next is client.

Found [this nice issue](https://github.com/quinn-rs/quinn/issues/950) too.

## Bidirectional vs unidirectional streams

It's important to note that QUIC streams are not like streams in other
protocols. They are only initiated when one side sends data on the
given stream number. Consider a chat service where we want bidirectional
communication between client and server. If the client initates the stream,
we may want to send data to them at a time that isn't possible. The opposite
applies too. In this case, we use a unidirectional stream for each direction,
server -> client and client -> server, each managed by its respective parent.
That way we can send data in both directions at any time in the connection
lifecycle.
