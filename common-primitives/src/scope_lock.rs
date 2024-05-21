// TODO: fix clippy warnings
use std::{
    mem::transmute,
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

use leak_playground_std::marker::Unleak;

#[derive(Default)]
pub struct ScopeLock {
    scope_mutex: Mutex<()>,
}

impl ScopeLock {
    pub const fn new() -> Self {
        Self {
            scope_mutex: Mutex::new(()),
        }
    }

    pub fn lock_fn<'new, 'old, 'lock, F, I, O>(
        &'lock mut self,
        f: &'old F,
    ) -> (ExtendedFn<'new, I, O>, ScopeGuard<'new, 'old>)
    where
        F: Fn(I) -> O, // no 'extended bound to not restrict captured state
        I: 'new,
        O: 'new,
        'lock: 'old,
    {
        unsafe {
            (
                ExtendedFn {
                    func: transmute(f as &dyn Fn(I) -> O),
                    _scope_lock: transmute(
                        self.scope_mutex.lock().unwrap_or_else(|e| e.into_inner()),
                    ),
                },
                ScopeGuard {
                    scope_mutex: Unleak::with_arbitrary_lifetime(&self.scope_mutex),
                },
            )
        }
    }

    pub fn lock_fn_mut<'new, 'old, 'lock, F, I, O>(
        &'lock mut self,
        f: &'old mut F,
    ) -> (ExtendedFnMut<'new, I, O>, ScopeGuard<'new, 'old>)
    where
        F: FnMut(I) -> O, // no 'extended bound to not restrict captured state
        I: 'new,
        O: 'new,
        'lock: 'old,
    {
        unsafe {
            (
                ExtendedFnMut {
                    func: transmute(f as &mut dyn FnMut(I) -> O),
                    _scope_lock: transmute(
                        self.scope_mutex.lock().unwrap_or_else(|e| e.into_inner()),
                    ),
                },
                ScopeGuard {
                    scope_mutex: Unleak::with_arbitrary_lifetime(&self.scope_mutex),
                },
            )
        }
    }
}

pub struct ScopeGuard<'new, 'old> {
    scope_mutex: Unleak<'new, &'old Mutex<()>>,
}

impl<'new, 'old> Drop for ScopeGuard<'new, 'old> {
    fn drop(&mut self) {
        let _synchronize = self.scope_mutex.lock().unwrap_or_else(|e| e.into_inner());
    }
}

pub struct ExtendedFn<'a, I: 'a, O: 'a> {
    func: &'a dyn Fn(I) -> O,
    _scope_lock: MutexGuard<'static, ()>,
}

impl<'a, I: 'a, O: 'a> Deref for ExtendedFn<'a, I, O> {
    type Target = dyn Fn(I) -> O + 'a;

    fn deref(&self) -> &Self::Target {
        self.func
    }
}

pub struct ExtendedFnMut<'a, I: 'a, O: 'a> {
    func: &'a mut dyn FnMut(I) -> O,
    _scope_lock: MutexGuard<'static, ()>,
}

impl<'a, I: 'a, O: 'a> Deref for ExtendedFnMut<'a, I, O> {
    type Target = dyn FnMut(I) -> O + 'a;

    fn deref(&self) -> &Self::Target {
        self.func
    }
}

impl<'a, I: 'a, O: 'a> DerefMut for ExtendedFnMut<'a, I, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.func
    }
}
