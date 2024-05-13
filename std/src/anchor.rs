//! A new module aimed to relax unforgetness in specific cases

use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task};

#[derive(Clone, Copy, Debug, Default)]
pub struct Anchor;

impl Anchor {
    #[inline(always)]
    pub fn new() -> Self {
        Anchor
    }

    /// To reanchor a value is to restrict its lifetime further
    pub fn reanchor<'new, 'old: 'new, T>(
        &'new self,
        value: Anchored<'old, T>,
    ) -> Anchored<'new, T> {
        Anchored {
            inner: value.inner,
            _anchor: PhantomData,
        }
    }
}

/// A transparent wrapper type to use with [`crate::marker::Unleak`]. To anchor
/// a value is to restrict its lifetime.
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Anchored<'a, T: ?Sized> {
    /// Invariant lifetime, shortening only allowed via [`Anchor::reanchor`]
    _anchor: PhantomData<fn(&'a Anchor) -> &'a Anchor>,
    inner: T,
}

impl<T: ?Sized + Debug> Debug for Anchored<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Anchored").field(&&self.inner).finish()
    }
}

impl<T> Anchored<'static, T> {
    pub const fn new_static(inner: T) -> Self {
        Anchored {
            _anchor: PhantomData,
            inner,
        }
    }
}

impl<T> Anchored<'_, T> {
    /// Get the inner value.
    ///
    /// # Safety
    ///
    /// Make sure you don't outlive [`crate::marker::Unleak`]
    pub unsafe fn unanchor(slot: Self) -> T {
        slot.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for Anchored<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for Anchored<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> Future for Anchored<'_, T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner).poll(cx) }
    }
}
