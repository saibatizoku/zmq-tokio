//! ØMQ (ZeroMQ) for tokio
//! ======================
//!
//! Run ØMQ sockets using `tokio` reactors, futures, etc.
//!
//! Examples
//! ========
//!
//! Sending and receiving simple messages with futures
//! --------------------------------------------------
//!
//! A PAIR of sockets is created. The `sender` socket sends
//! a message, and the `receiver` gets it.
//!
//! Everything runs within on a tokio reactor.
//!
//! ```
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate zmq_tokio;
//!
//! use futures::Future;
//! use tokio_core::reactor::Core;
//!
//! use zmq_tokio::{Context, Socket, PAIR};
//!
//! const TEST_ADDR: &str = "inproc://test";
//!
//!
//! fn main() {
//!     let mut reactor = Core::new().unwrap();
//!     let context = Context::new();
//!
//!     let recvr = context.socket(PAIR, &reactor.handle()).unwrap();
//!     let _ = recvr.bind(TEST_ADDR).unwrap();
//!
//!     let sendr = context.socket(PAIR, &reactor.handle()).unwrap();
//!     let _ = sendr.connect(TEST_ADDR).unwrap();
//!
//!     // Step 1: send any type implementing `Into<Message>`,
//!     //         meaning `&[u8]`, `Vec<u8>`, `String`, `&str`,
//!     //         and `Message` itself.
//!     let send_future = sendr.send("this message will be sent");
//!
//!     // Step 2: receive the message on the pair socket
//!     let recv_msg = send_future.and_then(|_| {
//!         recvr.recv()
//!     });
//!
//!     // Step 3: process the message and exit
//!     let process_msg = recv_msg.and_then(|msg| {
//!         assert_eq!(msg.as_str(), Some("this message will be sent"));
//!         Ok(())
//!     });
//!
//!     let _ = reactor.run(process_msg).unwrap();
//!
//!     // Exit our program, playing nice.
//!     ::std::process::exit(0);
//! }
//! ```
//!
//! Sending and receiving multi-part messages with futures
//! ------------------------------------------------------
//!
//! This time we use `PUSH` and `PULL` sockets to move multi-part messages.
//!
//! Remember that ZMQ will either send all parts or none at all.
//! Save goes for receiving.
//!
//! ```
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate zmq_tokio;
//!
//! use futures::Future;
//! use tokio_core::reactor::Core;
//!
//! use zmq_tokio::{Context, Socket, PULL, PUSH};
//!
//! const TEST_ADDR: &str = "inproc://test";
//!
//! fn main() {
//!     let mut reactor = Core::new().unwrap();
//!     let context = Context::new();
//!
//!     let recvr = context.socket(PULL, &reactor.handle()).unwrap();
//!     let _ = recvr.bind(TEST_ADDR).unwrap();
//!
//!     let sendr = context.socket(PUSH, &reactor.handle()).unwrap();
//!     let _ = sendr.connect(TEST_ADDR).unwrap();
//!
//!     let msgs: Vec<Vec<u8>> = vec![b"hello".to_vec(), b"goodbye".to_vec()];
//!     // Step 1: send a vector of byte-vectors, `Vec<Vec<u8>>`
//!     let send_future = sendr.send_multipart(msgs);
//!
//!     // Step 2: receive the complete multi-part message
//!     let recv_msg = send_future.and_then(|_| {
//!         recvr.recv_multipart()
//!     });
//!
//!     // Step 3: process the message and exit
//!     let process_msg = recv_msg.and_then(|msgs| {
//!         assert_eq!(msgs[0].as_str(), Some("hello"));
//!         assert_eq!(msgs[1].as_str(), Some("goodbye"));
//!         Ok(())
//!     });
//!
//!     let _ = reactor.run(process_msg).unwrap();
//!
//!     // Exit our program, playing nice.
//!     ::std::process::exit(0);
//! }
//! ```
//!
//! Manual use of tokio tranports with `Sink` and `Stream`
//! ------------------------------------------------------
//!
//! This time, we use `PUB`-`SUB` sockets to send and receive a message.
//!
//! ```
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate zmq_tokio;
//!
//! use futures::{Future, Sink, Stream, stream};
//! use tokio_core::reactor::Core;
//!
//! use zmq_tokio::{Context, Message, Socket, PUB, SUB};
//!
//! const TEST_ADDR: &str = "inproc://test";
//!
//!
//! fn main() {
//!     let mut reactor = Core::new().unwrap();
//!     let context = Context::new();
//!
//!     let recvr = context.socket(SUB, &reactor.handle()).unwrap();
//!     let _ = recvr.bind(TEST_ADDR).unwrap();
//!     let _ = recvr.set_subscribe(b"").unwrap();
//!
//!     let sendr = context.socket(PUB, &reactor.handle()).unwrap();
//!     let _ = sendr.connect(TEST_ADDR).unwrap();
//!
//!
//!     let (_, recvr_split_stream) = recvr.framed().split();
//!     let (sendr_split_sink, _) = sendr.framed().split();
//!
//!     let msg = Message::from_slice(b"hello there");
//!
//!     // Step 1: start a stream with only one item.
//!     let start_stream = stream::iter_ok::<_, ()>(vec![(sendr_split_sink, recvr_split_stream, msg)]);
//!
//!     // Step 2: send the message
//!     let send_msg = start_stream.and_then(|(sink, stream, msg)| {
//!             // send a message to the receiver.
//!             // return a future with the receiver
//!             let _ = sink.send(msg);
//!             Ok(stream)
//!         });
//!
//!     // Step 3: read the message
//!     let fetch_msg = send_msg.for_each(|stream| {
//!             // process the first response that the
//!             // receiver gets.
//!             // Assert that it equals the message sent
//!             // by the sender.
//!             // returns `Ok(())` when the stream ends.
//!             let _ = stream.into_future().and_then(|(response, _)| {
//!                 match response {
//!                     Some(msg) => assert_eq!(msg.as_str(), Some("hello there")),
//!                     None => panic!("expected a response"),
//!                 }
//!                 Ok(())
//!             });
//!             Ok(())
//!         });
//!
//!     // Run the stream
//!     let _ = reactor.run(fetch_msg).unwrap();
//!
//!     // Exit our program, playing nice.
//!     ::std::process::exit(0);
//! }
//! ```
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate log;
extern crate mio;
extern crate tokio_core;
extern crate tokio_io;
extern crate zmq;
extern crate zmq_futures;
extern crate zmq_mio;

