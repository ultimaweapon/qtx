use std::ops::Deref;
use std::ptr::NonNull;

/// Strong reference to reference-counted C++ object.
pub struct Strong<T: RefCnt + ?Sized>(NonNull<T>);

impl<T: RefCnt + ?Sized> Strong<T> {
    /// Create a first strong reference to the object.
    ///
    /// # Safety
    /// `v` cannot be null and must point to initialized value.
    #[inline(always)]
    pub unsafe fn new(v: *const T) -> Self {
        unsafe { (*v).increase_ref() };

        Self(unsafe { NonNull::new_unchecked(v.cast_mut()) })
    }
}

impl<T: RefCnt + ?Sized> Drop for Strong<T> {
    #[inline]
    fn drop(&mut self) {
        let v = self.0.as_ptr();
        let r = unsafe { (*v).decrease_ref() };

        if r == 0 {
            unsafe { std::ptr::drop_in_place(v) };
        }
    }
}

impl<T: RefCnt + ?Sized> Deref for Strong<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: RefCnt + ?Sized> Clone for Strong<T> {
    #[inline]
    fn clone(&self) -> Self {
        let v = self.0.as_ptr();

        unsafe { (*v).increase_ref() };

        Self(unsafe { NonNull::new_unchecked(v) })
    }
}

/// Provides methods to increase/decrease strong reference to reference-counted C++ object.
///
/// # Safety
/// The number of strong references store on the object can only modified by [Self::increase_ref()]
/// and [Self::decrease_ref()].
pub unsafe trait RefCnt {
    /// Increments the strong reference count on the object.
    ///
    /// # Panics
    /// If reference count already at [usize::MAX].
    fn increase_ref(&self);

    /// Decrements the strong reference count on the object and returns remaining references.
    fn decrease_ref(&self) -> usize;
}
