use zmq::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller, SocketBuilder};

/// Create a builder for a DEALER socket.
pub fn dealer(context: &ZmqContext) -> SocketBuilder<DealerBuilderBound> {
    SocketBuilder::new(context, zmq::SocketType::DEALER)
}

pub struct DealerBuilderBound {
    socket: zmq::Socket,
}

impl std::convert::From<zmq::Socket> for DealerBuilderBound {
    fn from(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl DealerBuilderBound {
    pub fn finish(self) -> crate::Result<Dealer> {
        Ok(Dealer {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous DEALER Socket.
pub struct Dealer {
    inner: SenderReceiver,
}
impl_wrapper!(Dealer, SenderReceiver, inner);
impl_wrapper_sink!(Dealer, inner);
impl_wrapper_stream!(Dealer, inner);
impl_split!(Dealer, inner);
