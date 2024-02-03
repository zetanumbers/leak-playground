//! Possible [`core::marker`] additions. Contains the proposed [`Leak`] trait.

use std::{fmt::Debug, future::Future, pin::Pin, task};

/// The core trait of the destruction guarantee.
///
/// # Safety
///
/// Implement only if you know there's absolutely no possible way to
/// leak your type.
pub unsafe auto trait Leak {}

/// A transparent wrapper to make your types `!Leak`
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unleak<T: ?Sized> {
    _unleak: PhantomStaticUnleak,
    inner: T,
}

impl<T: ?Sized + Debug> Debug for Unleak<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unleak").field(&&self.inner).finish()
    }
}

impl<T> Unleak<T> {
    pub const fn new(inner: T) -> Self {
        Unleak {
            _unleak: PhantomStaticUnleak,
            inner,
        }
    }

    pub fn into_inner(slot: Unleak<T>) -> T {
        slot.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for Unleak<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for Unleak<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> Future for Unleak<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner).poll(cx) }
    }
}

unsafe impl<T: ?Sized + 'static> Leak for Unleak<T> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomStaticUnleak;

impl !Leak for PhantomStaticUnleak {}

// SAFETY: borrows don't own anything
unsafe impl<T: ?Sized> Leak for &T {}
unsafe impl<T: ?Sized> Leak for &mut T {}

// Workaround impls since we aren't inside of std

// SAFETY: it is always safe to leak JoinHandle
unsafe impl<T: 'static> Leak for std::thread::JoinHandle<T> {}

#[cfg(feature = "tokio_rt")]
mod tokio_rt {
    unsafe impl<T: 'static> super::Leak for tokio::task::JoinHandle<T> {}
}
