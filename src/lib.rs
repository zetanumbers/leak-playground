#![feature(auto_traits, negative_impls, thread_spawn_unchecked)]
//! # Examples
//!
//! ## Futures
//!
//! ### `Leak` future with internal unleak logic
//!
//! **Currently emits "higher-ranked lifetime error"**
//!
//! ```
//! use leak_playground::*;
//! fn _internal_unleak_future() -> impl std::future::Future<Output = ()> + Leak {
//!     async {
//!         let num = std::hint::black_box(0);
//!         let bor = Unleak::new(&num);
//!         let () = std::future::pending().await;
//!         assert_eq!(*bor.0, 0);
//!     }
//! }
//! ```
//!
//! ### `Leak` future with `JoinGuard`s
//!
//! This is fine because of the pin's [drop
//! guarantee](https://doc.rust-lang.org/1.75.0/std/pin/index.html#drop-guarantee).
//!
//! **Currently emits "higher-ranked lifetime error"**
//!
//! ```
//! use leak_playground::*;
//! fn _internal_join_guard_future() -> impl std::future::Future<Output = ()> + Leak {
//!     async {
//!         let local = 42;
//!         let thrd = thread::spawn_scoped({
//!             let local = &local;
//!             move || {
//!                 let _inner_local = local;
//!             }
//!         });
//!         let () = std::future::pending().await;
//!         drop(thrd);
//!     }
//! }
//! ```
//!
//! ### `!Leak` future with external unleak logic
//!
//! **Currently emits "higher-ranked lifetime error", instead of something about unimplemented `Leak`**
//!
//! ```compile_fail
//! use leak_playground::*;
//! fn _external_unleak_future<'a>(num: &'a i32) -> impl std::future::Future<Output = ()> + Leak + 'a {
//!     async move {
//!         let bor = Unleak::new(num);
//!         let () = std::future::pending().await;
//!         assert_eq!(*bor.0, 0);
//!     }
//! }
//! ```
//!
//! ## JoinGuards
//!
//! ### Static JoinGuard self ownership
//!
//! ```
//! use leak_playground::*;
//! let (tx, rx) = sync::mpsc::rendezvous_channel();
//! let thrd = thread::spawn_scoped(move || {
//!     let _this_thread = rx.recv().unwrap();
//! });
//! tx.send(thrd).unwrap();
//! ```
//!
//! ### Self ownership of SendJoinGuard
//!
//! ```compile_fail
//! use leak_playground::*;
//! let local = 42;
//! let (tx, rx) = sync::mpsc::rendezvous_channel();
//! let mut f = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let thrd = thread::spawn_borrowed_scoped(&mut f);
//! tx.send(thrd).unwrap();
//! drop(tx);
//! ```
//!
//! ### Two-step self ownership
//!
//! ```compile_fail
//! use leak_playground::*;
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
//! let thrd1 = thread::spawn_borrowed_scoped(&mut f1);
//!
//! let (tx2, rx2) = sync::mpsc::rendezvous_channel();
//! let mut f2 = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let thrd2 = thread::spawn_borrowed_scoped(&mut f2);
//! tx1.send(thrd2).unwrap();
//! drop(tx1);
//! tx2.send(thrd1).unwrap();
//! drop(tx2);
//! ```
//!
//! ### Nested ownership without cycles
//!
//! ```
//! use leak_playground::*;
//! let local = 42;
//!
//! let mut f1 = || {
//!     let _inner_local = &local;
//! };
//! let thrd1 = thread::spawn_borrowed_scoped(&mut f1);
//!
//! let (tx2, rx2) = sync::mpsc::rendezvous_channel();
//! let mut f2 = {
//!     let local = &local;
//!     move || {
//!         let _this_thread = rx2.recv().unwrap();
//!         let _inner_local = local;
//!     }
//! };
//! let _thrd2 = thread::spawn_borrowed_scoped(&mut f2);
//! tx2.send(thrd1).unwrap();
//! drop(tx2);
//! ```
//!
//! ### Nested mixed ownership without cycles
//!
//! ```
//! use leak_playground::*;
//! let local = 42;
//!
//! let mut f1 = || {
//!     let _inner_local = &local;
//! };
//! let thrd1 = thread::spawn_borrowed_scoped(&mut f1);
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

pub mod mem;
pub mod rc;
pub mod sync;
pub mod thread;

/// Proposed `Leak` trait
///
/// # Safety
///
/// Implement only if you know there's absolutely no possible way to
/// leak your type.
pub unsafe auto trait Leak {}

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unleak<T>(pub T, PhantomStaticUnleak);

impl<T> std::fmt::Debug for Unleak<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unleak").field(&self.0).finish()
    }
}

impl<T> Unleak<T> {
    pub const fn new(v: T) -> Self {
        Unleak(v, PhantomStaticUnleak)
    }
}

unsafe impl<T: 'static> Leak for Unleak<T> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomStaticUnleak;

impl !Leak for PhantomStaticUnleak {}

// SAFETY: borrows don't own anything
unsafe impl<T> Leak for &T {}
unsafe impl<T> Leak for &mut T {}

// Workaround impls since we aren't inside of std

// SAFETY: it is always safe to leak JoinHandle
unsafe impl<T: 'static> Leak for std::thread::JoinHandle<T> {}
