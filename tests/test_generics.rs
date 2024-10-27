use errore::prelude::*;

mod a {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumDebugGeneric<T>
    where
        T: std::error::Error + 'static + Send + Sync,
    {
        #[error("{0:?}")]
        FatalError(T),
    }
}

mod b {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumFromGeneric {
        #[error("enum from generic")]
        Source(#[from] a::Ec<std::io::Error>),
    }
}

mod c {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumDisplay<T>
    where
        T: std::error::Error + 'static + Send + Sync,
    {
        #[error("{0} {0:?}")]
        DisplayDebug(T),
    }
}

#[test]
fn test_display_enum_compound() {
    #[derive(Debug)]
    pub struct DebugAndDisplay;

    impl std::fmt::Display for DebugAndDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("debug and display")
        }
    }

    impl std::error::Error for DebugAndDisplay {}

    let instance: c::EnumDisplay<DebugAndDisplay>;

    instance = c::EnumDisplay::DisplayDebug(DebugAndDisplay);
    assert_eq!(format!("{}", instance), "debug and display DebugAndDisplay");
}

mod d {
    use super::*;

    #[derive(Error, Debug)]
    pub enum EnumTransparentGeneric<T>
    where
        T: std::error::Error + 'static + Send + Sync,
    {
        #[error(transparent)]
        Other(T),
    }
}

mod e {
    use super::*;

    #[derive(Error, Debug)]
    #[error("{underlying:?}")]
    pub struct StructDebugGeneric<T>
    where
        T: std::error::Error + 'static + Send + Sync,
    {
        pub underlying: T,
    }
}

mod f {
    use super::*;

    #[derive(Error, Debug)]
    #[error(transparent)]
    pub struct StructTransparentGeneric<T>(pub T)
    where
        T: std::error::Error + 'static + Send + Sync;
}
