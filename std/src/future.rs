//! # Examples
//!
//! `Leak` future with internal unleak logic.
//!
//! **Currently emits "higher-ranked lifetime error"**
//!
//! ```
//! use leak_playground_std::marker::{Leak, Unleak};
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
//! `Leak` future with `JoinGuard`s. This is fine because of the pin's [drop
//! guarantee](https://doc.rust-lang.org/1.75.0/std/pin/index.html#drop-guarantee).
//!
//! **Currently emits "higher-ranked lifetime error"**
//!
//! ```
//! use leak_playground_std::thread;
//! use leak_playground_std::marker::{Leak, Unleak};
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
//! `!Leak` future with external unleak logic
//!
//! **Currently emits "higher-ranked lifetime error", instead of something about unimplemented `Leak`**
//!
//! ```compile_fail
//! use leak_playground_std::marker::{Leak, Unleak};
//! fn _external_unleak_future<'a>(num: &'a i32) -> impl std::future::Future<Output = ()> + Leak + 'a {
//!     async move {
//!         let bor = Unleak::new(num);
//!         let () = std::future::pending().await;
//!         assert_eq!(*bor.0, 0);
//!     }
//! }
//! ```
//!
