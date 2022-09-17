# Tokio Sockets

Following [this video](https://www.youtube.com/watch?v=T2mWg91sx-o).

Got very lost when socket.read_line was giving an unsatisfied trait error.
Simple mistake, had to switch to using reader.read_line.
https://users.rust-lang.org/t/how-can-i-read-line-by-line-from-tokio-tcpstream/38665

split() is used so that we can delegate ownership of read to
the BufReader and still use write in the loop.

async move allows us to create a future, avoids simply writing a new function
Kind of like an async lambda I guess?

"Type inside async block must be known in this context" -> a generic type can't be
inferred. Use turbo fish ::\<T\> to specify the type

"Use of moved value tx" -> this is why we have to clone in loop

broadcast channel rx was not cloned in the loop, instead we use tx.subscribe().
Just a design quirk.


tokio::select! -> waitForAny/gather/go select

Spawn vs select: Consider the shared state. Select is better when there's
shared state, spawn is better otherwise.
