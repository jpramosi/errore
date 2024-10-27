#[cfg(not(feature = "errore"))]
pub use core::{result::Result::Err, result::Result::Ok};
#[cfg(feature = "errore")]
pub use errore::{result::Result::Err, result::Result::Ok, *};
#[cfg(not(feature = "errore"))]
pub use thiserror::*;

#[cfg(feature = "errore")]
pub type Result<T, E> = errore::result::Result<T, E>;

#[cfg(not(feature = "errore"))]
pub type Result<T, E> = core::result::Result<T, E>;

#[cfg(feature = "errore")]
#[allow(unused_macros)]
#[macro_export]
macro_rules! err {
    ($enum_error:expr) => {
        errore::err!($enum_error)
    };
}

#[cfg(not(feature = "errore"))]
#[allow(unused_macros)]
#[macro_export]
macro_rules! err {
    ($enum_error:expr) => {
        Err($enum_error)
    };
}
