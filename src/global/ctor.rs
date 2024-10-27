// This is the default implementation for the global module.

extern crate alloc;

use crate::{
    formatter::{ErrorFormatter, Formatter},
    subscriber::Subscriber,
};

inventory::collect!(&'static dyn Subscriber);
inventory::collect!(&'static dyn Formatter);

#[doc(hidden)]
#[macro_export]
macro_rules! impl_subscriber {
    ($type:expr) => {
        $crate::__private::submit! {
            &$type as &dyn $crate::subscriber::Subscriber
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_formatter {
    ($type:expr) => {
        $crate::__private::submit! {
            &$type as &dyn $crate::formatter::Formatter
        }
    };
}

#[inline]
pub fn for_each_subscriber<F>(f: F)
where
    F: FnMut(&'static &dyn Subscriber),
{
    inventory::iter::<&'static dyn Subscriber>
        .into_iter()
        .for_each(f);
}

#[doc(hidden)]
#[inline]
pub fn get_formatter() -> &'static dyn Formatter {
    match inventory::iter::<&'static dyn Formatter>.into_iter().last() {
        Some(v) => *v,
        None => core::unreachable!("At least one error formatter must exist"),
    }
}

inventory::submit! {
    &ErrorFormatter as &dyn Formatter
}
