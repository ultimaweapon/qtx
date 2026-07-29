use std::alloc::Layout;
use std::cell::{Cell, RefCell};
use std::num::NonZero;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use rustc_hash::FxHashMap;

use crate::EXECUTOR;
use crate::ffi::Owned;

/// Encapsulates an instance of `Executor` class.
pub(crate) struct Executor([u8]);

impl Executor {
    /// # Safety
    /// This function can only called from the same thread that going run Qt event loop;
    pub unsafe fn new() -> Owned<Self> {
        let data = ExecutorData {
            tasks: RefCell::default(),
            recycle_ids: RefCell::default(),
            next_id: Cell::new(NonZero::<u32>::MIN),
        };

        // Get memory layout for Executor extended with ExecutorData.
        let size = unsafe { qtx_executor_size };
        let align = unsafe { qtx_executor_align };
        let (layout, off) = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();
        let exe = unsafe { qtx_executor_new(layout.size(), layout.align()) };

        unsafe { std::ptr::write(exe.add(off).cast(), data) };

        unsafe { Owned::new(std::ptr::slice_from_raw_parts_mut(exe, off) as *mut Executor) }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let data = self.data();
        let id = match data.recycle_ids.borrow_mut().pop() {
            Some(v) => v,
            None => {
                let v = data.next_id.get();

                data.next_id.set(v.checked_add(1).unwrap());

                v
            }
        };

        assert!(data.tasks.borrow_mut().insert(id, Box::pin(f)).is_none());

        // Schedule first poll.
        unsafe { qtx_executor_wake(self.0.as_ptr().cast_mut(), id.get()) };
    }

    #[inline(always)]
    fn data(&self) -> &ExecutorData {
        let base = self.0.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.0)).cast() };

        unsafe { &*data }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C-unwind" fn qtx_executor_poll(exe: *mut u8, id: u32) {
        // Takeout target task to poll.
        let size = unsafe { qtx_executor_size };
        let align = unsafe { qtx_executor_align };
        let off = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::new::<ExecutorData>())
            .unwrap()
            .1;
        let data = unsafe { exe.add(off).cast::<ExecutorData>().as_ref_unchecked() };
        let id = id.try_into().unwrap();
        let mut task = match data.tasks.borrow_mut().remove(&id) {
            Some(v) => v,
            None => return,
        };

        // Poll.
        let waker = Arc::new(AtomicU32::new(id.get()));
        let waker = unsafe { Waker::new(Arc::into_raw(waker).cast(), &WAKER) };

        match task.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Ready(_) => {
                drop(task);
                data.recycle_ids.borrow_mut().push(id);
            }
            Poll::Pending => assert!(data.tasks.borrow_mut().insert(id, task).is_none()),
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        let base = self.0.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.0)).cast() };

        unsafe { std::ptr::drop_in_place::<ExecutorData>(data) };
        unsafe { qtx_executor_destroy(base) };
    }
}

struct ExecutorData {
    tasks: RefCell<FxHashMap<NonZero<u32>, Pin<Box<dyn Future<Output = ()>>>>>,
    recycle_ids: RefCell<Vec<NonZero<u32>>>,
    next_id: Cell<NonZero<u32>>,
}

static WAKER: RawWakerVTable = RawWakerVTable::new(
    |w| unsafe {
        Arc::<AtomicU32>::increment_strong_count(w.cast());
        RawWaker::new(w, &WAKER)
    },
    |w| unsafe {
        // Check if waker already used.
        let w = Arc::<AtomicU32>::from_raw(w.cast());
        let t = w.swap(0, Ordering::Acquire);

        if t == 0 {
            return;
        }

        // Check if terminated.
        if let Some(e) = EXECUTOR.lock().unwrap().deref_mut() {
            qtx_executor_wake(e.deref_mut().0.as_mut_ptr(), t);
        }
    },
    |w| unsafe {
        // Check if waker already used.
        let w = w.cast::<AtomicU32>().as_ref_unchecked();
        let t = w.swap(0, Ordering::Acquire);

        if t == 0 {
            return;
        }

        // Check if terminated.
        if let Some(e) = EXECUTOR.lock().unwrap().deref_mut() {
            qtx_executor_wake(e.deref_mut().0.as_mut_ptr(), t);
        }
    },
    |w| unsafe { drop(Arc::<AtomicU32>::from_raw(w.cast())) },
);

unsafe extern "C-unwind" {
    static qtx_executor_size: usize;
    static qtx_executor_align: usize;

    fn qtx_executor_new(size: usize, align: usize) -> *mut u8;
    fn qtx_executor_destroy(exe: *mut u8);
    fn qtx_executor_wake(exe: *mut u8, task: u32);
}
