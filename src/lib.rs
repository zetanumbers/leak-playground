//! ```
//! use leak_playground::*;
//! let (tx, rx) = rendezvous_channel();
//! let mut scope = JoinScope::new(move || {
//!     let _this_thread = rx.recv().unwrap();
//! });
//! let thrd = scope.spawn_static();
//! tx.send(thrd).unwrap();
//! ```
//!
//! ```compile_fail
//! use leak_playground::*;
//! let local = 42;
//! let (tx, rx) = rendezvous_channel();
//! let mut scope = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx.recv().unwrap();
//!         let inner_local = local;
//!     }
//! });
//! let thrd = scope.spawn_static();
//! tx.send(thrd).unwrap();
//! ```
//!
//! ```compile_fail
//! use leak_playground::*;
//! let local = 42;
//! let (tx, rx) = rendezvous_channel();
//! let mut scope = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! let thrd = scope.spawn();
//! tx.send(thrd).unwrap();
//! ```
//!
//! ```compile_fail
//! use leak_playground::*;
//! let local = 42;
//!
//! let (tx1, rx1) = rendezvous_channel();
//! let mut scope1 = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx1.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! let thrd1 = scope1.spawn();
//!
//! let (tx2, rx2) = rendezvous_channel();
//! let mut scope2 = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! let thrd2 = scope2.spawn();
//! tx1.send(thrd2).unwrap();
//! tx2.send(thrd1).unwrap();
//! ```
//!
//! ```
//! use leak_playground::*;
//! let local = 42;
//!
//! let mut scope1 = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _inner_local = local;
//!     }
//! });
//! let thrd1 = scope1.spawn();
//!
//! let (tx2, rx2) = rendezvous_channel();
//! let mut scope2 = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! let _thrd2 = scope2.spawn();
//! tx2.send(thrd1).unwrap();
//! ```
//!
//! ```
//! use leak_playground::*;
//! let local = 42;
//!
//! let mut scope1 = JoinScope::new({
//!     let local = &local;
//!     move || {
//!         let _inner_local = local;
//!     }
//! });
//! let thrd1 = scope1.spawn();
//!
//! let (tx2, rx2) = rendezvous_channel();
//! let _thrd2 = JoinGuard::spawn({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! tx2.send(thrd1).unwrap();
//! ```

use std::marker::PhantomData;
use std::sync::mpsc;
use std::{mem, thread};

pub struct JoinGuard<'a> {
    // using unit as a return value for simplicity
    thread: Option<thread::JoinHandle<()>>,
    _invariant: PhantomData<&'a mut &'a ()>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl Send for JoinGuard<'static> {}
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
        }
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
    pub fn spawn<'a>(&'a mut self) -> JoinGuardScoped<'a> {
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

    #[track_caller]
    pub fn spawn_static(&mut self) -> JoinGuardScoped<'static>
    where
        F: 'static,
    {
        JoinGuardScoped {
            _inner: self.spawn_outside(),
        }
    }
}

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
