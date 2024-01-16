#![feature(auto_traits, negative_impls)]
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
//! ### Unstatic JoinGuard uses static methods
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

use std::marker::PhantomData;

mod join_guard;

pub use join_guard::*;

/// Proposed `Leak` trait
///
/// # Safety
///
/// Implement only if you know there's absolutelly no possible way to
/// leak your type.
pub unsafe auto trait Leak {}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhantomUnleak<'a>(PhantomData<&'a ()>, PhantomStaticUnleak);

impl<'a> std::fmt::Debug for PhantomUnleak<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "PhantomUnleak".fmt(f)
    }
}

impl<'a> PhantomUnleak<'a> {
    pub const fn new() -> Self {
        PhantomUnleak(PhantomData, PhantomStaticUnleak)
    }
}

unsafe impl Leak for PhantomUnleak<'static> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomStaticUnleak;

impl !Leak for PhantomStaticUnleak {}
