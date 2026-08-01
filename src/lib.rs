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

use std::alloc::Layout;
use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Mutex;

use memchr::memchr;
use thiserror::Error;

use self::executor::Executor;
use self::mem::{HeapPtr, Owned, RefCnt, Strong};

pub mod mem;
pub mod windows;

mod executor;

/// Encapsulates an instance of [QApplication](https://doc.qt.io/qt-6/qapplication.html).
pub struct App {
    phantom: PhantomData<*mut ()>, // For !send and !Sync.
    mem: [u8],
}

impl App {
    /// Spawns a new asynchronous task.
    ///
    /// # Panics
    /// If called after the main task was finished.
    pub fn spawn<F>(&self, f: F)
    where
        F: AsyncFnOnce(Strong<App>) + 'static,
    {
        let app = unsafe { Strong::new(self) };
        let f = f(app);

        unsafe { EXECUTOR.lock().unwrap().as_ref().unwrap().spawn(f) };
    }

    #[inline(always)]
    fn data(&self) -> &AppData {
        let base = self.mem.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { &*data }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let base = self.mem.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { std::ptr::drop_in_place::<AppData>(data) };
        unsafe { qtx_app_destroy(base) };
    }
}

unsafe impl RefCnt for App {
    fn increase_ref(&self) {
        let d = self.data();
        let v = d.refs.get();

        d.refs.set(v.strict_add(1));
    }

    fn decrease_ref(&self) -> usize {
        let d = self.data();
        let v = d.refs.get() - 1;

        d.refs.set(v);

        v
    }
}

struct AppData {
    refs: Cell<usize>,
}

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
    pub fn run<A, T, R>(
        self,
        args: A,
        f: impl AsyncFnOnce(Strong<App>) -> R + 'static,
    ) -> Result<R, RuntimeError>
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

        // Construct AppData.
        let data = AppData { refs: Cell::new(0) };

        // Get memory layout for QApplication extended with AppData.
        let (layout, off) = unsafe { Layout::from_size_align(qtx_app_size, qtx_app_align) }
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();

        // Create QApplication.
        let argv = argv.as_mut_ptr().cast();
        let app = unsafe { qtx_app_new(layout.size(), layout.align(), &mut argc, argv) };
        let app = unsafe {
            std::ptr::write(app.add(off).cast(), data);
            Strong::new(std::ptr::slice_from_raw_parts_mut(app, off) as *mut App)
        };

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

        Ok(r.take().unwrap())
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

static EXECUTOR: Mutex<Option<Owned<Executor>>> = Mutex::new(None);

unsafe extern "C-unwind" {
    static qtx_app_size: usize;
    static qtx_app_align: usize;

    fn qtx_application_set_style(name: *const c_char, len: isize) -> bool;
    fn qtx_application_set_organization_name(name: *const c_char, len: isize);
    fn qtx_application_set_application_name(name: *const c_char, len: isize);
    fn qtx_application_exec() -> c_int;
    fn qtx_exit(code: c_int);

    fn qtx_app_new(size: usize, align: usize, argc: *mut c_int, argv: *mut *mut c_char) -> *mut u8;
    fn qtx_app_destroy(app: *mut u8);
}
