//! Widgets to put on a window.
pub use self::tab::*;

pub(crate) use self::private::*;

mod tab;

mod private {
    /// Provides a method to get a pointer to `QWidget`.
    ///
    /// This is a sealed trait.
    ///
    /// # Safety
    /// [Self::as_ptr()] must return a valid pointer to `QWidget` object.
    pub unsafe trait Widget {
        fn as_ptr(&self) -> *mut u8;
        fn parent(&self) -> *mut u8;
    }

    /// Provides a method to get a pointer to `QWidget` that can contains other widgets.
    ///
    /// This is a sealed trait.
    ///
    /// # Safety
    /// [Self::as_ptr()] must return a valid pointer to `QWidget` object.
    pub unsafe trait Container {
        fn as_ptr(&self) -> *mut u8;
    }
}
