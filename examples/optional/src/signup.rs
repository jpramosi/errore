use crate::{err, result::*};

#[cfg(not(feature = "errore"))]
pub type Ec = Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid email '{0}'")]
    InvalidEmail(String),
    #[error("Password is too weak")]
    WeakPassword,
    #[error("Email '{0}' is already taken")]
    AlreadyRegistered(String),
}

fn verify_password(password: &str) -> Result<(), Ec> {
    if password.len() < 6 {
        return err!(Error::WeakPassword);
    }
    Ok(())
}

pub fn register(email: &str, password: &str) -> Result<(), Ec> {
    if !email.contains("@") {
        return err!(Error::InvalidEmail(email.into()));
    }
    if email == "root@errore.dev" {
        return err!(Error::AlreadyRegistered(email.into()));
    }
    verify_password(password)
}
