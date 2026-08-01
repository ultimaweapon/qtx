//! Asynchronous GUI framework based on Qt Widgets.
//!
//! # Prerequisites
//!
//! - CMake 3.20
//! - C++17 toolchain
//! - Qt 6
//!
//! # Specify Qt location
//!
//! You can set environment variable `QTX_QT_PATH` to point to Qt's directory. Usually you will need
//! this when you download Qt from its official site. For Qt that installed by a package manager it
//! is unlikely you will need this.
#![allow(clippy::new_without_default)] // Default on some type does not make sense.
#![allow(clippy::type_complexity)] // Type aliasing hide the actual type.
pub use self::app::*;

use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Mutex;

use memchr::memchr;
use thiserror::Error;

use self::executor::Executor;
use self::mem::{HeapPtr, Owned, Strong};

pub mod mem;
pub mod windows;

mod app;
mod executor;

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
    ///
    /// This methods return [None] if Qt's event loop exit before `f` run to completion.
    pub fn run<A, T, R>(
        self,
        args: A,
        f: impl AsyncFnOnce(Pin<Strong<App>>) -> R + 'static,
    ) -> Result<Option<R>, RuntimeError>
    where
        A: IntoIterator<Item = T>,
        T: AsRef<str>,
        R: 'static,
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

        // Create App
        let app = unsafe { App::new(&mut argc, argv.as_mut_ptr().cast()) };

        *EXECUTOR.lock().unwrap() = unsafe { Some(Executor::new()) };

        // Run event loop.
        let f = f(app.clone()); // Make sure QApplication alive for QApplication::exec.
        let r = Rc::new(Cell::new(None));
        let v = r.clone();
        let f = async move {
            v.set(Some(f.await));
            unsafe { qtx_exit(0) };
        };

        unsafe { EXECUTOR.lock().unwrap().as_ref().unwrap().spawn(f) };

        unsafe { qtx_application_exec() };

        // Drop Executor before App since unfinished tasks may access QApplication during droping.
        *EXECUTOR.lock().unwrap() = None;

        drop(app);

        Ok(r.take())
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

static EXECUTOR: Mutex<Option<Pin<Owned<Executor>>>> = Mutex::new(None);

unsafe extern "C-unwind" {
    fn qtx_application_set_style(name: *const c_char, len: isize) -> bool;
    fn qtx_application_set_organization_name(name: *const c_char, len: isize);
    fn qtx_application_set_application_name(name: *const c_char, len: isize);
    fn qtx_application_exec() -> c_int;
    fn qtx_exit(code: c_int);
}
