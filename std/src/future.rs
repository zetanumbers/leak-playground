//! # Examples
//!
//! `Forget` future with internal unforget logic.
//!
//! **Currently emits "higher-ranked lifetime error", instead of compiling**
//!
//! ```ignore
//! use leak_playground_std::marker::{Forget, Unforget};
//! fn _internal_unforget_future() -> impl std::future::Future<Output = ()> + Forget {
//!     async {
//!         let num = std::hint::black_box(0);
//!         let bor = Unforget::new_static(&num);
//!         let () = std::future::pending().await;
//!         assert_eq!(**bor, 0);
//!     }
//! }
//! ```
//!
//! `Forget` future with `JoinGuard`s. This is fine because of the pin's [drop
//! guarantee](https://doc.rust-lang.org/1.75.0/std/pin/index.html#drop-guarantee).
//!
//! **Currently emits "higher-ranked lifetime error", instead of compiling**
//!
//! ```ignore
//! use leak_playground_std::thread;
//! use leak_playground_std::marker::Forget;
//! fn _internal_join_guard_future() -> impl std::future::Future<Output = ()> + Forget {
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
//! `!Forget` future with external unforget logic
//!
//! **Currently emits "higher-ranked lifetime error", instead of something about unimplemented `Forget`**
//!
//! ```compile_fail
//! use leak_playground_std::marker::{Forget, Unforget};
//! fn _external_unforget_future<'a>(num: &'a i32) -> impl std::future::Future<Output = ()> + Forget + 'a {
//!     async move {
//!         let bor = Unforget::new_static(num);
//!         let () = std::future::pending().await;
//!         assert_eq!(**bor, 0);
//!     }
//! }
//! ```
//!
