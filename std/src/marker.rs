//! Possible [`core::marker`] additions. Contains the proposed [`Leak`] trait.

use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task};

use crate::anchor::Anchored;

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
pub struct Unleak<'a, T: ?Sized> {
    _unleak: PhantomStaticUnleak,
    /// Inner value must be able to outlive this lifetime to be able to
    /// be forgotten. Of course it is contravariant, since expanding it
    /// may only disable the [`Leak`] implementation.
    _anchor: PhantomData<fn(&'a ())>,
    inner: T,
}

impl<T: ?Sized + Debug> Debug for Unleak<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unleak").field(&&self.inner).finish()
    }
}

impl<T> Unleak<'static, T> {
    pub const fn new_static(inner: T) -> Self {
        Unleak {
            _unleak: PhantomStaticUnleak,
            _anchor: PhantomData,
            inner,
        }
    }
}

impl<'a, T> Unleak<'a, T> {
    pub fn with_anchored<U>(inner: T, _anchored: &Anchored<'a, U>) -> Self {
        Unleak {
            _unleak: PhantomStaticUnleak,
            _anchor: PhantomData,
            inner,
        }
    }

    pub fn with_arbitrary_lifetime(inner: T) -> Self {
        Unleak {
            _unleak: PhantomStaticUnleak,
            _anchor: PhantomData,
            inner,
        }
    }
}

// TODO: Consider simpifying
impl<T> Unleak<'_, T> {
    /// Get inner value.
    pub fn into_inner(slot: Self) -> T
    where
        Self: Leak,
    {
        slot.inner
    }

    /// Get inner value.
    ///
    /// # Safety
    ///
    /// Make sure T is dropped.
    pub unsafe fn into_inner_unchecked(slot: Self) -> T {
        slot.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for Unleak<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for Unleak<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> Future for Unleak<'_, T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner).poll(cx) }
    }
}

unsafe impl<'a, T: ?Sized + 'a> Leak for Unleak<'a, T> {}

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
#[doc(hidden)] // Nothing to document
mod tokio_rt {
    unsafe impl<T: 'static> super::Leak for tokio::task::JoinHandle<T> {}
}
