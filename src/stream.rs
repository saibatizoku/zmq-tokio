//! Streams for ØMQ sockets.
use std::io;
use std::ops::{Deref, DerefMut};

use futures::{Async, AsyncSink, Poll, StartSend};
use futures::{Stream, Sink};
use tokio_io::{AsyncRead, AsyncWrite};

pub use zmq::Message;
pub use zmq_futures::Listen;

/// A custom transport type for `Socket`.
pub struct SocketFramed<T> {
    socket: T,
}

impl<T> SocketFramed<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(socket: T) -> Self {
        SocketFramed { socket: socket }
    }
}

// TODO: Make this generic using a codec
impl<T> Sink for SocketFramed<T>
where
    T: AsyncRead + AsyncWrite,
{
    type SinkItem = Message;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Message) -> StartSend<Message, Self::SinkError> {
        trace!("SocketFramed::start_send()");
        match self.socket.write(item.deref()) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(AsyncSink::NotReady(item));
                } else {
                    return Err(e);
                }
            }
            Ok(_) => {
                return Ok(AsyncSink::Ready);
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}

// TODO: Make this generic using a codec
impl<T> Stream for SocketFramed<T>
where
    T: AsyncRead,
{
    type Item = Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut buf = Message::with_capacity(1024);
        trace!("SocketFramed::poll()");
        match self.socket.read(buf.deref_mut()) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(c) => {
                buf = Message::from_slice(&buf[..c]);
                Ok(Async::Ready(Some(buf)))
            }
        }
    }
}
