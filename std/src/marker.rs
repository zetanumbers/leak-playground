//! Possible [`core::marker`] additions. Contains the proposed [`Forget`] trait.

use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task};

/// The core trait of the destruction guarantee.
///
/// # Safety
///
/// Implement only if you know there's absolutely no possible way to
/// forget your type.
pub unsafe auto trait Forget {}

#[doc(inline)]
pub use Forget as Leak;

/// A transparent wrapper to make your types `!Forget`
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unforget<'a, T: ?Sized> {
    _unforget: PhantomStaticUnforget,
    /// Inner value must be able to outlive this lifetime to be able to
    /// be forgotten. Of course it is contravariant, since expanding it
    /// may only disable the [`Forget`] implementation.
    _anchor: PhantomData<fn(&'a ())>,
    inner: T,
}

impl<T: ?Sized + Debug> Debug for Unforget<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unforget").field(&&self.inner).finish()
    }
}

#[doc(inline)]
pub use Unforget as Unleak;

impl<T> Unforget<'static, T> {
    pub const fn new(inner: T) -> Self {
        Unforget {
            _unforget: PhantomStaticUnforget,
            _anchor: PhantomData,
            inner,
        }
    }
}

impl<'a, T> Unforget<'a, T> {
    pub fn with_lifetime(inner: T) -> Self {
        Unforget {
            _unforget: PhantomStaticUnforget,
            _anchor: PhantomData,
            inner,
        }
    }

    /// Get inner value.
    pub fn into_inner(slot: Self) -> T {
        slot.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for Unforget<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for Unforget<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> Future for Unforget<'_, T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner).poll(cx) }
    }
}

unsafe impl<'a, T: ?Sized + 'a> Forget for Unforget<'a, T> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomStaticUnforget;

impl !Forget for PhantomStaticUnforget {}

// SAFETY: borrows don't own anything
unsafe impl<T: ?Sized> Forget for &T {}
unsafe impl<T: ?Sized> Forget for &mut T {}

// Workaround impls since we aren't inside of std

// SAFETY: it is always safe to forget JoinHandle
unsafe impl<T: 'static> Forget for std::thread::JoinHandle<T> {}

#[cfg(feature = "tokio_rt")]
#[doc(hidden)] // Nothing to document
mod tokio_rt {
    unsafe impl<T: 'static> super::Forget for tokio::task::JoinHandle<T> {}
}
