use errore::prelude::*;
use test_utils::*;

#[test]
fn test_struct_metadata() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            #[error("...")]
            pub struct ErrorStruct;
        }
    }

    struct Format(x::a::Ec);

    impl std::fmt::Display for Format {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.display(f)
        }
    }

    let ec = x::a::Ec::new(x::a::ErrorStruct);
    assert_eq_text!(ec.name(), "errore::a::ErrorStruct");
    assert_eq!(*ec.id(), Id::from(5447241087302491405_u64));
    assert_eq_text!(ec.target(), "test_metadata");
    assert_eq!(*ec.target_id(), Id::from(1556350161995980347_u64));
    assert_eq!(*ec.target_id(), Id::from_target("test_metadata"));
    assert_eq_text!(
        Format(x::a::Ec::new(x::a::ErrorStruct)).to_string(),
        "
errore::a::ErrorStruct: ...
    at tests/test_metadata.rs:33:16"
    );
}

#[test]
fn test_struct_metadata_is_transparent() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            #[error("...")]
            pub struct ErrorStruct;
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            #[error(transparent)]
            pub struct ErrorTransparent(#[from] pub a::Ec);
        }
    }

    let ec = x::a::Ec::new(x::a::ErrorStruct);
    assert!(!ec.is_transparent());
    let ec = x::b::Ec::new(x::b::ErrorTransparent(ec));
    assert!(ec.is_transparent());
}

#[test]
fn test_enum_metadata() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            pub enum ErrorEnum {
                #[error("...")]
                Field,
            }
        }
    }

    struct Format(x::a::Ec);

    impl std::fmt::Display for Format {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.display(f)
        }
    }

    let ec = x::a::Ec::new(x::a::ErrorEnum::Field);
    assert_eq_text!(ec.name(), "errore::a::Field");
    assert_eq!(*ec.id(), Id::from(12062244763826042654_u64));
    assert_eq_text!(ec.target(), "test_metadata");
    assert_eq!(*ec.target_id(), Id::from(1556350161995980347_u64));
    assert_eq!(*ec.target_id(), Id::from_target("test_metadata"));
    assert_eq_text!(
        Format(x::a::Ec::new(x::a::ErrorEnum::Field)).to_string(),
        "
errore::a::Field: ...
    at tests/test_metadata.rs:99:16"
    );
}

#[test]
fn test_enum_metadata_is_transparent() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            pub enum ErrorEnum {
                #[error("...")]
                Field,
            }
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            pub enum ErrorTransparent {
                #[error(transparent)]
                Field(#[from] a::Ec),
            }
        }
    }

    let ec = x::a::Ec::new(x::a::ErrorEnum::Field);
    assert!(!ec.is_transparent());
    let ec = x::b::Ec::new(x::b::ErrorTransparent::Field(ec));
    assert!(ec.is_transparent());
}
