//! Possible `Rc` implementation

use core::fmt;
use std::sync::Arc as StdArc;

use crate::marker::Forget;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arc<T> {
    inner: StdArc<T>,
}

impl<T> Arc<T> {
    pub fn new(x: T) -> Self
    where
        T: Forget,
    {
        Arc {
            inner: StdArc::new(x),
        }
    }

    pub unsafe fn new_unchecked(x: T) -> Self {
        Arc {
            inner: StdArc::new(x),
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Arc {
            inner: StdArc::clone(&self.inner),
        }
    }
}

impl<T> AsRef<T> for Arc<T> {
    fn as_ref(&self) -> &T {
        StdArc::as_ref(&self.inner)
    }
}

impl<T> core::borrow::Borrow<T> for Arc<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

impl<T> std::ops::Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> fmt::Display for Arc<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<T> fmt::Debug for Arc<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}
