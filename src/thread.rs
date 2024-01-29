use std::thread::JoinHandle;
use std::{marker::PhantomData, thread};

use crate::rc::Rc;
use crate::sync::Arc;
use crate::{Leak, Unleak};

/// Handle to a thread, which joins on drop.
///
/// Cannot be sent across threads, as opposed to [`SendJoinGuard`].
///
/// To spawn use [`spawn_scoped`].
pub struct JoinGuard<'a> {
    // using unit as a return value for simplicity
    thread: Option<thread::JoinHandle<()>>,
    // not sure about covariance
    _borrow: PhantomData<Unleak<&'a mut ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl Send for JoinGuard<'_> where Self: Leak {}
unsafe impl Sync for JoinGuard<'_> {}

pub fn spawn_scoped<'a, F>(f: F) -> JoinGuard<'a>
where
    F: FnOnce() + Send + 'a,
{
    JoinGuard {
        thread: Some(thread::spawn(unsafe { make_fn_once_static(f) })),
        _borrow: PhantomData,
        _unsend: PhantomData,
    }
}

pub fn spawn_borrowed_scoped<'a, F>(f: &'a mut F) -> SendJoinGuard<'a>
where
    F: FnMut() + Send + 'a,
{
    SendJoinGuard {
        inner: spawn_scoped(f),
    }
}

impl JoinGuard<'_> {
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

impl From<JoinGuard<'static>> for JoinHandle<()> {
    fn from(mut value: JoinGuard<'static>) -> Self {
        value.thread.take().unwrap()
    }
}

impl Drop for JoinGuard<'_> {
    fn drop(&mut self) {
        // Ignoring error, not propagating, fine in this situation
        let _ = self.thread.take().unwrap().join();
    }
}

/// Handle to a thread, which joins on drop. Can be sent across threads,
/// but is more awkward to use than [`JoinGuard`]. This type is returned
/// by [`spawn_borrowed_scoped`].
pub struct SendJoinGuard<'a> {
    inner: JoinGuard<'a>,
}

unsafe impl Send for SendJoinGuard<'_> {}

impl SendJoinGuard<'_> {
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

impl<'a> From<JoinGuard<'a>> for SendJoinGuard<'a>
where
    JoinGuard<'a>: Leak,
{
    fn from(inner: JoinGuard<'a>) -> Self {
        SendJoinGuard { inner }
    }
}

impl<'a> From<SendJoinGuard<'a>> for JoinGuard<'a> {
    fn from(value: SendJoinGuard<'a>) -> Self {
        value.inner
    }
}

impl From<SendJoinGuard<'static>> for JoinHandle<()> {
    fn from(value: SendJoinGuard<'static>) -> Self {
        value.inner.into()
    }
}

unsafe fn make_fn_once_static<'a, F>(f: F) -> impl FnOnce() + Send + 'static
where
    F: FnOnce() + Send + 'a,
{
    let mut f = Some(f);
    make_fn_mut_static(move || (f.take().unwrap())())
}

unsafe fn make_fn_mut_static<'a, F>(f: F) -> impl FnMut() + Send + 'static
where
    F: FnMut() + Send + 'a,
{
    let f: Box<dyn FnMut() + Send + 'a> = Box::new(f);
    let f: Box<dyn FnMut() + Send> = core::mem::transmute(f);
    f
}
