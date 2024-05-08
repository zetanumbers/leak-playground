//! Possible [`std::thread`] additions. Contains examples.
//!
//! # Examples
//!
//! Static JoinGuard self ownership
//!
//! ```
//! use leak_playground_std::*;
//! let (tx, rx) = sync::mpsc::rendezvous_channel();
//! let thrd = thread::spawn_scoped(move || {
//!     let _this_thread = rx.recv().unwrap();
//! });
//! tx.send(thrd).unwrap();
//! ```
//!
//! Self ownership of SendJoinGuard
//!
//! ```compile_fail
//! use leak_playground_std::*;
//! let local = 42;
//! let (tx, rx) = sync::mpsc::rendezvous_channel();
//! let mut f = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let thrd = thread::spawn_borrowed(&mut f);
//! tx.send(thrd).unwrap();
//! drop(tx);
//! ```
//!
//! Two-step self ownership
//!
//! ```compile_fail
//! use leak_playground_std::*;
//! let local = 42;
//!
//! let (tx1, rx1) = sync::mpsc::rendezvous_channel();
//! let mut f1 = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx1.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let thrd1 = thread::spawn_borrowed(&mut f1);
//!
//! let (tx2, rx2) = sync::mpsc::rendezvous_channel();
//! let mut f2 = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let thrd2 = thread::spawn_borrowed(&mut f2);
//! tx1.send(thrd2).unwrap();
//! drop(tx1);
//! tx2.send(thrd1).unwrap();
//! drop(tx2);
//! ```
//!
//! Nested ownership without cycles
//!
//! ```
//! use leak_playground_std::*;
//! let local = 42;
//!
//! let mut f1 = || {
//!     let _inner_local = &local;
//! };
//! let thrd1 = thread::spawn_borrowed(&mut f1);
//!
//! let (tx2, rx2) = sync::mpsc::rendezvous_channel();
//! let mut f2 = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let _thrd2 = thread::spawn_borrowed(&mut f2);
//! tx2.send(thrd1).unwrap();
//! drop(tx2);
//! ```
//!
//! Nested mixed ownership without cycles
//!
//! ```
//! use leak_playground_std::*;
//! let local = 42;
//!
//! let mut f1 = || {
//!     let _inner_local = &local;
//! };
//! let thrd1 = thread::spawn_borrowed(&mut f1);
//!
//! let (tx2, rx2) = sync::mpsc::rendezvous_channel();
//! let _thrd2 = thread::spawn_scoped({
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! });
//! tx2.send(thrd1).unwrap();
//! ```

use std::thread::JoinHandle;
use std::{marker::PhantomData, thread};

use crate::marker::{Leak, Unleak};
use crate::mem::{self, ManuallyDrop};
use crate::rc::Rc;
use crate::sync::Arc;

/// Spawn `?Send` thread handles.
pub fn spawn_scoped<'a, F, T>(f: F) -> JoinGuard<'a, T>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    JoinGuard {
        // SAFETY: destruction guarantee from `Unleak<&'a ()>` and `T: 'a`
        child: unsafe {
            ManuallyDrop::new_unchecked(thread::Builder::new().spawn_unchecked(f).unwrap())
        },
        _borrow: PhantomData,
        _unsend: PhantomData,
    }
}

/// Spawn `Send` thread handles.
pub fn spawn_borrowed<'a, F, T>(f: &'a mut F) -> SendJoinGuard<'a, T>
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
/// Cannot be sent across threads, as opposed to [`SendJoinGuard`]. This
/// is made to ensure we won't put this into itself, thus leaking it.
///
/// To spawn use [`spawn_scoped`].
pub struct JoinGuard<'a, T> {
    child: ManuallyDrop<thread::JoinHandle<T>>,

    /// Not sure about covariance there.
    _borrow: PhantomData<Unleak<'static, &'a ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<T> Send for JoinGuard<'_, T> where Self: Leak {}
unsafe impl<T> Sync for JoinGuard<'_, T> {}

impl<T> JoinGuard<'_, T> {
    pub fn join(mut self) -> std::thread::Result<T> {
        let join_handle;
        // SAFETY: we immediately, join after
        unsafe {
            join_handle = ManuallyDrop::take(&mut self.child);
            // need this to avoid calling `JoinGuard::drop`
            mem::forget_unchecked(self);
        }
        join_handle.join()
    }

    pub fn thread(&self) -> &std::thread::Thread {
        self.child.thread()
    }

    pub fn is_finished(&self) -> bool {
        self.child.is_finished()
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

impl<T> JoinGuard<'static, T> {
    pub fn into_handle(self) -> JoinHandle<T> {
        self.into()
    }

    pub fn detach(self) {
        let _ = self.into_handle();
    }
}

impl<T> From<JoinGuard<'static, T>> for JoinHandle<T> {
    fn from(mut value: JoinGuard<'static, T>) -> Self {
        unsafe { ManuallyDrop::take(&mut value.child) }
    }
}

impl<'a, T> Drop for JoinGuard<'a, T> {
    fn drop(&mut self) {
        let join_handle = unsafe { ManuallyDrop::take(&mut self.child) };
        // Shouldn't panic
        let child = join_handle.thread().clone();
        // No panic since we guarantee that we would never join on ourselves,
        // except when `Self: Leak`, then we don't care.
        let res = join_handle.join();
        // Propagating panic there since structured parallelism, but ignoring
        // during panic. Anyway child thread is joined thus either would
        // be fine.
        if res.is_err() && !std::thread::panicking() {
            panic!("child thread {child:?} panicked");
        }
    }
}

/// Handle to a thread, which joins on drop. Implements `Send`.
///
/// Can be sent across threads, but is more awkward to use than
/// [`JoinGuard`].
///
/// To spawn use [`spawn_borrowed`].
pub struct SendJoinGuard<'a, T> {
    inner: JoinGuard<'a, T>,
}

unsafe impl<'a, T> Send for SendJoinGuard<'a, T> {}

impl<'a, T> SendJoinGuard<'a, T> {
    pub fn join(self) -> std::thread::Result<T> {
        self.inner.join()
    }

    pub fn thread(&self) -> &std::thread::Thread {
        self.inner.thread()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
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

impl<T> SendJoinGuard<'static, T> {
    pub fn into_handle(self) -> JoinHandle<T> {
        self.into()
    }

    pub fn detach(self) {
        let _ = self.into_handle();
    }
}

impl<T> From<JoinGuard<'static, T>> for SendJoinGuard<'static, T> {
    fn from(inner: JoinGuard<'static, T>) -> Self {
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
