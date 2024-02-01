//! Possible [`tokio::task`] additions.

use std::{future::Future, marker::PhantomData, mem, pin::Pin, ptr::NonNull};

use leak_playground_std::marker::{Leak, Unleak};
use leak_playground_std::mem::ManuallyDrop;
use tokio::task::{JoinError, JoinHandle};

/// Spawns a non-static `Send` future, returning a `!Send` for non-static
/// cases handle for it.
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
        _output: PhantomData,
    }
}

/// Spawns a non-static `Send` future, returning a `Send` handle for it.
pub fn spawn_borrowed<'a, F>(future: Pin<&'a mut F>) -> ScopedSendJoinHandle<'a, F::Output>
where
    F: Future + Send + 'a,
    F::Output: Send + 'a,
{
    ScopedSendJoinHandle {
        inner: spawn_scoped(future),
    }
}

/// Spawns a non-static `!Send` future.
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
        _output: PhantomData,
    }
}

/// Runs the provided non-static closure on a thread where blocking is
/// acceptable. Returns a `?Send` handle for it.
pub fn spawn_blocking_scoped<'a, F, T>(f: F) -> ScopedJoinHandle<'a, T>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    ScopedJoinHandle {
        inner: unsafe {
            ManuallyDrop::new_unchecked(tokio::task::spawn_blocking(erased_send_fn_once(f)))
        },
        _unleak: PhantomData,
        _unsend: PhantomData,
        _output: PhantomData,
    }
}

/// Runs the provided non-static closure on a thread where blocking is
/// acceptable. Returns a `Send` handle for it.
pub fn spawn_blocking_borrowed<'a, F, T>(f: &'a mut F) -> ScopedSendJoinHandle<'a, T>
where
    F: FnMut() -> T + Send + 'a,
    T: Send + 'a,
{
    ScopedSendJoinHandle {
        inner: spawn_blocking_scoped(f),
    }
}

/// Handle to a task, which cancels on drop.
///
/// Cannot be sent across threads, as opposed to
/// [`ScopedSendJoinHandle`]. This is made to ensure we won't put this
/// into itself, thus leaking it.
///
/// To spawn use [`spawn_scoped`], [`spawn_local_scoped`], or [`spawn_blocking_scoped`].
pub struct ScopedJoinHandle<'a, T> {
    inner: ManuallyDrop<JoinHandle<Payload>>,
    _unleak: PhantomData<Unleak<&'a ()>>,
    // No need for Unleak since we put bound `T: 'a` on constructors
    _output: PhantomData<fn() -> T>,
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

/// Handle to a task, which cancels on drop. Implements `Send`.
///
/// Can be sent across threads, but is more awkward to use than
/// [`JoinGuard`].
///
/// To spawn use [`spawn_borrowed`], [`spawn_blocking_borrowed`].
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

unsafe fn erased_send_fn_once<F, R>(f: F) -> impl FnOnce() -> Payload + Send + 'static
where
    F: FnOnce() -> R + Send,
{
    let f = move || Payload::new_unchecked(f());
    let f: Box<dyn FnOnce() -> Payload + Send + '_> = Box::new(f);
    let f: Box<dyn FnOnce() -> Payload + Send> = mem::transmute(f);
    f
}

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
