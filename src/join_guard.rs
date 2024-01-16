use std::marker::PhantomData;
use std::sync::mpsc;
use std::{mem, thread};

use crate::{Leak, PhantomUnleak};

/// Handle to a thread, which joins on drop.
/// Cannot be sent across threads, as opposed to [`JoinGuardScoped`].
pub struct JoinGuard<'a> {
    // using unit as a return value for simplicity
    thread: Option<thread::JoinHandle<()>>,
    _invariant: PhantomData<&'a mut &'a ()>,
    _unsend: PhantomData<*mut ()>,
    _unleak: PhantomUnleak<'a>,
}

unsafe impl Send for JoinGuard<'_> where Self: Leak {}
unsafe impl Sync for JoinGuard<'_> {}

impl<'a> JoinGuard<'a> {
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'a,
    {
        JoinGuard {
            thread: Some(thread::spawn(unsafe { make_fn_once_static(f) })),
            _invariant: PhantomData,
            _unsend: PhantomData,
            _unleak: PhantomUnleak::new(),
        }
    }
}

impl JoinGuard<'static> {
    pub fn into_static_scoped(self) -> JoinGuardScoped<'static> {
        JoinGuardScoped { _inner: self }
    }
}

impl Drop for JoinGuard<'_> {
    fn drop(&mut self) {
        // Ignoring error, not propating, fine in this situation
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
        JoinGuard::spawn(self.f.take().expect("Second spawn"))
    }
}

/// Handle to a thread, which joins on drop.
/// Can be sent across threads, but is more awkward to use than [`JoinGuard`].
/// This type is returned by [`JoinScope::spawn`].
pub struct JoinGuardScoped<'a> {
    _inner: JoinGuard<'a>,
}

unsafe impl Send for JoinGuardScoped<'_> {}

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
    let f: Box<dyn FnMut() + Send> = mem::transmute(f);
    f
}

pub fn rendezvous_channel<T>() -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
    mpsc::sync_channel(0)
}
