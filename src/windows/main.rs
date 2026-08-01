use std::alloc::Layout;
use std::borrow::Cow;
use std::cell::Cell;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use crate::App;
use crate::mem::Owned;

/// Main application window.
pub struct MainWindow<'a> {
    _pd: PhantomData<Rc<Cow<'a, str>>>, // For !send and !Sync.
    _pp: PhantomPinned,
    mem: [u8],
}

impl<'a> MainWindow<'a> {
    /// Create a new [MainWindow].
    ///
    /// This does not show the window.
    pub fn new(app: &'a App) -> Pin<Owned<Self>> {
        let _ = app;
        let data = Data {
            closed: Cell::new(false),
            waiter: Cell::new(None),
        };

        // Construct.
        let size = unsafe { qtx_main_window_size };
        let align = unsafe { qtx_main_window_align };
        let (layout, off) = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();
        let win = unsafe { qtx_main_window_new(layout.size(), layout.align()) };

        unsafe { std::ptr::write(win.add(off).cast(), data) };

        // Wrap.
        let v = unsafe { Owned::new(std::ptr::slice_from_raw_parts_mut(win, off) as *mut Self) };

        unsafe { Pin::new_unchecked(v) }
    }

    /// Show the window.
    #[inline(always)]
    pub fn show(&self) {
        unsafe { qtx_main_window_show(self.mem.as_ptr().cast_mut()) };
    }

    #[inline(always)]
    fn data(&self) -> &Data {
        let base = self.mem.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { &*data }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C-unwind" fn qtx_main_window_on_close(win: *mut u8) -> bool {
        let size = unsafe { qtx_main_window_size };
        let align = unsafe { qtx_main_window_align };
        let off = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::new::<Data>())
            .unwrap()
            .1;
        let data = unsafe { win.add(off).cast::<Data>().as_ref_unchecked() };

        data.closed.set(true);

        if let Some(w) = data.waiter.take() {
            w.wake();
        }

        true
    }
}

impl<'a> Drop for MainWindow<'a> {
    fn drop(&mut self) {
        let base = self.mem.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { std::ptr::drop_in_place::<Data>(data) };
        unsafe { qtx_main_window_destroy(base) };
    }
}

impl<'a> Future for MainWindow<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if closed.
        let data = self.data();

        if data.closed.get() {
            return Poll::Ready(());
        }

        // Set waker.
        data.waiter.set(Some(cx.waker().clone()));

        Poll::Pending
    }
}

struct Data {
    closed: Cell<bool>,
    waiter: Cell<Option<Waker>>,
}

unsafe extern "C-unwind" {
    static qtx_main_window_size: usize;
    static qtx_main_window_align: usize;

    fn qtx_main_window_new(size: usize, align: usize) -> *mut u8;
    fn qtx_main_window_destroy(win: *mut u8);
    fn qtx_main_window_show(win: *mut u8);
}
