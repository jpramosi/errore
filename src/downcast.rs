extern crate alloc;

use alloc::sync::Arc;
use core::error::Error;
use core::{fmt, marker::PhantomData, ops::Deref};

/// A construct to represent a reference to a downcasted error.
pub struct Downcasted<'a, T>
where
    T: Error + 'static,
{
    origin: Arc<dyn Error>,
    t: PhantomData<&'a T>,
}

impl<'a, T> fmt::Debug for Downcasted<'a, T>
where
    T: Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.origin, f)
    }
}

impl<'a, T> fmt::Display for Downcasted<'a, T>
where
    T: Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.origin, f)
    }
}

impl<'a, T> Downcasted<'a, T>
where
    T: Error + 'static,
{
    #[inline]
    pub(crate) fn new(origin: Arc<dyn Error>) -> Self {
        Self {
            origin,
            t: PhantomData,
        }
    }

    /// Returns a downcasted error reference.
    #[inline]
    fn downcast_ref(&self) -> &T {
        return self.origin.downcast_ref::<T>().expect(
            "Implementation of trait 'Extract' must check the value. Please open an issue.",
        );
    }
}

impl<'a, T> Deref for Downcasted<'a, T>
where
    T: Error + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.downcast_ref()
    }
}
