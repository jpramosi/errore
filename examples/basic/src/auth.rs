use std::{fs, path::PathBuf};

// if 'error::result::Result' is not needed, a simple wildcard import can be used:
// use errore::*;
use errore::prelude::*;

/// Errors for any failed authentication.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid email or password")]
    ReadPassword(#[from] std::io::Error),
    #[error("Invalid email or password")]
    InvalidCredentials,
}

// Automatically generated:
// pub struct Ec(pub Span<Error>)

fn read_password(email: &str) -> Result<String, Ec> {
    Ok(fs::read_to_string(PathBuf::from(email))?)
}

pub fn verify(email: &str, password: &str) -> Result<(), Ec> {
    if read_password(email)? != password {
        return err!(Error::InvalidCredentials);
    }
    Ok(())
}
