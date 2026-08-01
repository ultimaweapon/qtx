use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{Context, Poll};

/// Encapsulates a pointer to heap allocated C++ object.
pub struct Owned<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> Owned<T> {
    /// # Safety
    /// `v` cannot be null and must point to initialized value.
    #[inline(always)]
    pub(crate) unsafe fn new(v: *mut T) -> Self {
        Self(unsafe { NonNull::new_unchecked(v) })
    }
}

impl<T: ?Sized> Drop for Owned<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { std::ptr::drop_in_place(self.0.as_ptr()) };
    }
}

impl<T: ?Sized> Deref for Owned<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Owned<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T: Future + Unpin + ?Sized> Future for Owned<T> {
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::poll(Pin::new(self.deref_mut()), cx)
    }
}

unsafe impl<T: Send + ?Sized> Send for Owned<T> {}
