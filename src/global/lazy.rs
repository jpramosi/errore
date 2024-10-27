// This is the fallback implementation for the global module.
// https://github.com/rust-lang/miri/issues/450

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use conquer_once::spin::Lazy;
#[cfg(feature = "std")]
use conquer_once::Lazy;
use spin::{RwLock, RwLockReadGuard};

use crate::{
    formatter::{ErrorFormatter, Formatter},
    subscriber::Subscriber,
};

#[doc(hidden)]
pub static ERROR_SUBSCRIBERS: Lazy<RwLock<Vec<Box<dyn Subscriber>>>> =
    Lazy::new(|| RwLock::new(Vec::<Box<dyn Subscriber>>::with_capacity(10)));

#[doc(hidden)]
pub static ERROR_FORMATTER: Lazy<RwLock<Box<dyn Formatter + 'static>>> =
    Lazy::new(|| RwLock::new(Box::new(ErrorFormatter)));

#[doc(hidden)]
pub fn append_subscriber(subscriber: impl Subscriber + 'static) {
    ERROR_SUBSCRIBERS.write().push(Box::new(subscriber));
}

#[doc(hidden)]
pub fn set_formatter(formatter: impl Formatter + 'static) {
    *ERROR_FORMATTER.write() = Box::new(formatter);
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_subscriber {
    ($type:expr) => {
        $crate::global::append_subscriber($type);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_formatter {
    ($type:expr) => {
        $crate::global::set_formatter($type);
    };
}

#[doc(hidden)]
#[inline]
pub fn for_each_subscriber<F>(f: F)
where
    F: FnMut(&Box<dyn Subscriber>),
{
    ERROR_SUBSCRIBERS.read().iter().for_each(f);
}

#[doc(hidden)]
#[inline]
pub fn get_formatter() -> RwLockReadGuard<'static, Box<(dyn Formatter + 'static)>> {
    ERROR_FORMATTER.read()
}
