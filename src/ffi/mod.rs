//! Utilities related to FFI bindings.
pub use self::heap_ptr::*;
pub use self::owned::*;

mod heap_ptr;
mod owned;

unsafe extern "C-unwind" {
    static qtx_max_align: usize;
}
