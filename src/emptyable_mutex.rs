use std::cell::UnsafeCell;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU8, Ordering};
use crossbeam_utils::Backoff;

pub(crate) struct Mutex<T> {
    guard: AtomicU8,
    val: UnsafeCell<T>,
}

const GUARD_UNLOCKED_FULL: u8 = 0;
const GUARD_UNLOCKED_EMPTY: u8 = 1;
const GUARD_LOCKED: u8 = 2;

impl<T> Mutex<T> {

    pub const fn new_full(val: T) -> Self {
        Self {
            guard: AtomicU8::new(GUARD_UNLOCKED_FULL),
            val: UnsafeCell::new(val),
        }
    }

    pub const fn new_empty(val: T) -> Self {
        Self {
            guard: AtomicU8::new(GUARD_UNLOCKED_EMPTY),
            val: UnsafeCell::new(val),
        }
    }

    pub fn lock(&self) -> Option<MutexGuard<T, true>> {
        let backoff = Backoff::new();
        loop {
            match self.guard.compare_exchange(GUARD_UNLOCKED_FULL, GUARD_LOCKED, Ordering::Acquire, Ordering::Acquire) {
                Ok(_) => {
                    return Some(MutexGuard(self));
                }
                Err(err) => {
                    if err == GUARD_UNLOCKED_EMPTY {
                        return None;
                    }
                    backoff.snooze();
                }
            }
        }
    }

}

pub(crate) struct MutexGuard<'a, T, const FULL: bool>(&'a Mutex<T>);

impl<'a, T, const FULL: bool> Deref for MutexGuard<'a, T, FULL> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &'a Self::Target {
        unsafe { &*self.0.val.get() }
    }
}

impl<'a, T, const FULL: bool> DerefMut for MutexGuard<'a, T, FULL> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { &mut *self.0.val.get() }
    }
}

impl<'a, T> MutexGuard<'a, T, true> {

    #[inline]
    pub fn empty(self) -> MutexGuard<'a, T, false> {
        let ret = MutexGuard(self.0);
        mem::forget(self);
        ret
    }

}

impl<'a, T> MutexGuard<'a, T, false> {

    #[inline]
    pub fn fill(self) -> MutexGuard<'a, T, true> {
        let ret = MutexGuard(self.0);
        mem::forget(self);
        ret
    }

}

impl<T, const FULL: bool> Drop for MutexGuard<'_, T, FULL> {
    fn drop(&mut self) {
        let guard = if FULL {
            GUARD_UNLOCKED_FULL
        } else {
            GUARD_UNLOCKED_EMPTY
        };
        self.0.guard.store(guard, Ordering::Release);
    }
}
