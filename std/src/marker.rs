//! Possible [`core::marker`] additions. Contains the proposed [`Leak`] trait.

/// The core trait of the destruction guarantee.
///
/// # Safety
///
/// Implement only if you know there's absolutely no possible way to
/// leak your type.
pub unsafe auto trait Leak {}

/// A simple wrapper to make your types `!Leak`
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

#[cfg(feature = "tokio_rt")]
mod tokio_rt {
    unsafe impl<T: 'static> super::Leak for tokio::task::JoinHandle<T> {}
}
