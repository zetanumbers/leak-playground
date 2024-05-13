//! A new module aimed to relax unforgetness in specific cases

use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task};

/// A transparent wrapper type to use with [`crate::marker::Unleak`]. To
/// anchor a value is to restrict its lifetime.
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Anchored<'a, T: ?Sized> {
    /// Invariant lifetime, shortening only allowed via [`Anchored::anchor`]
    _anchor: PhantomData<fn(&'a ()) -> &'a ()>,
    inner: T,
}

impl<T: ?Sized + Debug> Debug for Anchored<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Anchored").field(&&self.inner).finish()
    }
}

impl<T> Anchored<'static, T> {
    pub const fn new(inner: T) -> Self {
        Anchored {
            _anchor: PhantomData,
            inner,
        }
    }
}

// NOTE: Library authors should use `Anchored` as with `Pin`, i.e. give
// those directly to the user or at least use it with their public API.
//
// TODO: Maybe allow to implement methods on `self: Anchored<'a, Self>`
// within the compiler and remove `Defer` impl
impl<'a, T> Anchored<'a, T> {
    pub fn anchor<'b>(self, _borrow: &'b T) -> Anchored<'b, T>
    where
        'a: 'b,
        T: 'b,
    {
        Anchored {
            inner: self.inner,
            _anchor: PhantomData,
        }
    }

    /// Get the inner value.
    ///
    /// # Safety
    ///
    /// Make sure you don't outlive your [`crate::marker::Unleak`]
    /// instances.
    pub unsafe fn unanchor(self) -> T {
        self.inner
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
