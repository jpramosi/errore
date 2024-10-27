#![cfg(feature = "std")]
use core::fmt::Display;
use std::path::{Path, PathBuf};

use errore::prelude::*;

mod a {
    use super::*;

    #[derive(Error, Debug)]
    #[error("failed to read '{file}'")]
    pub struct StructPathBuf {
        pub file: PathBuf,
    }
}

mod c {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumPathBuf {
        #[error("failed to read '{0}'")]
        Read(PathBuf),
    }
}

fn assert<T: Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_display() {
    let path = Path::new("/thiserror");
    let file = path.to_owned();
    assert("failed to read '/thiserror'", a::StructPathBuf { file });
    let file = path.to_owned();
    assert("failed to read '/thiserror'", c::EnumPathBuf::Read(file));
}
