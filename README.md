ØMQ (ZeroMQ) for tokio
======================

Run asynchronous ØMQ sockets with Rust's `tokio` framework.

>   [Rust](https://www.rust-lang.org/) is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.

>   [ZeroMQ](http://zeromq.org/) (also known as ØMQ, 0MQ, or zmq) looks like an embeddable networking library but acts like a concurrency framework. It gives you sockets that carry atomic messages across various transports like in-process, inter-process, TCP, and multicast. You can connect sockets N-to-N with patterns like fan-out, pub-sub, task distribution, and request-reply. It's fast enough to be the fabric for clustered products. Its asynchronous I/O model gives you scalable multicore applications, built as asynchronous message-processing tasks. It has a score of language APIs and runs on most operating systems. ZeroMQ is from iMatix and is LGPLv3 open source.

>   [mio](https://github.com/carllerche/mio). Lightweight non-blocking IO.

>   [Tokio](https://tokio.rs/). A platform for writing fast networking code with Rust.


This crate uses [rust-zmq](https://github.com/erickt/rust-zmq)'s bindings.

Status
------

[![Build Status](https://travis-ci.org/saibatizoku/zmq-tokio.svg?branch=master)](https://travis-ci.org/saibatizoku/zmq-tokio)

* __[Documentation](https://saibatizoku.github.io/zmq-tokio/zmq_tokio/index.html)__


This is the barely budding seed of providing access to
ZeroMQ via the tokio async I/O abstraction.

This project is in its very infancy. Do not expect to be able to build
something useful on top of this (yet). The API will certainly change
wildly before approaching some kind of stability.

The underlying library API is still not complete.

Justification
-------------

While it may cause controversy amongst some network developers due to its highly _non-neutral_ approach to messaging, ZeroMQ is a [fully-documented](http://zguide.zeromq.org/page:all) library that is meant to act as an _intelligent transport layer_ for messaging patterns. It's extremely portable across OS platforms, as well as across programming languages.

Rust's bindings for the ZeroMQ library, [rust-zmq](https://github.com/erickt/rust-zmq), wrap the C++ library into a very ergonomic implementation of **most** of the API.

While ZeroMQ's model is asynchronous by nature, the user API states that there is a need for IO error handling, specifically for the case of `std::io::ErrorKind::WouldBlock`. Therefore, to use ZMQ sockets asynchronously, it is necessary to have some higher-level code that can enforce correct handling of non-blocking messaging. Which is what `tokio` is for. `mio` helps build the bridge that connects the synchronous with the asynchronous, by adding non-blocking compatibility, as well as a polling mechanism that is meant to be cross-platform.

Integrating ZeroMQ into the `tokio` ecosystem is a perfect match.

It can be used to extend existing ZeroMQ infrastructure, as well as to create new network components that are guaranteed to be safe, fast, and concurrent. The Rust way.


Usage
=====

For a typical crate, you need at least `futures`, `tokio_core` and `zmq_tokio`. Realistically, you might have to install `tokio_io`, etc. That's really up to your design requirements.

Add this to your `Cargo.toml`:

```Cargo
[dependencies]
futures = "0.1"
tokio_core = "0.1"
zmq_tokio = { git = "https://github.com/saibatizoku/zmq-tokio.git" }
```

Then, in your crates main module (`src/lib.rs` or `src/main.rs`):

```rust
extern crate futures;
extern crate tokio_core;
extern crate zmq_tokio;
```

Examples
========

Sending and receiving simple messages with futures
--------------------------------------------------

A PAIR of sockets is created. The `sender` socket sends
a message, and the `receiver` gets it.

Everything runs within on a tokio reactor.

```rust
extern crate futures;
extern crate tokio_core;
extern crate zmq_tokio;

use futures::Future;
use tokio_core::reactor::Core;

use zmq_tokio::{Context, Socket, PAIR};

const TEST_ADDR: &str = "inproc://test";


fn main() {
    let mut reactor = Core::new().unwrap();
    let context = Context::new();

    let recvr = context.socket(PAIR, &reactor.handle()).unwrap();
    let _ = recvr.bind(TEST_ADDR).unwrap();

    let sendr = context.socket(PAIR, &reactor.handle()).unwrap();
    let _ = sendr.connect(TEST_ADDR).unwrap();

    // Step 1: send any type implementing `Into<zmq::Message>`,
    //         meaning `&[u8]`, `Vec<u8>`, `String`, `&str`,
    //         and `zmq::Message` itself.
    let send_future = sendr.send("this message will be sent");

    // Step 2: receive the message on the pair socket
    let recv_msg = send_future.and_then(|_| {
        recvr.recv()
    });

    // Step 3: process the message and exit
    let process_msg = recv_msg.and_then(|msg| {
        assert_eq!(msg.as_str(), Some("this message will be sent"));
        Ok(())
    });

    let _ = reactor.run(process_msg).unwrap();

    // Exit our program, playing nice.
    ::std::process::exit(0);
}
```

Sending and receiving multi-part messages with futures
------------------------------------------------------

This time we use `PUSH` and `PULL` sockets to move multi-part messages.

Remember that ZMQ will either send all parts or none at all.
Save goes for receiving.

```rust
extern crate futures;
extern crate tokio_core;
extern crate zmq_tokio;

use futures::Future;
use tokio_core::reactor::Core;

use zmq_tokio::{Context, Socket, PULL, PUSH};

const TEST_ADDR: &str = "inproc://test";

fn main() {
    let mut reactor = Core::new().unwrap();
    let context = Context::new();

    let recvr = context.socket(PULL, &reactor.handle()).unwrap();
    let _ = recvr.bind(TEST_ADDR).unwrap();

    let sendr = context.socket(PUSH, &reactor.handle()).unwrap();
    let _ = sendr.connect(TEST_ADDR).unwrap();

    let msgs: Vec<Vec<u8>> = vec![b"hello".to_vec(), b"goodbye".to_vec()];
    // Step 1: send a vector of byte-vectors, `Vec<Vec<u8>>`
    let send_future = sendr.send_multipart(msgs);

    // Step 2: receive the complete multi-part message
    let recv_msg = send_future.and_then(|_| {
        recvr.recv_multipart()
    });

    // Step 3: process the message and exit
    let process_msg = recv_msg.and_then(|msgs| {
        assert_eq!(msgs[0].as_str(), Some("hello"));
        assert_eq!(msgs[1].as_str(), Some("goodbye"));
        Ok(())
    });

    let _ = reactor.run(process_msg).unwrap();

    // Exit our program, playing nice.
    ::std::process::exit(0);
}
```

Manual use of tokio tranports with `Sink` and `Stream`
------------------------------------------------------

This time, we use `PUB`-`SUB` sockets to send and receive a message.

```rust
extern crate futures;
extern crate tokio_core;
extern crate zmq_tokio;

use futures::{Future, Sink, Stream, stream};
use tokio_core::reactor::Core;

use zmq_tokio::{Context, Message, Socket, PUB, SUB};

const TEST_ADDR: &str = "inproc://test";


fn main() {
    let mut reactor = Core::new().unwrap();
    let context = Context::new();

    let recvr = context.socket(SUB, &reactor.handle()).unwrap();
    let _ = recvr.bind(TEST_ADDR).unwrap();
    let _ = recvr.set_subscribe(b"").unwrap();

    let sendr = context.socket(PUB, &reactor.handle()).unwrap();
    let _ = sendr.connect(TEST_ADDR).unwrap();


    let (_, recvr_split_stream) = recvr.framed().split();
    let (sendr_split_sink, _) = sendr.framed().split();

    let msg = Message::from_slice(b"hello there");

    // Step 1: start a stream with only one item.
    let start_stream = stream::iter_ok::<_, ()>(vec![(sendr_split_sink, recvr_split_stream, msg)]);

    // Step 2: send the message
    let send_msg = start_stream.and_then(|(sink, stream, msg)| {
            // send a message to the receiver.
            // return a future with the receiver
            let _ = sink.send(msg);
            Ok(stream)
        });

    // Step 3: read the message
    let fetch_msg = send_msg.for_each(|stream| {
            // process the first response that the
            // receiver gets.
            // Assert that it equals the message sent
            // by the sender.
            // returns `Ok(())` when the stream ends.
            let _ = stream.into_future().and_then(|(response, _)| {
                match response {
                    Some(msg) => assert_eq!(msg.as_str(), Some("hello there")),
                    None => panic!("expected a response"),
                }
                Ok(())
            });
            Ok(())
        });

    // Run the stream
    let _ = reactor.run(fetch_msg).unwrap();

    // Exit our program, playing nice.
    ::std::process::exit(0);
}
```
