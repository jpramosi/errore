use errore::prelude::*;

use crate::auth;

/// Errors for account related operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Authentication(#[from] auth::Ec),
    #[error("Submitted captcha '{hash}' is wrong")]
    WrongCaptcha { hash: String },
    #[error("Captcha session '{session}' was not found or is expired")]
    InvalidCaptcha { session: String },
}

// Automatically generated:
// pub struct Ec(pub Span<Error>)

pub fn login(email: &str, password: &str) -> Result<(), Ec> {
    auth::verify(email, password)?;
    // errors can also be defined without err!() macro
    Err(Ec::new(Error::WrongCaptcha {
        hash: "abc123".into(),
    }))
}