pub mod future;
pub mod stream;

use std::io;
use std::io::{Read, Write};

use futures::Poll;
use futures::stream::{Empty, empty};

use tokio_core::reactor::{Handle, PollEvented};
use tokio_io::{AsyncRead, AsyncWrite};

use zmq::Sendable;

use self::future::*;
use self::stream::*;


pub use zmq::{Message, SocketType};

pub use self::SocketType::{DEALER, PAIR, PUB, PULL, PUSH, REP, REQ, ROUTER, STREAM, SUB, XPUB, XSUB};

/// Wrapper for `zmq::Context`.
#[derive(Clone, Default)]
pub struct Context {
    inner: zmq_mio::Context,
}

impl Context {
    /// Create a new ØMQ context for the `tokio` framework.
    pub fn new() -> Context {
        Context {
            inner: zmq_mio::Context::new(),
        }
    }

    /// Create a new ØMQ socket for the `tokio` framework.
    pub fn socket(&self, typ: SocketType, handle: &Handle) -> io::Result<Socket> {
        Ok(Socket::new(try!(self.inner.socket(typ)), handle)?)
    }

    /// Try to destroy the underlying context. This is different than the destructor;
    /// the destructor will loop when zmq_ctx_destroy returns EINTR.
    pub fn destroy(&mut self) -> io::Result<()> {
        self.inner.destroy()
    }

    /// Get a cloned instance of the underlying `zmq_mio::Context`.
    pub fn get_inner(&self) -> zmq_mio::Context {
        self.inner.clone()
    }
}

/// Poll-evented ØMQ socket. Can be used directly on transports implementing
/// `futures::stream::Stream` and `futures::sink::Sink`.
pub struct Socket {
    io: PollEvented<zmq_mio::Socket>,
}

impl Socket {
    /// Create a new poll-evented ØMQ socket, along with a tokio reactor handle
    /// to drive its event-loop.
    fn new(socket: zmq_mio::Socket, handle: &Handle) -> io::Result<Self> {
        let io = try!(PollEvented::new(socket, handle));
        let socket = Socket { io };
        Ok(socket)
    }

