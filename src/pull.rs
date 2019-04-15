use crate::Multipart;

use futures::{Async, Poll, Stream};

use tokio::reactor::PollEvented2;

use failure::Error;

use zmq::{self, Context, SocketType};

use poll::Poller;
use socket::MioSocket;

pub fn pull(context: &Context) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a Context,
}

pub struct PullBuilderBounded {
    socket: MioSocket,
}

impl<'a> PullBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<PullBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<PullBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.bind(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl PullBuilderBounded {
    pub fn finish(self) -> Pull<PollEvented2<MioSocket>> {
        Pull {
            socket: PollEvented2::new(self.socket),
            buffer: zmq::Message::new(),
            messages: Multipart::new(),
        }
    }
}

pub struct Pull<P: Poller> {
    socket: P,
    buffer: zmq::Message,
    messages: Multipart,
}

impl<P: Poller> Stream for Pull<P> {
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Poll Hit!");

        loop {
            match self.socket.recv_message(&mut self.buffer)? {
                Async::Ready(()) => {
                    self.messages.push(std::mem::replace(&mut self.buffer, zmq::Message::new()));
                    match self.socket.get_rcvmore() {
                        Ok(true) => (), // there's more to get, so let loop {} run again.
                        Ok(false) => {
                            break Ok(Async::Ready(Some(std::mem::replace(&mut self.messages, Multipart::new()))))
                        }
                        Err(e) => break Err(e)
                    }
                }
                Async::NotReady => {
                    break Ok(Async::NotReady);
                }
            }
        }
    }
}
