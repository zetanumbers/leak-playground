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

use std::thread::JoinHandle;
use std::{marker::PhantomData, thread};

use crate::marker::{Forget, Unforget};
use crate::mem::{self, ManuallyDrop};
use crate::rc::Rc;
use crate::sync::Arc;

/// Spawn borrowing thread handles.
pub fn spawn_scoped<'a, F, T>(f: F) -> JoinGuard<'a, T>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    JoinGuard {
        // SAFETY: destruction guarantee from `Unforget<&'a ()>` and `T: 'a`
        child: unsafe {
            ManuallyDrop::new_unchecked(thread::Builder::new().spawn_unchecked(f).unwrap())
        },
        _borrow: Unforget::new(PhantomData),
        _unsend: PhantomData,
    }
}

/// Handle to a thread, which joins on drop.
///
/// Cannot be sent across threads.
/// This is made to ensure we won't put this into itself, thus forgetting it.
///
/// To spawn use [`spawn_scoped`].
pub struct JoinGuard<'a, T> {
    child: ManuallyDrop<thread::JoinHandle<T>>,

    /// Not sure about covariance there.
    _borrow: Unforget<'static, PhantomData<&'a ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<T> Send for JoinGuard<'_, T> where Self: Forget {}
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
        //   Arc<JoinGuard>: !Send, or otherwise JoinGuard: Forget
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
        // except when `Self: Forget`, then we don't care.
        let res = join_handle.join();
        // Propagating panic there since structured parallelism, but ignoring
        // during panic. Anyway child thread is joined thus either would
        // be fine.
        if res.is_err() && !std::thread::panicking() {
            panic!("child thread {child:?} panicked");
        }
    }
}
