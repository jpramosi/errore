#![allow(
    clippy::module_name_repetitions,
    clippy::needless_lifetimes,
    clippy::return_self_not_must_use,
    clippy::wildcard_imports,
    stable_features
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_mut_refs)]
#![feature(error_in_core)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![doc = include_str!("lib.md")]
#![doc(html_root_url = "https://docs.rs/errore/")]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

mod aserror;
mod data;
mod display;
mod downcast;
mod extensions;
mod extract;
pub mod formatter;
pub mod global;
mod hash;
mod location;
mod logging;
pub mod result;
pub mod span;
pub mod subscriber;
mod trace;

pub use data::*;
pub use downcast::Downcasted;
pub use errore_impl::*;
pub use extensions::{Extensions, ExtensionsMut};
pub use extract::{Extract, Extractable};
pub use location::Location;
pub use trace::{TraceAccess, TraceContext, TraceRecord, Traceable};

pub mod prelude {
    pub use crate::{
        result::Result::{self, *},
        *,
    };
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use super::access_static_var;
    #[doc(hidden)]
    pub use super::impl_formatter;
    #[doc(hidden)]
    pub use super::impl_static_var;
    #[doc(hidden)]
    pub use super::impl_subscriber;
    #[doc(hidden)]
    pub use crate::aserror::AsDynError;
    #[doc(hidden)]
    pub use crate::display::AsDisplay;
    #[doc(hidden)]
    pub use crate::global::for_each_subscriber;
    #[doc(hidden)]
    pub use crate::global::get_formatter;
    #[doc(hidden)]
    pub use crate::hash::fnv1a_hash_64;
    #[doc(hidden)]
    pub use crate::logging::*;
    #[doc(hidden)]
    pub use crate::trace::TraceRecordIterator;
    #[doc(hidden)]
    #[cfg(all(not(feature = "std"), not(feature = "ctor")))]
    pub use conquer_once::spin::Lazy;
    #[doc(hidden)]
    #[cfg(all(feature = "std", any(not(feature = "ctor"), miri)))]
    pub use conquer_once::Lazy;
    #[doc(hidden)]
    #[cfg(feature = "ctor")]
    pub use ctor::ctor;
    #[doc(hidden)]
    #[cfg(feature = "debug-no-std")]
    pub use defmt;
    #[doc(hidden)]
    pub use inventory::submit;
    #[doc(hidden)]
    #[cfg(feature = "debug-std")]
    pub use log;
}
