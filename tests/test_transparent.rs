use anyhow::anyhow;
use std::error::Error as _;
use std::io;

use errore::prelude::*;
use test_utils::*;

mod a {
    use super::*;

    #[derive(Error, Debug)]
    pub enum ErrorKind {
        #[error("E0")]
        E0,
        #[error("E1")]
        E1(#[from] io::Error),
    }
}

#[test]
fn test_transparent_struct() {
    #[derive(Error, Debug)]
    #[error(transparent)]
    struct Error(a::ErrorKind);

    let error = Error(a::ErrorKind::E0);
    assert_eq!("E0", error.to_string());
    assert!(error.source().is_none());
    let ec = Ec::new(error);
    assert_eq_text!(
        "
errore::test_transparent::Error: E0
    at tests/test_transparent.rs:29:14",
        ec.to_string()
    );

    let io = io::Error::new(io::ErrorKind::Other, "oh no!");
    let error = Error(a::ErrorKind::from(io));
    assert_eq!("E1", error.to_string());
    error.source().unwrap().downcast_ref::<io::Error>().unwrap();

    let ec = Ec::new(error);
    assert_eq_text!(
        "
errore::test_transparent::Error: E1
    at tests/test_transparent.rs:42:14",
        ec.to_string()
    );
}

#[test]
fn test_transparent_enum() {
    #[derive(Error, Debug)]
    enum Error {
        #[error("this failed")]
        This,
        #[error(transparent)]
        Other(anyhow::Error),
    }

    let error = Error::This;
    assert_eq!("this failed", error.to_string());
    let ec = Ec::new(error);
    assert_eq_text!(
        "
errore::test_transparent::This: this failed
    at tests/test_transparent.rs:63:14",
        ec.to_string()
    );

    let error = Error::Other(anyhow!("inner").context("outer"));
    assert_eq!("outer", error.to_string());
    assert_eq!("inner", error.source().unwrap().to_string());
    let ec = Ec::new(error);
    assert_eq_text!(
        "
errore::test_transparent::Other: outer
    at tests/test_transparent.rs:74:14",
        ec.to_string()
    );
}
