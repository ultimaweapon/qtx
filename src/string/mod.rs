//! Provides a specialized string that efficient to construct from
//! [QString](https://doc.qt.io/qt-6/qstring.html).
use std::alloc::{Layout, handle_alloc_error};
use std::cell::Cell;
use std::ops::Deref;

/// A reference-counted immutable string.
///
/// The main difference from [std::rc::Rc] of [str] is this type allows conversion from UTF-16
/// string into UTF-8 string on a reference-counted memory block directly, which result in a single
/// allocation in the majority of cases rather than 2 allocations.
#[repr(transparent)]
pub struct Str(*const Data);

impl Str {
    /// Creates [Str] from a UTF-16 string.
    ///
    /// Returns [None] if `v` is not valid UTF-16.
    pub fn from_utf16(v: &[u16]) -> Option<Self> {
        // Allocate memory.
        let size = size_of::<usize>() + v.len() * 4;
        let layout = Layout::from_size_align(size, align_of::<usize>()).unwrap();
        let mut mem = unsafe { std::alloc::alloc(layout) };

        if mem.is_null() {
            handle_alloc_error(layout);
        }

        // Decode.
        let data = unsafe { mem.add(size_of::<usize>()) };
        let mut next = data;
        let mut buf = [0; 4];

        for c in char::decode_utf16(v.iter().copied()) {
            // Check if succeed.
            let c = match c {
                Ok(v) => v,
                Err(_) => {
                    unsafe { std::alloc::dealloc(mem, layout) };
                    return None;
                }
            };

            // Encode to UTF-8.
            let c = c.encode_utf8(&mut buf);

            unsafe { next.copy_from_nonoverlapping(c.as_ptr(), c.len()) };
            unsafe { next = next.add(c.len()) };
        }

        // Shrink memory.
        let len = unsafe { next.offset_from_unsigned(data) };
        let size = size_of::<usize>() + len;

        if size != layout.size() {
            mem = unsafe { std::alloc::realloc(mem, layout, size) };

            if mem.is_null() {
                handle_alloc_error(Layout::from_size_align(size, align_of::<usize>()).unwrap());
            }
        }

        unsafe { std::ptr::write(mem.cast(), 1usize) };

        Some(Self(std::ptr::slice_from_raw_parts(mem, len) as *const Data))
    }

    #[unsafe(no_mangle)]
    unsafe extern "C-unwind" fn qtx_str_from_utf16(s: *const u16, l: usize) -> Self {
        Self::from_utf16(unsafe { std::slice::from_raw_parts(s, l) }).unwrap()
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        let d = self.0;
        let v = unsafe { (*d).refs.get() - 1 };

        unsafe { (*d).refs.set(v) };

        if v == 0 {
            let l = Layout::for_value(unsafe { &*d });

            unsafe { std::alloc::dealloc(d.cast_mut().cast(), l) };
        }
    }
}

impl Clone for Str {
    #[inline]
    fn clone(&self) -> Self {
        let d = self.0;

        unsafe { (*d).refs.update(|v| v.strict_add(1)) };

        Self(d)
    }
}

impl Deref for Str {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        let d = self.0;

        unsafe { std::str::from_utf8_unchecked(&(*d).data) }
    }
}

impl From<&str> for Str {
    fn from(v: &str) -> Self {
        let size = size_of::<usize>() + v.len();
        let layout = Layout::from_size_align(size, align_of::<usize>()).unwrap();
        let mem = unsafe { std::alloc::alloc(layout) };

        if mem.is_null() {
            handle_alloc_error(layout);
        }

        unsafe { std::ptr::write(mem.cast(), 1usize) };
        unsafe { std::ptr::copy_nonoverlapping(v.as_ptr(), mem.add(size_of::<usize>()), v.len()) };

        Self(std::ptr::slice_from_raw_parts(mem, v.len()) as *const Data)
    }
}

#[repr(C)]
struct Data {
    refs: Cell<usize>,
    data: [u8],
}
