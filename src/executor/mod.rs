use std::alloc::Layout;
use std::cell::{Cell, RefCell};
use std::marker::PhantomPinned;
use std::num::NonZero;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

use rustc_hash::FxHashMap;

use crate::EXECUTOR;
use crate::mem::Owned;

/// Encapsulates an instance of `Executor` class.
pub(crate) struct Executor {
    _pp: PhantomPinned,
    mem: [u8],
}

impl Executor {
    /// # Safety
    /// This function can only called from the same thread that going run Qt event loop;
    pub unsafe fn new() -> Pin<Owned<Self>> {
        let data = Data {
            tasks: RefCell::default(),
            recycle_ids: RefCell::default(),
            next_id: Cell::new(NonZero::<u32>::MIN),
        };

        // Construct.
        let size = unsafe { qtx_executor_size };
        let align = unsafe { qtx_executor_align };
        let (layout, off) = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::for_value(&data))
            .unwrap();
        let layout = layout.pad_to_align();
        let exe = unsafe { qtx_executor_new(layout.size(), layout.align()) };

        unsafe { std::ptr::write(exe.add(off).cast(), data) };

        // Wrap.
        let v = std::ptr::slice_from_raw_parts_mut(exe, off) as *mut Executor;
        let v = unsafe { Owned::new(v) };

        unsafe { Pin::new_unchecked(v) }
    }

    /// # Safety
    /// This method can only called from the same thread that run Qt event loop.
    pub unsafe fn spawn<F>(&self, f: F)
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
        unsafe { qtx_executor_wake(self.mem.as_ptr().cast_mut(), id.get()) };
    }

    #[inline(always)]
    fn data(&self) -> &Data {
        let base = self.mem.as_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { &*data }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C-unwind" fn qtx_executor_poll(exe: *mut u8, id: u32) {
        // Takeout target task to poll.
        let size = unsafe { qtx_executor_size };
        let align = unsafe { qtx_executor_align };
        let off = Layout::from_size_align(size, align)
            .unwrap()
            .extend(Layout::new::<Data>())
            .unwrap()
            .1;
        let data = unsafe { exe.add(off).cast::<Data>().as_ref_unchecked() };
        let id = id.try_into().unwrap();
        let task = data.tasks.borrow_mut().remove(&id);
        let mut task = match task {
            Some(v) => v,
            None => {
                // The waker has been used on a finished task.
                data.recycle_ids.borrow_mut().push(id);
                return;
            }
        };

        // Poll.
        let wd = Arc::new(AtomicU32::new(id.get()));
        let waker = unsafe { Waker::new(Arc::into_raw(wd.clone()).cast(), &WAKER) };

        if task
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
        {
            assert!(data.tasks.borrow_mut().insert(id, task).is_none());
            return;
        }

        drop(task);

        // Check if waker has been used.
        if wd.swap(0, Ordering::Acquire) == 0 {
            // The waker has been used. We can't put task ID to recycle here otherwise we may wake a
            // wrong task. We will do it after we receive a wake up for this task instead (see above
            // code).
            return;
        }

        // The waker has not used yet. We are safely to put task ID to recycle here.
        data.recycle_ids.borrow_mut().push(id);
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        let base = self.mem.as_mut_ptr();
        let data = unsafe { base.add(size_of_val(&self.mem)).cast() };

        unsafe { std::ptr::drop_in_place::<Data>(data) };
        unsafe { qtx_executor_destroy(base) };
    }
}

struct Data {
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
            qtx_executor_wake(e.mem.as_ptr().cast_mut(), t);
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
            qtx_executor_wake(e.mem.as_ptr().cast_mut(), t);
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
