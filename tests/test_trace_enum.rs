use errore::prelude::*;
use test_utils::*;

#[test]
fn test_trace_from_error() {
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("...")]
        Field(#[from] std::io::Error),
    }

    let error = std::io::Error::new(std::io::ErrorKind::Other, "");
    let ec = Ec::from(error);
    let trace = ec.trace();

    assert_eq!(trace.len(), 1);
    assert_eq_text!(trace.last().location.file(), "tests/test_trace_enum.rs");
    assert_eq!(trace.last().location.line(), 13);
    assert_eq_file!(ec.to_string(), trace.to_string());
}

#[test]
fn test_trace_from_ec() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("...")]
                Field,
            }
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("...")]
                Field(#[from] a::Ec),
            }
        }
    }

    let ec = x::a::Ec::new(x::a::Error::Field);
    let trace = ec.trace();

    assert_eq!(trace.len(), 1);
    assert_eq_text!(trace.last().location.file(), "tests/test_trace_enum.rs");
    assert_eq!(trace.last().location.line(), 48);

    // trace record is only appended with 'Result'
    let ec = x::b::Ec::from(ec);
    assert_eq!(ec.trace().len(), 1);
    assert_eq_file!(ec.trace().to_string());
}

#[test]
fn test_trace_result() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("display-a")]
                Field,
            }

            pub fn func1() -> Result<(), Ec> {
                err!(Error::Field)
            }

            pub fn func2() -> Result<(), Ec> {
                func1()?;
                Ok(())
            }
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("display-b")]
                Field(#[from] a::Ec),
            }

            pub fn func1() -> Result<(), Ec> {
                a::func2()?;
                Ok(())
            }

            pub fn func2() -> Result<(), Ec> {
                func1()?;
                Ok(())
            }
        }
    }

    let ec = x::b::func2().unwrap_err();
    let trace = ec.trace().iter();

    assert_eq!(trace.len(), 4);
    assert_eq_text!(trace[0].location.file(), "tests/test_trace_enum.rs");
    assert_eq_file!(
        trace[0].to_string(),
        trace[1].to_string(),
        trace[2].to_string(),
        trace[3].to_string(),
        ec.trace().to_string()
    );
}
