use std::cell::UnsafeCell;
use std::mem::{align_of, align_of_val, ManuallyDrop, MaybeUninit, size_of, size_of_val, transmute};
use std::ptr;

pub struct InlinableDynPtr<T: ?Sized> {
    vtable_ptr: *const (),
    val: UnsafeCell<*mut ()>, // the first bit indicates whether the value is inlined or not!
    _align: [*mut T; 0],
}

unsafe impl<T: Send> Send for InlinableDynPtr<T> {}
unsafe impl<T: Sync> Sync for InlinableDynPtr<T> {}

const NOT_INLINED: usize = 0;
const INLINED: usize = 1;

impl<T: ?Sized> InlinableDynPtr<T> {

    /// SAFETY: `val` may not be dropped after calling this, except this method panics
    /// in this case, dropping `val` is okay.
    pub unsafe fn new<F: FnOnce(&T) -> *mut T>(val: &ManuallyDrop<T>, alloc: F) -> Self {
        let ptr = val as *const ManuallyDrop<T> as *const T;
        if size_of::<*const T>() == size_of::<usize>() {
            // FIXME: add fast path for non-dyn types
        }
        let components = ptr.to_raw_parts();
        let (data_ptr, vtable) = (components.0, components.1 as *const ());
        if align_of::<*mut ()>() < 2 {
            unreachable!("Only alignments for pointers of 2 or more bytes are supported!");
        }
        let align = align_of_val(val);
        let size = size_of_val(val);
        // let size_align = size.max(align);
        // FIXME: support types with a size larger than usize/2 in size and alignment!
        if size <= size_of::<*mut ()>() / 2 && align <= align_of::<*mut ()>() / 2 {
            let mut val = MaybeUninit::uninit();
            // write the metadata byte right into the first byte
            val.as_mut_ptr().cast::<u8>().write(INLINED as u8);
            val.as_mut_ptr().cast::<u8>().add(size_of::<usize>() / 2).cast::<T>().write(ptr::read(ptr));
            Self {
                vtable_ptr: vtable,
                val: UnsafeCell::new(val.assume_init()),
                _align: [],
            }
        } else {
            Self {
                vtable_ptr: vtable,
                val: UnsafeCell::new(alloc(val)),
                _align: [],
            }
        }
    }

    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        let val = *self.val.get();
        if val as usize & INLINED == 1 {
            val.offset(4).as_ref().unwrap_unchecked()
        } else {
            val
        }
    }

    #[inline]
    pub unsafe fn as_mut(&self) -> &mut T {
        let val = *self.val.get();
        if val as usize & INLINED == 1 {
            val.offset(4).as_mut().unwrap_unchecked()
        } else {
            val
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *mut T {
        unsafe { *self.val.get() }
    }

}

pub struct InlinablePtr<T> {
    val: UnsafeCell<*mut T>,
}

unsafe impl<T: Send> Send for InlinablePtr<T> {}
unsafe impl<T: Send> Sync for InlinablePtr<T> {}

impl<T> InlinablePtr<T> {
    
    pub fn new<F: FnOnce(T) -> *mut T>(val: T, alloc: F) -> Self {
        if size_of::<T>() <= size_of::<*const T>() && align_of::<T>() <= align_of::<*const T>() {
            let mut val = MaybeUninit::uninit();
            unsafe { val.as_mut_ptr().cast::<T>().write(val); }
            Self {
                val: UnsafeCell::new(unsafe { val.assume_init() }),
            }
        } else {
            Self {
                val: UnsafeCell::new(alloc(val)),
            }
        }
    }
    
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        self.val.get().as_ref().unwrap_unchecked().as_ref().unwrap_unchecked()
    }

    #[inline]
    pub unsafe fn as_mut(&self) -> &mut T {
        self.val.get().as_mut().unwrap_unchecked().as_mut().unwrap_unchecked()
    }

    #[inline]
    pub fn as_raw(&self) -> *mut T {
        unsafe { *self.val.get() }
    }
}
