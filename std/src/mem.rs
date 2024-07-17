//! Possible [`core::mem`] additions and replacements.

use core::mem;

use crate::marker::Forget;

pub fn forget<T: Forget>(x: T) {
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

unsafe impl<T: ?Sized> Forget for ManuallyDrop<T> {}

impl<T> ManuallyDrop<T> {
    pub const fn new(value: T) -> Self
    where
        T: Forget,
    {
        Self {
            inner: mem::ManuallyDrop::new(value),
        }
    }

    pub const unsafe fn new_unchecked(value: T) -> Self {
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

#[repr(transparent)]
pub struct MaybeUninit<T> {
    inner: mem::MaybeUninit<T>,
}

unsafe impl<T> Forget for MaybeUninit<T> {}

impl<T> MaybeUninit<T> {
    pub const fn new(val: T) -> Self
    where
        T: Forget,
    {
        MaybeUninit {
            inner: mem::MaybeUninit::new(val),
        }
    }

    pub const unsafe fn new_unchecked(val: T) -> Self {
        MaybeUninit {
            inner: mem::MaybeUninit::new(val),
        }
    }

    pub const fn uninit() -> Self {
        MaybeUninit {
            inner: mem::MaybeUninit::uninit(),
        }
    }

    pub const fn zeroed() -> Self {
        MaybeUninit {
            inner: mem::MaybeUninit::uninit(),
        }
    }

    pub fn write(&mut self, val: T) -> &mut T
    where
        T: Forget,
    {
        self.inner.write(val)
    }

    pub unsafe fn write_unchecked(&mut self, val: T) -> &mut T {
        self.inner.write(val)
    }

    pub const fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    pub const unsafe fn assume_init(self) -> T {
        self.inner.assume_init()
    }

    pub const unsafe fn assume_init_read(&self) -> T {
        self.inner.assume_init_read()
    }

    pub unsafe fn assume_init_drop(&mut self) {
        self.inner.assume_init_drop()
    }

    pub const unsafe fn assume_init_ref(&self) -> &T {
        self.inner.assume_init_ref()
    }

    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        self.inner.assume_init_mut()
    }
}

impl<T: Copy> Copy for MaybeUninit<T> {}

impl<T: Copy> Clone for MaybeUninit<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for MaybeUninit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
