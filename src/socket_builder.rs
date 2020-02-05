
pub struct SocketBuilder<'a,BoundType> {
    context: &'a ::zmq::Context,
    socket_type: ::zmq::SocketType,
    _phantom: std::marker::PhantomData<BoundType>,
}

impl<'a,BoundType> SocketBuilder<'a,BoundType> where BoundType : std::convert::From<zmq::Socket> {
    pub(crate) fn new(context: &'a ::zmq::Context, socket_type: ::zmq::SocketType) -> Self {
        Self { context, socket_type, _phantom: std::default::Default::default() }
    }

    pub fn connect(self, endpoint: &str) -> crate::Result<BoundType> {
        let socket = self.context.socket(self.socket_type)?;
        socket.connect(endpoint)?;
        crate::Result::Ok(BoundType::from(socket))
    }

    pub fn bind(self, endpoint: &str) -> crate::Result<BoundType> {
        let socket = self.context.socket(self.socket_type)?;
        socket.bind(endpoint)?;
        crate::Result::Ok(BoundType::from(socket))
    }
}
