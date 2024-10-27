extern crate alloc;

use core::error::Error;

use crate::downcast::Downcasted;

/// Marker trait for extractable errors.
///
/// The trait is usually implemented for a struct or enum that uses the derive procedural macro `errore::error`.
///
/// <div class="warning">
/// This trait should not be implemented by the user.
/// </div>
pub trait Extractable {}

/// Provides access to the trace context for an object.
///
/// <div class="warning">
/// This trait should not be implemented by the user.
/// </div>
pub trait Extract {
    /// Iterates the error chain and try to extract an error by its type.
    ///
    /// This function can only be used for error structs or enums that use the derive procedural macro `errore::error`.
    fn get<'a, E>(&'a self) -> Option<Downcasted<'a, E>>
    where
        E: Error + Extractable + 'static;

    /// Iterates the error chain and returns [`true`] if a type matches.
    ///
    /// This function can only be used for error structs or enums that use the derive procedural macro `errore::error`.
    fn has<'a, E>(&'a self) -> bool
    where
        E: Error + Extractable + 'static;
}
