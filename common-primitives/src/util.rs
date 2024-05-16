use std::ptr;

pub struct UnsafeInternalRef<T> {
    ptr: ptr::NonNull<T>,
}

unsafe impl<T: Sync> Send for UnsafeInternalRef<T> {}
unsafe impl<T: Sync> Sync for UnsafeInternalRef<T> {}

impl<T> Clone for UnsafeInternalRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnsafeInternalRef<T> {}

impl<T> UnsafeInternalRef<T> {
    pub const unsafe fn new(ptr: *const T) -> Self {
        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr.cast_mut()) },
        }
    }
}

impl<T> AsRef<T> for UnsafeInternalRef<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}