    /// A reference to the underlying `zmq_mio::Socket`. Useful
    /// for building futures.
    pub fn get_ref(&self) -> &zmq_mio::Socket {
        self.io.get_ref()
    }

    /// Returns a mutable reference to the underlying `zmq_mio::Socket`.
    /// Useful for setting socket options at runtime.
    pub fn get_mut(&mut self) -> &mut zmq_mio::Socket {
        self.io.get_mut()
    }


    /// Bind the underlying socket to the given address.
    pub fn bind(&self, endpoint: &str) -> io::Result<()> {
        self.get_ref().bind(endpoint)
    }

    /// Connect a socket.
    pub fn connect(&self, endpoint: &str) -> io::Result<()> {
        self.get_ref().connect(endpoint)
    }

    /// Disconnect a previously connected socket.
    pub fn disconnect(&self, endpoint: &str) -> io::Result<()> {
        self.get_ref().disconnect(endpoint)
    }

    /// Sends a type implementing `Into<Message>` as a `Future`.
    pub fn send<'a, M: Into<Message>>(&'a self, message: M) -> SendMessage<'a, Socket> {
        SendMessage::new(self, message)
    }

    /// Sends a type implementing `Into<Message>` as a `Future`.
    pub fn send_multipart<'a, I, U>(&'a self, messages: I) -> SendMultipartMessage<'a, Socket>
    where
        I: IntoIterator<Item = U>,
        U: Into<Vec<u8>>,
    {
        SendMultipartMessage::new(self, messages)
    }

    /// Returns a `Future` that resolves into a `Message`
    pub fn recv<'a>(&'a self) -> ReceiveMessage<'a, Socket> {
        ReceiveMessage::new(self)
    }

    /// Returns a `Future` that resolves into a `Vec<Message>`
    pub fn recv_multipart<'a>(&'a self) -> ReceiveMultipartMessage<'a, Socket> {
        ReceiveMultipartMessage::new(self)
    }

    /// Return the type of this socket.
    pub fn get_socket_type(&self) -> io::Result<zmq::SocketType> {
        self.get_ref().get_socket_type()
    }

    /// Return true if there are more frames of a multipart message to receive.
    pub fn get_rcvmore(&self) -> io::Result<bool> {
        self.get_ref().get_rcvmore().map_err(|e| e.into())
    }

    /// Subscribe the underlying socket to the given prefix.
    pub fn set_subscribe(&self, value: &[u8]) -> io::Result<()> {
        self.get_ref().set_subscribe(value)
    }

    /// Unsubscribe the underlying socket from the given prefix.
    pub fn set_unsubscribe(&self, value: &[u8]) -> io::Result<()> {
        self.get_ref().set_unsubscribe(value)
    }

    pub fn framed(self) -> SocketFramed<Self> {
        SocketFramed::new(self)
    }
}

unsafe impl Send for Socket {}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.io.read(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.io.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsyncRead for Socket {}

impl AsyncWrite for Socket {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        Ok(().into())
    }
}

impl MessageSend for Socket {
    fn send<T>(&self, msg: T, flags: i32) -> io::Result<()>
    where
        T: Sendable,
    {
        self.get_ref().send(msg, flags).map_err(|e| e.into())
    }
    fn send_multipart<I, T>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<Message>,
    {
        self.get_ref()
            .send_multipart(iter, flags)
            .map_err(|e| e.into())
    }
}

impl MessageRecv for Socket {
    fn get_rcvmore(&self) -> io::Result<bool> {
        self.get_ref().get_rcvmore().map_err(|e| e.into())
    }

    fn recv(&self, msg: &mut Message, flags: i32) -> io::Result<()> {
        self.get_ref().recv(msg, flags).map_err(|e| e.into())
    }
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        self.get_ref().recv_into(buf, flags).map_err(|e| e.into())
    }

    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        self.get_ref().recv_msg(flags).map_err(|e| e.into())
    }

    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        self.get_ref().recv_bytes(flags).map_err(|e| e.into())
    }

    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        self.get_ref().recv_string(flags).map_err(|e| e.into())
    }

    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        self.get_ref().recv_multipart(flags).map_err(|e| e.into())
    }
}

impl Listen for Socket {
    type Stream = Empty<(), ()>;
    fn listen(&self) -> Empty<(), ()> {
        empty()
    }
}
