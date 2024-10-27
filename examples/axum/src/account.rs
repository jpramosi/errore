use axum::Json;
use errore::prelude::*;
use serde::Deserialize;

/// Errors for account related operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Account with email '{}' is already registered", payload.email)]
    AlreadyRegistered { payload: Json<RegisterRequest> },
    #[error("Invalid email '{}'", payload.email)]
    InvalidEmail { payload: Json<RegisterRequest> },
    #[error("Password is too weak")]
    WeakPassword,
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

pub fn register(payload: Json<RegisterRequest>) -> Result<(), Ec> {
    if payload.password.len() < 6 {
        return err!(Error::WeakPassword);
    }
    if !payload.email.contains("@") {
        return err!(Error::InvalidEmail { payload });
    }
    err!(Error::AlreadyRegistered { payload })
}
