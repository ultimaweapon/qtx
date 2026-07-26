//! Asynchronous GUI framework based on Qt Widgets.
//!
//! # Prerequisites
//!
//! - CMake 3.20
//! - C++17 toolchain
//! - Qt 6
use std::alloc::Layout;
use std::borrow::Cow;
use std::ffi::{c_char, c_int};

use memchr::memchr;
use thiserror::Error;

use self::ffi::{HeapPtr, Owned};

pub mod ffi;

/// Encapsulates an instance of [QApplication](https://doc.qt.io/qt-6/qapplication.html).
pub struct App([u8]);

impl Drop for App {
    #[inline]
    fn drop(&mut self) {
        let base = self.0.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.0)).cast() };

        unsafe { std::ptr::drop_in_place::<AppData>(data) };
        unsafe { qtx_app_destroy(base) };
    }
}

struct AppData {}

/// Encapsulates Qt's event loop to run the application.
pub struct Runtime {
    organization_name: Option<Cow<'static, str>>,
    application_name: Option<Cow<'static, str>>,
    style: Option<Cow<'static, str>>,
}

impl Runtime {
    /// Create a new instance of [Runtime].
    ///
    /// This is the only unsafe function you need. Unfortunately it is impossible to make this
    /// function safe similar to [std::env::set_var()].
    ///
    /// # Safety
    /// [QCoreApplication](https://doc.qt.io/qt-6/qcoreapplication.html) or its derived classes must
    /// not been instantiated in the calling process. Usually the only cases this function unsafe to
    /// call are:
    ///
    /// - You have other Qt bindings.
    /// - You call this function a second time.
    pub unsafe fn new() -> Self {
        Self {
            organization_name: None,
            application_name: None,
            style: None,
        }
    }

    /// Set organization's name to be used with [QCoreApplication::setOrganizationName](https://doc.qt.io/qt-6/qcoreapplication.html#organizationName-prop).
    pub fn set_organization_name(&mut self, v: impl Into<Cow<'static, str>>) {
        self.organization_name = Some(v.into());
    }

    /// Set application's name to be used with [QCoreApplication::setApplicationName](https://doc.qt.io/qt-6/qcoreapplication.html#applicationName-prop).
    pub fn set_application_name(&mut self, v: impl Into<Cow<'static, str>>) {
        self.application_name = Some(v.into());
    }

    /// Set style name to be used with [QApplication::setStyle](https://doc.qt.io/qt-6/qapplication.html#setStyle-1).
    pub fn set_style(&mut self, v: impl Into<Cow<'static, str>>) {
        self.style = Some(v.into());
    }

    /// Run `f` to completion and return its result.
    pub fn run<A, T, R>(self, args: A, f: impl AsyncFnOnce(&App) -> R) -> Result<R, RuntimeError>
    where
        A: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        // Build argv.
        let mut argv = Vec::new();

        for (i, arg) in args.into_iter().enumerate() {
            // Check if contains NUL.
            let arg = arg.as_ref().as_bytes();

            if memchr(0, arg).is_some() {
                return Err(RuntimeError::ArgContainsNul(i));
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
            return Err(RuntimeError::ZeroArg);
        }

        // Set fallible properties.
        if let Some(v) = self.style {
            let l = v.len().try_into().unwrap();

            if unsafe { !qtx_application_set_style(v.as_ptr().cast(), l) } {
                return Err(RuntimeError::UnknownStyle(v.into_owned()));
            }
        }

        // Get argc.
        let mut argc = argv.len().try_into().unwrap();

        argv.push(None);

        // Set non-fallible properties.
        if let Some(v) = self.organization_name {
            let l = v.len().try_into().unwrap();

            unsafe { qtx_application_set_organization_name(v.as_ptr().cast(), l) };
        }

        if let Some(v) = self.application_name {
            let l = v.len().try_into().unwrap();

            unsafe { qtx_application_set_application_name(v.as_ptr().cast(), l) };
        }

        // Construct AppData.
        let data = AppData {};

        // Get memory layout for QApplication extended with AppData.
        let (layout, off) = unsafe { Layout::from_size_align(qtx_app_size, qtx_app_align) }
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();

        // Create QApplication.
        let argv = argv.as_mut_ptr().cast();
        let app = unsafe { qtx_app_new(layout.size(), layout.align(), &mut argc, argv) };

        unsafe { std::ptr::write(app.add(off).cast(), data) };

        // Run event loop.
        let app = unsafe { Owned::new(std::ptr::slice_from_raw_parts_mut(app, off) as *mut App) };
        let f = f(&app);

        unsafe { qtx_application_exec() };

        todo!()
    }
}

/// Reason why [Runtime::run()] fails.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Some command line arguments contains NUL character.
    #[error("command line argument #{0} contains NUL character")]
    ArgContainsNul(usize),

    /// At least one command line argument is required.
    #[error("at least one command line argument is required")]
    ZeroArg,

    /// An unknown style was passed to [Runtime::set_style()].
    #[error("unknown style '{0}'")]
    UnknownStyle(String),
}

#[allow(improper_ctypes)]
unsafe extern "C-unwind" {
    static qtx_app_size: usize;
    static qtx_app_align: usize;

    fn qtx_application_set_style(name: *const c_char, len: isize) -> bool;
    fn qtx_application_set_organization_name(name: *const c_char, len: isize);
    fn qtx_application_set_application_name(name: *const c_char, len: isize);
    fn qtx_application_exec() -> c_int;

    fn qtx_app_new(size: usize, align: usize, argc: *mut c_int, argv: *mut *mut c_char) -> *mut u8;
    fn qtx_app_destroy(app: *mut u8);
}
