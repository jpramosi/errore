use core::{fmt, ops::Deref};

/// A struct containing information about the location of the caller.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Location(&'static core::panic::Location<'static>);

impl From<&'static core::panic::Location<'static>> for Location {
    #[inline]
    fn from(value: &'static core::panic::Location<'static>) -> Self {
        Self(value)
    }
}

impl fmt::Display for Location {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

impl Deref for Location {
    type Target = &'static core::panic::Location<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
