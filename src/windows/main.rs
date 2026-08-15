use crate::App;
use crate::mem::Owned;
use crate::reactive::{Reactive, Watch};
use crate::string::Str;
use std::alloc::Layout;
use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::c_char;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

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
            title: Watch::new(),
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

    /// Gets [Reactive] for windows's title.
    pub fn title(&self) -> &Reactive<Str> {
        return unsafe { self.data().title.get(&self.mem, init, Self::update_title) };

        fn init(o: &[u8]) -> Str {
            unsafe { qtx_main_window_window_title(o.as_ptr().cast_mut()) }
        }
    }

    /// Sets windows's title.
    ///
    /// # Panics
    /// If called from any subscribers of `v` or after the title has been bound.
    pub fn set_title(&self, v: impl Into<Reactive<Str>>) {
        let d = self.data();

        unsafe { d.title.subscribe(v.into(), &self.mem, Self::update_title) };
    }

    fn update_title(o: &[u8], v: &Str) -> bool {
        let s = v.as_ptr().cast();
        let l = v.len().try_into().unwrap();

        unsafe { qtx_main_window_set_window_title(o.as_ptr().cast_mut(), s, l) };

        true
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
    unsafe extern "C-unwind" fn qtx_main_window_on_close(w: *mut u8) -> bool {
        let d = unsafe { Data::from_obj(w) };

        d.closed.set(true);

        if let Some(w) = d.waiter.take() {
            w.wake();
        }

        true
    }

    #[unsafe(no_mangle)]
    unsafe extern "C-unwind" fn qtx_main_window_on_window_title(
        w: *mut u8,
        s: *const u16,
        l: usize,
    ) {
        let d = unsafe { Data::from_obj(w) };

        d.title.set(w, move || unsafe {
            let s = std::slice::from_raw_parts(s, l);

            Str::from_utf16(s).unwrap()
        });
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
    title: Watch<Str>,
    closed: Cell<bool>,
    waiter: Cell<Option<Waker>>,
}

impl Data {
    unsafe fn from_obj<'a>(o: *const u8) -> &'a Self {
        let size = unsafe { qtx_main_window_size };
        let align = unsafe { qtx_main_window_align };
        let off = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::new::<Self>())
            .unwrap()
            .1;

        unsafe { o.add(off).cast::<Self>().as_ref_unchecked() }
    }
}

unsafe extern "C-unwind" {
    static qtx_main_window_size: usize;
    static qtx_main_window_align: usize;

    fn qtx_main_window_new(size: usize, align: usize) -> *mut u8;
    fn qtx_main_window_destroy(w: *mut u8);
    fn qtx_main_window_window_title(w: *mut u8) -> Str;
    fn qtx_main_window_set_window_title(w: *mut u8, s: *const c_char, l: isize);
    fn qtx_main_window_show(w: *mut u8);
}
