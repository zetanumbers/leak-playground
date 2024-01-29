use core::mem;

use crate::Leak;

pub fn forget<T: Leak>(x: T) {
    mem::forget(x)
}

pub unsafe fn forget_unchecked<T>(x: T) {
    mem::forget(x)
}

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
}
