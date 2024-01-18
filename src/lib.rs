#![feature(auto_traits, negative_impls)]
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
//! This is fine assuming that pinned future is never
//! able to be deallocated without a drop (i.e. forgotten). See
//! [rust-lang/rust#79327](https://github.com/rust-lang/rust/pull/79327).
//!
//! **Currently emits an error about unimplemented `Leak`**
//!
//! ```
//! use leak_playground::*;
//! fn _internal_join_guard_future() -> impl std::future::Future<Output = ()> + Leak {
//!     async {
//!         let local = 42;
//!         let thrd = JoinGuard::spawn({
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
//! let (tx, rx) = rendezvous_channel();
//! let mut scope = JoinScope::new(move || {
//!     let _this_thread = rx.recv().unwrap();
//! });
//! let thrd = scope.spawn_outside().into_static_scoped();
//! tx.send(thrd).unwrap();
//! ```
//!
//! ### Non-static JoinGuard uses static methods
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
//! let thrd = scope.spawn_outside().into_static_scoped();
//! tx.send(thrd).unwrap();
//! ```
//!
//! ### Self ownership of JoinGuardScoped
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
//! ### Two-step self ownership
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
//! ### Nested ownership without cycles
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
//! ### Nested mixed ownership without cycles
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

mod join_guard;

pub use join_guard::*;

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
