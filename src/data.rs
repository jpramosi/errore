use core::{
    fmt::{self},
    ops::{Deref, DerefMut},
};

use crate::hash::fnv1a_hash_64;

/// Represents an identifier hash.
#[derive(Clone, Copy, Default, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Id(u64);

impl Id {
    #[inline]
    pub fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn from_target(target: &'static str) -> Self {
        Self(fnv1a_hash_64(target.as_bytes()))
    }
}

impl PartialEq<i16> for Id {
    #[inline]
    fn eq(&self, other: &i16) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl PartialEq<i32> for Id {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl PartialEq<i64> for Id {
    #[inline]
    fn eq(&self, other: &i64) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl PartialEq<i128> for Id {
    #[inline]
    fn eq(&self, other: &i128) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl PartialEq<u16> for Id {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.0 == (*other).into()
    }
}

impl PartialEq<u32> for Id {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.0 == (*other).into()
    }
}

impl PartialEq<u64> for Id {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<u128> for Id {
    #[inline]
    fn eq(&self, other: &u128) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl PartialEq<usize> for Id {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        self.0 == (*other).try_into().unwrap_or_default()
    }
}

impl Deref for Id {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Id {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Id {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl From<i32> for Id {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value.try_into().unwrap_or_default())
    }
}

impl From<u64> for Id {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for Id {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<u16> for Id {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value.into())
    }
}

/// Defines the metadata for an error type.
///
/// <div class="warning">
/// This trait should not be implemented by the user.
/// </div>
pub trait Metadata {
    /// Returns the name with the following syntax:
    ///
    /// `<crate-name>::<module-name>::<enum-field-name|struct-name>`
    fn name(&self) -> &'static str;

    /// Returns an identifier hash derived from string:
    ///
    /// `<crate-name><module-path><enum-name+enum-field-name|struct-name>`
    fn id(&self) -> &'static Id;

    /// Returns a string describing the part of the system where the error
    /// that this metadata describes occurred.
    fn target(&self) -> &'static str;

    /// Returns an identifier describing the part of the system where the error
    /// that this metadata describes occurred.
    fn target_id(&self) -> &'static Id;

    /// Uses the formatter to apply the output of the implemented [`Display`](std::fmt::Display) trait.
    ///
    /// Usually this uses the implementation from the `#[error]` attribute macro.
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Returns `true` if the field forwards its [`Display`](std::fmt::Display) trait implementation to the source.
    fn is_transparent(&self) -> bool;
}

// These macros are used by the procedural macro `errore::error`.

#[cfg(all(feature = "ctor", not(miri)))]
#[doc(hidden)]
#[macro_export]
macro_rules! access_static_var {
    ($ctor_var:expr, $lazy_var:expr) => {
        &*$ctor_var
    };
}

#[cfg(any(not(feature = "ctor"), miri))]
#[doc(hidden)]
#[macro_export]
macro_rules! access_static_var {
    ($ctor_var:expr, $lazy_var:expr) => {
        &*$lazy_var
    };
}

#[cfg(all(feature = "ctor", not(miri)))]
#[doc(hidden)]
#[macro_export]
macro_rules! impl_static_var {
    ($ctor_var_name:ident, $lazy_var_name:ident, $var_type:ty, $body:block) => {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #[errore::__private::ctor]
        static $ctor_var_name: $var_type = $body;
    };
}

#[cfg(any(not(feature = "ctor"), miri))]
#[doc(hidden)]
#[macro_export]
macro_rules! impl_static_var {
    ($ctor_var_name:ident, $lazy_var_name:ident, $var_type:ty, $body:block) => {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $lazy_var_name: errore::__private::Lazy<$var_type> =
            errore::__private::Lazy::new(|| $body);
    };
}
