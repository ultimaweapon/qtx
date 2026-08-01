//! Utilities related to memory management.
pub use self::heap_ptr::*;
pub use self::owned::*;
pub use self::strong::*;

use std::ffi::c_void;

mod heap_ptr;
mod owned;
mod strong;

unsafe extern "C-unwind" {
    static qtx_max_align: usize;

    fn qtx_delete(ptr: *mut c_void);
}
