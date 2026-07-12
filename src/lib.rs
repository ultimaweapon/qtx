//! Cross-platform GUI framework based on Qt Widgets.
//!
//! # Prerequisites
//!
//! - CMake 3.20
//! - C++ toolchain
//! - Qt 6
use std::ffi::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};

use libc::free;
use memchr::memchr;
use thiserror::Error;

use self::ffi::HeapPtr;

pub mod ffi;

/// Encapsulates an instance of [QApplication](https://doc.qt.io/qt-6/qapplication.html).
///
/// The value of this struct can construct only once.
pub struct App {
    app: *mut QApplication,
    argc: *mut c_int,
    argv: *mut [Option<HeapPtr<c_char>>],
}

impl App {
    /// Enter main event loop.
    pub fn run<R>(self, f: impl Future<Output = R>) -> R {
        todo!()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { qtx_application_destroy(self.app) };
        unsafe { free(self.argc.cast()) };
        unsafe { drop(Box::from_raw(self.argv)) };
    }
}

/// Provides method to build [App]'s instance.
pub struct Builder {}

impl Builder {
    /// Create an instance of [App].
    pub fn build<A, T>(self, args: A) -> Result<App, AppError>
    where
        A: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        // Check for a second call.
        static CALLED: AtomicBool = AtomicBool::new(false);

        if CALLED.swap(true, Ordering::Acquire) {
            return Err(AppError::RecreationAttempt);
        }

        // Build argv.
        let mut argv = Vec::new();

        for (i, arg) in args.into_iter().enumerate() {
            // Check if contains NUL.
            let arg = arg.as_ref().as_bytes();

            if memchr(0, arg).is_some() {
                return Err(AppError::ArgContainsNul(i));
            }

            // Allocate C string.
            let len = (arg.len() + 1).try_into().unwrap();
            let buf = HeapPtr::<c_char>::array(len);
            let src = arg.as_ptr().cast();

            unsafe { buf.get().copy_from_nonoverlapping(src, arg.len()) };
            unsafe { buf.get().add(arg.len()).write(0) };

            argv.push(Some(buf));
        }

        if argv.is_empty() {
            return Err(AppError::ZeroArg);
        }

        // Allocate argc.
        let argc = HeapPtr::<c_int>::new();

        unsafe { argc.get().write(argv.len().try_into().unwrap()) };

        argv.push(None);

        // Create QApplication.
        let mut argv = argv.into_boxed_slice();
        let app = unsafe { qtx_application_new(argc.get(), argv.as_mut_ptr().cast()) };

        Ok(App {
            app,
            argc: argc.into_raw(),
            argv: Box::into_raw(argv),
        })
    }
}

/// Reason why [Builder::build()] fails.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AppError {
    /// Attempt to re-create App instance.
    #[error("attempt to re-create App instance")]
    RecreationAttempt,

    /// Some command line arguments contains NUL character.
    #[error("command line argument #{0} contains NUL character")]
    ArgContainsNul(usize),

    /// At least one command line argument is required.
    #[error("at least one command line argument is required")]
    ZeroArg,
}

struct QApplication([u8; 0]);

#[allow(improper_ctypes)]
unsafe extern "C-unwind" {
    fn qtx_application_new(argc: *mut c_int, argv: *mut *mut c_char) -> *mut QApplication;
    fn qtx_application_destroy(app: *mut QApplication);
}
