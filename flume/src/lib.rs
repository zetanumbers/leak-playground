use leak_playground_std::marker::Forget;

pub mod rendezvous;
pub use rendezvous::rendezvous;

/// Create a bounded channel.
pub fn bounded<T: Forget>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::bounded(cap);
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create a bounded channel for the unforgettable parameter type `T`.
///
/// # Safety
///
/// `T` must not take ownership over itself.
pub unsafe fn bounded_unchecked<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::bounded(cap);
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create an unbounded channel.
pub fn unbounded<T: Forget>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create an unbounded channel for the unforgettable parameter type `T`.
///
/// # Safety
///
/// `T` must not take ownership over itself.
pub unsafe fn unbounded_unchecked<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

pub struct Sender<T> {
    inner: flume::Sender<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> Result<(), flume::SendError<T>> {
        self.inner.send(msg)
    }

    pub fn try_send(&self, msg: T) -> Result<(), flume::TrySendError<T>> {
        self.inner.try_send(msg)
    }

    pub fn send_async(&self, item: T) -> flume::r#async::SendFut<T> {
        self.inner.send_async(item)
    }

    pub fn into_send_async<'a>(self, item: T) -> flume::r#async::SendFut<'a, T> {
        self.inner.into_send_async(item)
    }
}

pub struct Receiver<T> {
    inner: flume::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, flume::RecvError> {
        self.inner.recv()
    }

    pub fn try_recv(&self) -> Result<T, flume::TryRecvError> {
        self.inner.try_recv()
    }

    pub fn recv_async(&self) -> flume::r#async::RecvFut<'_, T> {
        self.inner.recv_async()
    }

    pub fn into_recv_async<'a>(self) -> flume::r#async::RecvFut<'a, T> {
        self.inner.into_recv_async()
    }
}

unsafe impl<T: Forget> Forget for Sender<T> {}
unsafe impl<T: Forget> Forget for Receiver<T> {}
