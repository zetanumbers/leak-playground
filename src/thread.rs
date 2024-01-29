use std::thread::JoinHandle;
use std::{marker::PhantomData, thread};

use crate::mem::ManuallyDrop;
use crate::rc::Rc;
use crate::sync::Arc;
use crate::{Leak, Unleak};

pub fn spawn_scoped<'a, F, T>(f: F) -> JoinGuard<'a, T>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    JoinGuard {
        // SAFETY: destruction guarantee from `Unleak<&'a ()>` and `T: 'a`
        thread: unsafe {
            ManuallyDrop::new_unchecked(thread::Builder::new().spawn_unchecked(f).unwrap())
        },
        _borrow: PhantomData,
        _unsend: PhantomData,
    }
}

pub fn spawn_borrowed_scoped<'a, F, T>(f: &'a mut F) -> SendJoinGuard<'a, T>
where
    F: FnMut() -> T + Send + 'a,
    T: Send + 'a,
{
    SendJoinGuard {
        inner: spawn_scoped(f),
    }
}

/// Handle to a thread, which joins on drop.
///
/// Cannot be sent across threads, as opposed to [`SendJoinGuard`].
///
/// To spawn use [`spawn_scoped`].
pub struct JoinGuard<'a, T> {
    // using unit as a return value for simplicity
    thread: ManuallyDrop<thread::JoinHandle<T>>,
    // not sure about covariance
    _borrow: PhantomData<Unleak<&'a ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<'a, T> Send for JoinGuard<'a, T> where Self: Leak {}
unsafe impl<'a, T> Sync for JoinGuard<'a, T> {}

impl<'a, T> JoinGuard<'a, T> {
    pub fn join(mut self) -> std::thread::Result<T> {
        unsafe { ManuallyDrop::take(&mut self.thread) }.join()
    }

    pub fn into_rc(self) -> Rc<Self> {
        // SAFETY: we cannot move Rc<JoinGuard> into it's closure because
        //   impl !Send for Rc<JoinGuard>
        unsafe { Rc::new_unchecked(self) }
    }

    pub fn into_arc(self) -> Arc<Self> {
        // SAFETY: we cannot move Arc<JoinGuard> into it's closure because
        //   Arc<JoinGuard>: !Send, or otherwise JoinGuard: Leak
        unsafe { Arc::new_unchecked(self) }
    }
}

impl<T> From<JoinGuard<'static, T>> for JoinHandle<T> {
    fn from(mut value: JoinGuard<'static, T>) -> Self {
        unsafe { ManuallyDrop::take(&mut value.thread) }
    }
}

impl<'a, T> Drop for JoinGuard<'a, T> {
    fn drop(&mut self) {
        // Ignoring error, not propagating, fine in this situation
        let _ = unsafe { ManuallyDrop::take(&mut self.thread) }.join();
    }
}

/// Handle to a thread, which joins on drop. Implements [`Send`].
///
/// Can be sent across threads, but is more awkward to use than
/// [`JoinGuard`].
///
/// To spawn use [`spawn_borrowed_scoped`].
pub struct SendJoinGuard<'a, T> {
    inner: JoinGuard<'a, T>,
}

unsafe impl<'a, T> Send for SendJoinGuard<'a, T> {}

impl<'a, T> SendJoinGuard<'a, T> {
    pub fn join(self) -> std::thread::Result<T> {
        self.inner.join()
    }

    pub fn into_rc(self) -> Rc<Self> {
        // SAFETY: we cannot move Rc<SendJoinGuard> into it's
        //   closure because impl !Send for Rc<SendJoinGuard>
        unsafe { Rc::new_unchecked(self) }
    }

    pub fn into_arc(self) -> Arc<Self> {
        // SAFETY: we cannot move Arc<SendJoinGuard> into it's
        //   closure SendJoinGuard is bounded by a borrow of the
        //   same closure thus moving guard into the closure would
        //   introduce self-referential type which is prohibited
        unsafe { Arc::new_unchecked(self) }
    }
}

impl<'a, T> From<JoinGuard<'a, T>> for SendJoinGuard<'a, T>
where
    JoinGuard<'a, T>: Leak,
{
    fn from(inner: JoinGuard<'a, T>) -> Self {
        SendJoinGuard { inner }
    }
}

impl<'a, T> From<SendJoinGuard<'a, T>> for JoinGuard<'a, T> {
    fn from(value: SendJoinGuard<'a, T>) -> Self {
        value.inner
    }
}

impl<T> From<SendJoinGuard<'static, T>> for JoinHandle<T> {
    fn from(value: SendJoinGuard<'static, T>) -> Self {
        value.inner.into()
    }
}
