//! Utilities related to FFI bindings.
pub use self::heap_ptr::*;

mod heap_ptr;

unsafe extern "C-unwind" {
    static qtx_max_align: usize;
}
