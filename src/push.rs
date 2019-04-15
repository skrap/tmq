use futures::{task, Async, AsyncSink, Poll, Sink, StartSend};

use tokio::reactor::PollEvented2;

use failure::Error;

use std::collections::VecDeque;

use zmq::{self, Context, SocketType};

use poll::Poller;
use socket::MioSocket;

pub fn push(context: &Context) -> PushBuilder {
    PushBuilder { context }
}

pub struct PushBuilder<'a> {
    context: &'a Context,
}

pub struct PushBuilderBounded {
    socket: MioSocket,
}

impl<'a> PushBuilder<'a> {
    pub fn bind(self, endpoint: &str) -> Result<PushBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.bind(endpoint)?;

        Ok(PushBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn connect(self, endpoint: &str) -> Result<PushBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.connect(endpoint)?;

        Ok(PushBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl PushBuilderBounded {
    pub fn finish<M: Into<zmq::Message>>(self) -> Push<M, PollEvented2<MioSocket>> {
        Push {
            socket: PollEvented2::new(self.socket),
            buffer: VecDeque::new(),
            current: None,
        }
    }
}

pub struct Push<M: Into<zmq::Message>, P: Poller> {
    socket: P,
    buffer: VecDeque<M>,
    current: Option<zmq::Message>,
}

impl<P: Poller, M: Into<zmq::Message>> Sink for Push<M, P> {
    type SinkItem = M;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if self.current.is_none() {
            self.current = Some(item.into());
        } else {
            self.buffer.push_back(item);
        }

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        debug!("Poll complete hit!");

        if let Some(msg) = self.current.take() {
            match self.socket.send_message(&msg)? {
                Async::NotReady => {
                    //Plop it back into our queue
                    self.current = Some(msg);
                    return Ok(Async::NotReady);
                }
                Async::Ready(()) => {
                    if let Some(new_msg) = self.buffer.pop_front().map(|val| val.into()) {
                        //Message was sent, add a notify to be polled once more to check whether there are any messages.
                        task::current().notify();

                        self.current = Some(new_msg);
                        return Ok(Async::NotReady);
                    }
                }
            }
        }

        Ok(Async::Ready(()))
    }
}
