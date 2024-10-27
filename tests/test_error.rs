#![allow(dead_code)]

use errore::prelude::*;

macro_rules! unimplemented_display {
    ($ty:ty) => {
        impl std::fmt::Display for $ty {
            fn fmt(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                unimplemented!()
            }
        }
    };
}

mod a {
    use super::*;

    #[derive(Error, Debug)]
    pub struct BracedError {
        msg: String,
        pos: usize,
    }
}

mod b {
    use super::*;

    #[derive(Error, Debug)]
    pub struct TupleError(String, usize);
}

mod c {
    use super::*;

    #[derive(Error, Debug)]
    pub struct UnitError;
}

mod d {
    use super::*;

    #[derive(Error, Debug)]
    pub struct WithSource {
        #[source]
        cause: std::io::Error,
    }
}

mod e {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumError {
        Braced {
            #[source]
            cause: std::io::Error,
        },
        Tuple(#[source] std::io::Error),
        Unit,
    }
}

unimplemented_display!(a::BracedError);
unimplemented_display!(b::TupleError);
unimplemented_display!(c::UnitError);
unimplemented_display!(d::WithSource);
unimplemented_display!(e::EnumError);
