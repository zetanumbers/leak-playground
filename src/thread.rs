use std::thread::JoinHandle;
use std::{marker::PhantomData, thread};

use crate::rc::Rc;
use crate::sync::Arc;
use crate::{mem, Leak, Unleak};

/// Handle to a thread, which joins on drop.
///
/// Cannot be sent across threads, as opposed to [`JoinGuardScoped`].
///
/// To spawn use [`spawn_scoped`].
pub struct JoinGuard<'a> {
    // using unit as a return value for simplicity
    thread: Option<thread::JoinHandle<()>>,
    // not sure about invariance
    _borrow: PhantomData<Unleak<&'a mut &'a ()>>,
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

impl JoinGuard<'_> {
    pub fn into_rc(self) -> Rc<Self> {
        // SAFETY: we cannot move Rc<JoinGuard> into it's closure because
        //   impl !Send for Rc<JoinGuard>
        unsafe { Rc::new_unchecked(self) }
    }
}

impl JoinGuard<'static> {
    pub fn into_static_scoped(self) -> JoinGuardScoped<'static> {
        JoinGuardScoped { _inner: self }
    }

    pub fn into_join_handle(mut self) -> JoinHandle<()> {
        let out = self.thread.take().unwrap();
        unsafe { mem::forget_unchecked(self) };
        out
    }
}

impl Drop for JoinGuard<'_> {
    fn drop(&mut self) {
        // Ignoring error, not propagating, fine in this situation
        let _ = self.thread.take().unwrap().join();
    }
}

pub struct JoinScope<F> {
    f: Option<F>,
}

impl<F> JoinScope<F> {
    pub fn new(f: F) -> Self {
        JoinScope { f: Some(f) }
    }
}

impl<F> JoinScope<F>
where
    F: FnOnce() + Send,
{
    #[track_caller]
    pub fn spawn(&mut self) -> JoinGuardScoped<'_> {
        JoinGuardScoped {
            _inner: self.spawn_outside(),
        }
    }

    #[track_caller]
    pub fn spawn_outside<'a>(&mut self) -> JoinGuard<'a>
    where
        F: 'a,
    {
        spawn_scoped(self.f.take().expect("Second spawn"))
    }
}

/// Handle to a thread, which joins on drop. Can be sent across threads,
/// but is more awkward to use than [`JoinGuard`]. This type is returned
/// by [`JoinScope::spawn`].
pub struct JoinGuardScoped<'a> {
    _inner: JoinGuard<'a>,
}

unsafe impl Send for JoinGuardScoped<'_> {}

impl JoinGuardScoped<'_> {
    pub fn into_rc(self) -> Rc<Self> {
        // SAFETY: we cannot move Rc<JoinGuardScoped> into it's
        //   closure because impl !Send for Rc<JoinGuardScoped>
        unsafe { Rc::new_unchecked(self) }
    }

    pub fn into_arc(self) -> Arc<Self> {
        // SAFETY: we cannot move Arc<JoinGuardScoped> into it's
        //   closure JoinGuardScoped is bounded by a borrow of the
        //   same closure thus moving guard into the closure would
        //   introduce self-referential type which are prohibited
        unsafe { Arc::new_unchecked(self) }
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
