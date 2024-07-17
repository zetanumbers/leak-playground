//! Possible [`std::rc`] replacements.

use core::fmt;
use std::rc::Rc as StdRc;

use crate::marker::Forget;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rc<T> {
    inner: StdRc<T>,
}

impl<T> Rc<T> {
    pub fn new(x: T) -> Self
    where
        T: Forget,
    {
        Rc {
            inner: StdRc::new(x),
        }
    }

    pub unsafe fn new_unchecked(x: T) -> Self {
        Rc {
            inner: StdRc::new(x),
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        Rc {
            inner: StdRc::clone(&self.inner),
        }
    }
}

impl<T> AsRef<T> for Rc<T> {
    fn as_ref(&self) -> &T {
        StdRc::as_ref(&self.inner)
    }
}

impl<T> core::borrow::Borrow<T> for Rc<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> fmt::Display for Rc<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<T> fmt::Debug for Rc<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> fmt::Pointer for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}
