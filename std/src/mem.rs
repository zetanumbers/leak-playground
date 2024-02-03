//! Possible [`core::mem`] additions and replacements.

use core::mem;

use crate::marker::Leak;

pub fn forget<T: Leak>(x: T) {
    mem::forget(x)
}

pub unsafe fn forget_unchecked<T>(x: T) {
    mem::forget(x)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ManuallyDrop<T: ?Sized> {
    inner: mem::ManuallyDrop<T>,
}

impl<T> ManuallyDrop<T> {
    pub fn new(value: T) -> Self
    where
        T: Leak,
    {
        Self {
            inner: mem::ManuallyDrop::new(value),
        }
    }

    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            inner: mem::ManuallyDrop::new(value),
        }
    }

    pub unsafe fn take(slot: &mut ManuallyDrop<T>) -> T {
        mem::ManuallyDrop::take(&mut slot.inner)
    }

    pub const fn into_inner(slot: ManuallyDrop<T>) -> T {
        mem::ManuallyDrop::into_inner(slot.inner)
    }
}

impl<T: ?Sized> std::ops::DerefMut for ManuallyDrop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for ManuallyDrop<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
