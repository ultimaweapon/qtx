//! Data reactive system.
pub(crate) use self::watch::*;

pub(self) use self::data::*;

use crate::string::Str;
use std::assert_matches;
use std::cell::RefCell;
use std::ptr::null_mut;
use std::rc::Rc;

mod data;
mod watch;

/// Provides notification when the value was changed.
pub struct Reactive<T>(Rc<RefCell<Data<T>>>);

impl<T> Reactive<T> {
    /// Create a new [Reactive] with `value` as initial value.
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(Data::new(value))))
    }

    /// Changes the value.
    ///
    /// # Panics
    /// If called from any subscribers.
    pub fn set(&self, v: impl Into<T>) {
        self.0.borrow_mut().set_value(v.into(), null_mut());
    }

    /// Subscribes for value changed event and return identifier for this subscription.
    ///
    /// The function can be removed by returns `false` from it or [Self::unsubscribe()].
    ///
    /// Note that call order is unspecified.
    ///
    /// # Panics
    /// If called from any subscribers.
    pub fn subscribe(&self, f: impl FnMut(&T) -> bool + 'static) -> usize {
        self.0
            .borrow_mut()
            .subscribe(Subscriber::Function(Box::new(f)))
    }

    /// Removes a function that was passed to [Self::subscribe()].
    ///
    /// The `id` will be reused by [Self::subscribe()] after this method returns.
    ///
    /// # Panics
    /// If `id` is not valid or called from any subscribers.
    pub fn unsubscribe(&self, id: usize) {
        assert_matches!(self.0.borrow_mut().unsubscribe(id), Subscriber::Function(_));
    }
}

impl<T: Clone> Reactive<T> {
    /// Returns current value.
    ///
    /// # Panics
    /// If called from any subscribers.
    pub fn get(&self) -> T {
        self.0.borrow().value().clone()
    }
}

impl<T> Clone for Reactive<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<&str> for Reactive<Str> {
    #[inline]
    fn from(value: &str) -> Self {
        Self::new(Str::from(value))
    }
}

impl<T> From<T> for Reactive<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
