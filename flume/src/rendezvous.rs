//! Unforgettable-types-friendly rendezvous channel.
//!
//! # Examples
//!
//! ```rust,compile_fail
//! use std::{future::Future, pin::Pin, task};
//! use noop_waker::noop_waker;
//! use leak_playground_flume::*;
//! use leak_playground_std::marker::{Forget, Unforget};
//!
//! let i = 37;
//! {
//!    let d = Unforget::<'static, &i32>::new(&i);
//!    let (tx, rx) = rendezvous();
//!
//!    let waker = noop_waker();
//!    let mut cx = task::Context::from_waker(&waker);
//!
//!    let mut rx = Box::pin(rx.into_recv_async());
//!    assert!(rx.as_mut().poll(&mut cx).is_pending());
//!
//!    tx.try_send((rx as Pin<Box<dyn Forget + '_>>, d)).unwrap();
//! }
//! ```

use std::{
    future::Future,
    pin::{self, Pin},
    task,
};

use leak_playground_std::marker::Forget;

/// Create a rendezvous channel.
pub fn rendezvous<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::bounded(0);
    (Sender { inner: tx }, Receiver { inner: rx })
}

pub struct Sender<T> {
    inner: flume::Sender<T>,
}

unsafe impl<T> Forget for Sender<T> {}

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> Result<(), flume::SendError<T>> {
        self.inner.send(msg)
    }

    pub fn try_send(&self, msg: T) -> Result<(), flume::TrySendError<T>> {
        self.inner.try_send(msg)
    }

    /// Asynchronously send an item.
    pub fn send_async(&self, item: T) -> flume::r#async::SendFut<T> {
        self.inner.send_async(item)
    }

    /// Asynchronously send an item and consume the sender.
    pub fn into_send_async<'a>(self, item: T) -> flume::r#async::SendFut<'a, T> {
        self.inner.into_send_async(item)
    }
}

pub struct Receiver<T> {
    inner: flume::Receiver<T>,
}

unsafe impl<T> Forget for Receiver<T> {}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, flume::RecvError> {
        self.inner.recv()
    }

    pub fn try_recv(&self) -> Result<T, flume::TryRecvError> {
        self.inner.try_recv()
    }

    /// Asynchronously receive an item.
    pub fn recv_async(&self) -> RecvFut<'_, T>
    where
        T: Forget,
    {
        RecvFut {
            inner: self.inner.recv_async(),
        }
    }

    /// Asynchronously receive an unforgettable item.
    ///
    /// # Safety
    ///
    /// `T` must not take ownership over itself.
    pub unsafe fn recv_async_unchecked(&self) -> RecvFut<'_, T> {
        RecvFut {
            inner: self.inner.recv_async(),
        }
    }

    /// Asynchronously receive an item and consume the receiver.
    pub fn into_recv_async<'a>(self) -> RecvFut<'a, T>
    where
        T: Forget,
    {
        RecvFut {
            inner: self.inner.into_recv_async(),
        }
    }

    /// Asynchronously receive an unforgettable item and consume the receiver.
    ///
    /// # Safety
    ///
    /// `T` must not take ownership over itself.
    pub unsafe fn into_recv_async_unchecked<'a>(self) -> RecvFut<'a, T> {
        RecvFut {
            inner: self.inner.into_recv_async(),
        }
    }
}

pub struct RecvFut<'a, T> {
    inner: flume::r#async::RecvFut<'a, T>,
}

impl<'a, T> Future for RecvFut<'a, T> {
    type Output = Result<T, flume::RecvError>;

    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }.poll(cx)
    }
}

unsafe impl<T: Forget> Forget for RecvFut<'_, T> {}
