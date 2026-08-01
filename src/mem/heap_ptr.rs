use std::alloc::{Layout, handle_alloc_error};
use std::mem::ManuallyDrop;
use std::num::NonZero;
use std::ptr::NonNull;

use libc::{free, malloc};

use super::qtx_max_align;

/// Encapsulates a pointer to memory allocated by
/// [malloc](https://en.cppreference.com/c/memory/malloc).
#[repr(transparent)]
pub struct HeapPtr<T>(NonNull<T>);

impl<T> HeapPtr<T> {
    /// Allocate a memory for `T`.
    ///
    /// # Panics
    /// If `T` is ZST.
    #[inline]
    pub fn new() -> Self {
        let layout = Layout::new::<T>();

        if layout.size() == 0 {
            panic!("ZST is not supported");
        } else if unsafe { layout.align() > qtx_max_align } {
            handle_alloc_error(layout);
        }

        // Allocate.
        let ptr = unsafe { malloc(layout.size()) };

        match NonNull::new(ptr.cast()) {
            Some(v) => Self(v),
            None => handle_alloc_error(layout),
        }
    }

    /// Allocate a new array of `T`.
    ///
    /// # Panics
    /// If `T` is ZST.
    #[inline]
    pub fn array(n: NonZero<usize>) -> Self {
        // The only cases this will going to fails is the result size is too large. We assume that
        // it is impossible to allocate such a memory so we treat is as a panic like memory
        // allocation fails.
        let layout = Layout::array::<T>(n.get()).unwrap();

        if layout.size() == 0 {
            panic!("ZST is not supported");
        } else if unsafe { layout.align() > qtx_max_align } {
            handle_alloc_error(layout);
        }

        // Allocate.
        let ptr = unsafe { malloc(layout.size()) };

        match NonNull::new(ptr.cast()) {
            Some(v) => Self(v),
            None => handle_alloc_error(layout),
        }
    }

    /// Returns the encapsulated pointer.
    #[inline(always)]
    pub fn get(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// Releases the ownership of the pointer.
    #[inline(always)]
    pub fn into_raw(self) -> *mut T {
        ManuallyDrop::new(self).0.as_ptr()
    }
}

impl<T> Drop for HeapPtr<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { free(self.0.as_ptr().cast()) };
    }
}

unsafe impl<T: Send> Send for HeapPtr<T> {}
