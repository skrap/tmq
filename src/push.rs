use futures::{task, Async, AsyncSink, Poll, Sink, StartSend};

use tokio::reactor::PollEvented2;

use failure::Error;

use std::collections::VecDeque;

use zmq::{self, Context, SocketType};

use poll::Poller;
use socket::MioSocket;
use crate::Multipart;

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
    pub fn finish(self) -> Push<PollEvented2<MioSocket>> {
        Push {
            socket: PollEvented2::new(self.socket),
            buffer: VecDeque::new(),
            current: None,
        }
    }
}

pub struct Push<P: Poller> {
    socket: P,
    buffer: VecDeque<MultipartSegment>,
    current: Option<MultipartSegment>,
}

struct MultipartSegment {
    msg: zmq::Message,
    has_more: bool
}

impl<'a, P: Poller> Sink for Push<P> {
    type SinkItem = Multipart;
    type SinkError = Error;

    fn start_send(&mut self, parts: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let part_count = parts.len();
        for (i,msgpart) in parts.into_iter().enumerate() {
            self.buffer.push_back(MultipartSegment { msg: msgpart.into(), has_more: i + 1 < part_count} )
        }
        if self.current.is_none() {
            self.current = self.buffer.pop_front();
        }

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        debug!("Poll complete hit!");

        if let Some(msg) = self.current.take() {
            match self.socket.send_message(&msg.msg, msg.has_more)? {
                Async::NotReady => {
                    //Plop it back into our queue
                    self.current = Some(msg);
                    return Ok(Async::NotReady);
                }
                Async::Ready(()) => {
                    if let Some(segment) = self
                        .buffer
                        .pop_front()
                    {
                        //Message was sent, add a notify to be polled once more to check whether there are any messages.
                        task::current().notify();

                        self.current = Some(segment);
                        return Ok(Async::NotReady);
                    }
                }
            }
        }

        Ok(Async::Ready(()))
    }
}
