# Writing a HTTP3 Filesystem

I took some time to try writing it in python with aioquic again,
but I didn't find a fuse library that was up to date so I decided
against it. I really need to commit to learning Rust, no backing out.

I figured out the quiche library one evening by rewriting the
http3 server + client examples by hand. I didn't like them though,
they were far too verbose for my needs and I made building an
abstraction my first priority.

## Mutable pointers

I was really stumped by this error for a while:

```
Cannot borrow `self.conn` as mutable, as it is behind a `&` reference
self is a `&` reference, so the dat it refers to cannot be borrowed as mutable.
```

The few grammar mistakes kept pulling my attention away from the issue. I
really didn't understand what was wrong until
I looked at the line above, where a tiny grey underline
was recommending I change `&self` to `&mut self`.

This error was saying that I couldn't have a mutable connection because
it was within a non-mutable reference - which was `self` in this case. This
error would be much clearer if written like so:

`Cannot borrow self.conn as mutable from non mutable reference self.`

Once I change it to `&mut self` all was good.
