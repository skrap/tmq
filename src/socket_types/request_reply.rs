use crate::{poll::ZmqPoller, Multipart, SocketBuilder};
use zmq::{self, Context as ZmqContext};

/// Create a builder for a REQ socket
pub fn request(context: &ZmqContext) -> SocketBuilder<RequestBuilderBound> {
    SocketBuilder::new(context, zmq::SocketType::REQ)
}

pub struct RequestBuilderBound {
    socket: zmq::Socket,
}

impl std::convert::From<zmq::Socket> for RequestBuilderBound {
    fn from(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl RequestBuilderBound {
    pub fn finish(self) -> crate::Result<RequestSender> {
        Ok(RequestSender {
            poller: ZmqPoller::from_zmq_socket(self.socket)?,
        })
    }
}

pub struct RequestSender {
    poller: ZmqPoller,
}

impl crate::socket::AsZmqSocket for RequestSender {
    #[inline]
    fn get_socket(&self) -> &zmq::Socket {
        self.poller.get_socket()
    }
}

impl RequestSender {
    pub async fn send(self, mut msg: Multipart) -> crate::Result<RequestReceiver> {
        futures::future::poll_fn(|cx| self.poller.multipart_flush(cx, &mut msg)).await?;
        Ok(RequestReceiver {
            poller: self.poller,
        })
    }
}

/// Create a builder for a REP socket
pub fn reply(context: &ZmqContext) -> SocketBuilder<ReplyBuilderBound> {
    SocketBuilder::new(context, zmq::SocketType::REP)
}

pub struct ReplyBuilderBound {
    socket: zmq::Socket,
}

impl std::convert::From<zmq::Socket> for ReplyBuilderBound {
    fn from(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl ReplyBuilderBound {
    pub fn finish(self) -> crate::Result<RequestReceiver> {
        Ok(RequestReceiver {
            poller: ZmqPoller::from_zmq_socket(self.socket)?,
        })
    }
}

pub struct RequestReceiver {
    poller: ZmqPoller,
}

impl crate::socket::AsZmqSocket for RequestReceiver {
    #[inline]
    fn get_socket(&self) -> &zmq::Socket {
        self.poller.get_socket()
    }
}

impl RequestReceiver {
    pub async fn recv(self) -> crate::Result<(Multipart, RequestSender)> {
        let msg = futures::future::poll_fn(|cx| self.poller.multipart_recv(cx)).await?;
        Ok((
            msg,
            RequestSender {
                poller: self.poller,
            },
        ))
    }
}
