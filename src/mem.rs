use core::mem;

use crate::Leak;

pub fn forget<T: Leak>(x: T) {
    mem::forget(x)
}

pub unsafe fn forget_unchecked<T>(x: T) {
    mem::forget(x)
}
