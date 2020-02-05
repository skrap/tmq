use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, Sender, SocketBuilder};

/// Create a builder for a PUB socket.
pub fn publish(context: &ZmqContext) -> SocketBuilder<PublishBuilderBound> {
    SocketBuilder::new(context, zmq::SocketType::PUB)
}

pub struct PublishBuilderBound {
    socket: zmq::Socket,
}

impl std::convert::From<zmq::Socket> for PublishBuilderBound {
    fn from(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl PublishBuilderBound {
    pub fn finish(self) -> crate::Result<Publish> {
        Ok(Publish {
            inner: Sender::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous PUB socket.
pub struct Publish {
    inner: Sender,
}
impl_wrapper!(Publish, Sender, inner);
impl_wrapper_sink!(Publish, inner);
