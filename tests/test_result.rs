use errore::prelude::*;
use test_utils::*;

/// impl<T, E, F> FromResidual<core::result::Result<Infallible, E>> for Result<T, F>
#[test]
fn test_from_residual_core_result_for_result() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error(#[from] std::io::Error);

    fn core_result() -> core::result::Result<(), std::io::Error> {
        core::result::Result::Err(std::io::Error::new(std::io::ErrorKind::Other, "..."))
    }

    fn result() -> Result<(), Ec> {
        Ok(core_result()?)
    }

    let ec = result().unwrap_err();
    assert_eq_text!(
        ec.to_string(),
        "
errore::test_result::Error: ...
    at tests/test_result.rs:16:12"
    );
}

/// impl<T, E, F> FromResidual<Result<!, E>> for Result<T, F>
#[test]
fn test_from_residual_never_result_for_result() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("Field")]
                Field,
            }

            pub fn result() -> Result<(), Ec> {
                Err(Ec::new(Error::Field))
            }
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            #[error("...")]
            pub struct Error(#[from] a::Ec);

            pub fn result() -> Result<(), Ec> {
                Ok(a::result()?)
            }
        }
    }

    let ec = x::b::result().unwrap_err();
    assert_eq_text!(
        ec.to_string(),
        "
errore::b::Error: ...
    at tests/test_result.rs:44:21"
    );
}

/// impl<T, E, F> FromResidual<Result<!, E>> for core::result::Result<T, F>
#[test]
fn test_from_residual_never_result_for_core_result() {
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Field")]
        Field,
    }

    pub fn result() -> Result<(), Ec> {
        Err(Ec::new(Error::Field))
    }

    pub fn core_result() -> core::result::Result<(), Ec> {
        result()?;
        core::result::Result::Ok(())
    }

    let ec = core_result().unwrap_err();
    assert_eq_text!(
        ec.to_string(),
        "
errore::test_result::Field: Field
    at tests/test_result.rs:80:13"
    );
}
