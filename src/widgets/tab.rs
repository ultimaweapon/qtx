use super::{Container, Widget};
use std::alloc::Layout;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::rc::Rc;

/// Encapsulates a [QTabWidget](https://doc.qt.io/qt-6/qtabwidget.html).
pub struct Tab {
    _pd: PhantomData<Rc<()>>, // For !Send and !Sync.
    _pp: PhantomPinned,
    mem: [u8],
}

impl Tab {
    /// Create a new [Tab] on `parent`.
    pub fn new<P>(parent: &P) -> Pin<&Self>
    where
        P: Container + ?Sized,
    {
        let parent = parent.as_ptr();
        let data = Data { parent };

        // Construct.
        let size = unsafe { qtx_tab_size };
        let align = unsafe { qtx_tab_align };
        let (layout, off) = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();
        let w = unsafe { qtx_tab_new(parent, layout.size(), layout.align()) };

        unsafe { std::ptr::write(w.add(off).cast(), data) };

        // Wrap.
        let v = std::ptr::slice_from_raw_parts_mut(w, off) as *mut Self;

        unsafe { Pin::new_unchecked(&*v) }
    }

    #[inline(always)]
    fn data(&self) -> &Data {
        let base = self.mem.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { &*data }
    }
}

unsafe impl Widget for Tab {
    #[inline(always)]
    fn as_ptr(&self) -> *mut u8 {
        self.mem.as_ptr().cast_mut()
    }

    #[inline(always)]
    fn parent(&self) -> *mut u8 {
        self.data().parent
    }
}

struct Data {
    parent: *mut u8,
}

unsafe extern "C-unwind" {
    static qtx_tab_size: usize;
    static qtx_tab_align: usize;

    fn qtx_tab_new(parent: *mut u8, size: usize, align: usize) -> *mut u8;
}
