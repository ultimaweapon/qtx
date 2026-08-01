use std::alloc::Layout;
use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;

use crate::EXECUTOR;
use crate::mem::{RefCnt, Strong};

/// Encapsulates an instance of [QApplication](https://doc.qt.io/qt-6/qapplication.html).
pub struct App {
    _pd: PhantomData<*mut ()>, // For !send and !Sync.
    _pp: PhantomPinned,
    mem: [u8],
}

impl App {
    pub(crate) unsafe fn new(argc: *mut c_int, argv: *mut *mut c_char) -> Pin<Strong<Self>> {
        let data = Data { refs: Cell::new(0) };

        // Create.
        let (layout, off) = unsafe { Layout::from_size_align(qtx_app_size, qtx_app_align) }
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();
        let app = unsafe { qtx_app_new(layout.size(), layout.align(), argc, argv) };
        let app = unsafe {
            std::ptr::write(app.add(off).cast(), data);
            Strong::new(std::ptr::slice_from_raw_parts_mut(app, off) as *mut App)
        };

        unsafe { Pin::new_unchecked(app) }
    }

    /// Spawns a new asynchronous task.
    ///
    /// # Panics
    /// If called after the main task was finished.
    pub fn spawn<F>(&self, f: F)
    where
        F: AsyncFnOnce(Pin<Strong<App>>) + 'static,
    {
        let app = unsafe { Pin::new_unchecked(Strong::new(self)) };
        let f = f(app);

        unsafe { EXECUTOR.lock().unwrap().as_ref().unwrap().spawn(f) };
    }

    #[inline(always)]
    fn data(&self) -> &Data {
        let base = self.mem.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { &*data }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let base = self.mem.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { std::ptr::drop_in_place::<Data>(data) };
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

struct Data {
    refs: Cell<usize>,
}

unsafe extern "C-unwind" {
    static qtx_app_size: usize;
    static qtx_app_align: usize;

    fn qtx_app_new(size: usize, align: usize, argc: *mut c_int, argv: *mut *mut c_char) -> *mut u8;
    fn qtx_app_destroy(app: *mut u8);
}
