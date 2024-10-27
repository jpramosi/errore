use errore::prelude::*;

fn assert_from_stderror<T: From<std::io::Error>>() {}

#[test]
fn test_from_struct() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error {
        #[from]
        source: std::io::Error,
    }

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_struct_optional() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error {
        #[from]
        source: Option<std::io::Error>,
    }

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_tuple() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error(#[from] std::io::Error);

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_tuple_optional() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error(#[from] Option<std::io::Error>);

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_enum() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub enum Error {
        Test {
            #[from]
            source: std::io::Error,
        },
    }

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_enum_optional() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub enum Error {
        Test {
            #[from]
            source: Option<std::io::Error>,
        },
    }

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}

#[test]
fn test_from_many() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub enum Error {
        Io(#[from] std::io::Error),
    }

    fn assert_from_error<T: From<Error>>() {}

    assert_from_stderror::<Error>();
    assert_from_stderror::<Ec>();
    assert_from_error::<Ec>();
}
