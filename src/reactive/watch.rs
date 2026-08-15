use super::{Reactive, Subscriber};
use std::cell::OnceCell;

pub struct Watch<T>(OnceCell<Data<T>>);

impl<T> Watch<T> {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }

    /// # Safety
    /// `obj` must outlive this [Watch].
    #[inline]
    pub unsafe fn get(
        &self,
        obj: &[u8],
        i: fn(&[u8]) -> T,
        f: fn(&[u8], &T) -> bool,
    ) -> &Reactive<T> {
        let d = self.0.get_or_init(move || {
            let src = Reactive::new(i(obj));
            let sub = src
                .0
                .borrow_mut()
                .subscribe(Subscriber::Object(obj.into(), f));

            Data { src, sub }
        });

        &d.src
    }

    pub fn set(&self, o: *mut u8, f: impl FnOnce() -> T) {
        let d = match self.0.get() {
            Some(v) => v,
            None => return,
        };

        if let Ok(mut d) = d.src.0.try_borrow_mut() {
            d.set_value(f(), o);
        };
    }

    /// # Safety
    /// `obj` must outlive this [Watch].
    ///
    /// # Panics
    /// If called from any subscribers of `src` or this watch already bound.
    pub unsafe fn subscribe(&self, src: Reactive<T>, obj: &[u8], f: fn(&[u8], &T) -> bool) {
        // Invoke handler for current value.
        let mut d = src.0.borrow_mut();

        f(obj, d.value());

        // Subscribe.
        let sub = d.subscribe(Subscriber::Object(obj.into(), f));

        drop(d);

        if self.0.set(Data { src, sub }).is_err() {
            panic!("attempt to subscribe a watch that already bound");
        }
    }
}

struct Data<T> {
    src: Reactive<T>,
    sub: usize,
}

impl<T> Drop for Data<T> {
    fn drop(&mut self) {
        self.src.0.borrow_mut().unsubscribe(self.sub);
    }
}
