use std::{future::Future, marker::PhantomData, mem, pin::Pin, ptr::NonNull};

use leak_playground::{mem::ManuallyDrop, Leak, Unleak};
use tokio::task::{JoinError, JoinHandle};

pub fn spawn_scoped<'a, F>(future: F) -> ScopedJoinHandle<'a, F::Output>
where
    F: Future + Send + 'a,
    F::Output: Send + 'a,
{
    ScopedJoinHandle {
        inner: unsafe {
            ManuallyDrop::new_unchecked(tokio::task::spawn(erased_send_future(future)))
        },
        _unleak: PhantomData,
        _unsend: PhantomData,
    }
}

pub fn spawn_borrowed<'a, F>(future: Pin<&'a mut F>) -> ScopedSendJoinHandle<'a, F::Output>
where
    F: Future + Send + 'a,
    F::Output: Send + 'a,
{
    ScopedSendJoinHandle {
        inner: spawn_scoped(future),
    }
}

pub fn spawn_local_scoped<'a, F>(future: F) -> ScopedJoinHandle<'a, F::Output>
where
    F: Future + 'a,
    F::Output: 'a,
{
    ScopedJoinHandle {
        inner: unsafe {
            ManuallyDrop::new_unchecked(tokio::task::spawn_local(erased_future(future)))
        },
        _unleak: PhantomData,
        _unsend: PhantomData,
    }
}

pub struct ScopedJoinHandle<'a, T> {
    inner: ManuallyDrop<JoinHandle<Payload>>,
    _unleak: PhantomData<Unleak<(&'a (), T)>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<T: Send> Send for ScopedJoinHandle<'_, T> where Self: Leak {}
unsafe impl<T: Send> Sync for ScopedJoinHandle<'_, T> {}
impl<T> Unpin for ScopedJoinHandle<'_, T> {}

impl<'a, T> Future for ScopedJoinHandle<'a, T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        JoinHandle::poll(Pin::new(&mut self.inner), cx)
            .map(|r| r.map(|r| unsafe { r.get_unchecked::<T>() }))
    }
}

impl<'a, T> ScopedJoinHandle<'a, T> {
    pub async fn cancel(mut self) -> Result<(), JoinError> {
        self.inner.abort();
        let task = unsafe { ManuallyDrop::take(&mut self.inner) };
        match task.await {
            Err(e) if e.is_cancelled() => Ok(()),
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T> Drop for ScopedJoinHandle<'a, T> {
    fn drop(&mut self) {
        self.inner.abort();
        let task = unsafe { ManuallyDrop::take(&mut self.inner) };
        // TODO: this is hack-around without async drop
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                match task.await {
                    Err(e) if e.is_cancelled() => (),
                    Ok(_) => (),
                    Err(e) => std::panic::resume_unwind(e.into_panic()),
                }
            })
        });
    }
}

pub struct ScopedSendJoinHandle<'a, T> {
    inner: ScopedJoinHandle<'a, T>,
}

// SAFETY: we use this for borrowed futures
unsafe impl<'a, T: Send> Send for ScopedSendJoinHandle<'a, T> {}

impl<'a, T> Future for ScopedSendJoinHandle<'a, T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<'a, T> ScopedSendJoinHandle<'a, T> {
    pub async fn cancel(self) -> Result<(), JoinError> {
        self.inner.cancel().await
    }
}

// # Hack-around utilities

unsafe fn erased_send_future<F>(f: F) -> impl Future<Output = Payload> + Send + 'static
where
    F: Future + Send,
{
    let f = async move { Payload::new_unchecked(f.await) };
    let f: Pin<Box<dyn Future<Output = Payload> + Send + '_>> = Box::pin(f);
    let f: Pin<Box<dyn Future<Output = Payload> + Send>> = mem::transmute(f);
    f
}

unsafe fn erased_future<F>(f: F) -> impl Future<Output = Payload> + 'static
where
    F: Future,
{
    let f = async move { Payload::new_unchecked(f.await) };
    let f: Pin<Box<dyn Future<Output = Payload> + '_>> = Box::pin(f);
    let f: Pin<Box<dyn Future<Output = Payload>>> = mem::transmute(f);
    f
}

struct Payload {
    ptr: NonNull<()>,
}

unsafe impl Send for Payload {}
unsafe impl Sync for Payload {}

impl Payload {
    unsafe fn new_unchecked<T>(v: T) -> Payload {
        Payload {
            ptr: NonNull::new_unchecked(Box::into_raw(Box::new(v)).cast()),
        }
    }

    unsafe fn get_unchecked<T>(self) -> T {
        *Box::from_raw(self.ptr.cast().as_ptr())
    }
}
