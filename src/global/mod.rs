#[cfg(all(feature = "ctor", not(miri)))]
mod ctor;
#[cfg(any(not(feature = "ctor"), miri))]
mod lazy;

#[cfg(all(feature = "ctor", not(miri)))]
pub use ctor::*;
#[cfg(any(not(feature = "ctor"), miri))]
pub use lazy::*;

/// Registers an error subscriber. Multiple subscribers can be submitted.
///
/// One subscriber can receive events even when it was registered in an external crate or dependency.
///
/// If the subscriber is to be used for logging or tracing purposes, for example,
/// it is recommended that only one subscriber be registered in the application code.
/// Library authors usually don't need this function.
/// 
/// This macro needs to be called within a function.
#[macro_export]
#[rustfmt::skip]
macro_rules! subscriber {
    ($type:expr) => {{ let _call_this_within_function = 0; }
        errore::__private::impl_subscriber!($type);
    };
}

/// Sets a global formatter for errors.
///
/// This macro needs to be called within a function.
/// 
/// <div class="warning">
/// This function should only be used in application code.
/// </div>
#[macro_export]
#[rustfmt::skip]
macro_rules! formatter {
    ($type:expr) => {{ let _call_this_within_function = 0; }
        errore::__private::impl_formatter!($type);
    };
}
