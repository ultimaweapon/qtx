use slab::Slab;
use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

pub struct Data<T> {
    value: T,
    subscribers: Slab<Subscriber<T>>,
}

impl<T> Data<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        Self {
            value,
            subscribers: Slab::new(),
        }
    }

    #[inline(always)]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn set_value(&mut self, val: T, src: *mut u8) {
        self.subscribers.retain(|_, s| match s {
            Subscriber::Object(o, f) => unsafe {
                if o.as_ptr().cast::<u8>() != src {
                    f(o.as_ref(), &val)
                } else {
                    true
                }
            },
            Subscriber::Function(f) => f(&val),
        });

        self.value = val;
    }

    #[inline]
    pub fn subscribe(&mut self, s: Subscriber<T>) -> usize {
        self.subscribers.insert(s)
    }

    #[inline]
    pub fn unsubscribe(&mut self, id: usize) -> Subscriber<T> {
        self.subscribers.remove(id)
    }
}

pub enum Subscriber<T> {
    Object(NonNull<[u8]>, fn(&[u8], &T) -> bool),
    Function(Box<dyn FnMut(&T) -> bool>),
}

impl<T> Debug for Subscriber<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(_, _) => f.write_str("Object"),
            Self::Function(_) => f.write_str("Function"),
        }
    }
}
