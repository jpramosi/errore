use std::path::PathBuf;
use std::sync::Once;

use errore::prelude::*;
use test_utils::*;

pub mod x {
    use super::*;

    pub mod a {
        use super::*;

        #[derive(Error, Debug)]
        pub enum Error {
            #[error("display-a")]
            Field,
        }

        pub fn func1(on: usize) -> Result<(), Ec> {
            if on == 1 {
                return err!(Error::Field);
            };
            err!(Error::Field)
        }

        pub fn func2(on: usize) -> Result<(), Ec> {
            if on == 2 {
                return err!(Error::Field);
            };
            func1(on)?;
            Ok(())
        }
    }

    pub mod b {
        use super::*;

        #[derive(Error, Debug, Default)]
        #[error("display-b")]
        pub struct Error {
            #[from]
            source: Option<a::Ec>,
        }

        pub fn func3(on: usize) -> Result<(), Ec> {
            if on == 3 {
                return err!(Error::default());
            };
            a::func2(on)?;
            Ok(())
        }

        pub fn func4(on: usize) -> Result<(), Ec> {
            if on == 4 {
                return err!(Error::default());
            };
            func3(on)?;
            Ok(())
        }
    }

    pub mod c {
        use super::*;

        #[derive(Error, Debug)]
        pub enum Error {
            #[error("display-c")]
            Field(#[from] Option<b::Ec>),
        }

        pub fn func5(on: usize) -> Result<(), Ec> {
            if on == 5 {
                return err!(Error::Field(None));
            };
            b::func4(on)?;
            Ok(())
        }

        pub fn func6(on: usize) -> Result<(), Ec> {
            if on == 6 {
                return err!(Error::Field(None));
            };
            func5(on)?;
            Ok(())
        }
    }

    pub mod d {
        use super::*;

        #[derive(Error, Debug, Default)]
        #[error("display-d")]
        pub struct Error {
            #[from]
            source: Option<c::Ec>,
        }

        pub fn func7(on: usize) -> Result<(), Ec> {
            if on == 7 {
                return err!(Error::default());
            };
            c::func6(on)?;
            Ok(())
        }

        pub fn func8(on: usize) -> Result<(), Ec> {
            if on == 8 {
                return err!(Error::default());
            };
            func7(on)?;
            Ok(())
        }
    }
}

static INIT: Once = Once::new();

pub fn init_subscriber() {
    INIT.call_once(|| {
        errore::subscriber!(TestSubscriber);
    });
}

fn init<'a>(on: usize, cb: &dyn Fn(&TraceContext)) -> (String, String) {
    init_subscriber();
    let ec_str;
    let ec_trace_str;

    // let trace context drop to catch also 'on_end()' event
    {
        let ec = x::d::func8(on).unwrap_err();
        let trace = ec.trace();
        ec_str = ec.to_string();
        ec_trace_str = trace.to_string();
        cb(&trace);
    }

    return (ec_str, ec_trace_str);
}

#[test]
fn test_trace_func0() {
    let (ec_str, ec_trace_str) = init(0, &|trace| {
        assert!(trace.has::<x::a::Error>());
        assert!(trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(trace.get::<x::a::Error>().is_some());
        assert!(trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 4);
    assert_eq!(data.on_try_record, 8);
    assert_eq!(data.on_record, 8);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func1() {
    let (ec_str, ec_trace_str) = init(1, &|trace| {
        assert!(trace.has::<x::a::Error>());
        assert!(trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(trace.get::<x::a::Error>().is_some());
        assert!(trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 4);
    assert_eq!(data.on_try_record, 8);
    assert_eq!(data.on_record, 8);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func2() {
    let (ec_str, ec_trace_str) = init(2, &|trace| {
        assert!(trace.has::<x::a::Error>());
        assert!(trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(trace.get::<x::a::Error>().is_some());
        assert!(trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 4);
    assert_eq!(data.on_try_record, 7);
    assert_eq!(data.on_record, 7);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func3() {
    let (ec_str, ec_trace_str) = init(3, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 3);
    assert_eq!(data.on_try_record, 6);
    assert_eq!(data.on_record, 6);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func4() {
    let (ec_str, ec_trace_str) = init(4, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 3);
    assert_eq!(data.on_try_record, 5);
    assert_eq!(data.on_record, 5);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func5() {
    let (ec_str, ec_trace_str) = init(5, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(!trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(!trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 2);
    assert_eq!(data.on_try_record, 4);
    assert_eq!(data.on_record, 4);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func6() {
    let (ec_str, ec_trace_str) = init(6, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(!trace.has::<x::b::Error>());
        assert!(trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(!trace.get::<x::b::Error>().is_some());
        assert!(trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 2);
    assert_eq!(data.on_try_record, 3);
    assert_eq!(data.on_record, 3);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func7() {
    let (ec_str, ec_trace_str) = init(7, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(!trace.has::<x::b::Error>());
        assert!(!trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(!trace.get::<x::b::Error>().is_some());
        assert!(!trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 1);
    assert_eq!(data.on_try_record, 2);
    assert_eq!(data.on_record, 2);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_func8() {
    let (ec_str, ec_trace_str) = init(8, &|trace| {
        assert!(!trace.has::<x::a::Error>());
        assert!(!trace.has::<x::b::Error>());
        assert!(!trace.has::<x::c::Error>());
        assert!(trace.has::<x::d::Error>());
        assert!(!trace.get::<x::a::Error>().is_some());
        assert!(!trace.get::<x::b::Error>().is_some());
        assert!(!trace.get::<x::c::Error>().is_some());
        assert!(trace.get::<x::d::Error>().is_some());
    });
    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 1);
    assert_eq!(data.on_try_record, 1);
    assert_eq!(data.on_record, 1);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}

#[test]
fn test_trace_iterator() {
    let (_, _) = init(0, &|trace| {
        let mut iterator = String::with_capacity(1024);
        let mut double_ended_iterator = String::with_capacity(1024);

        for rec in trace {
            iterator.push_str(&format!("{}\n", rec));
        }

        for rec in trace.iter().rev() {
            double_ended_iterator.push_str(&format!("{}\n", rec));
        }

        assert_eq_file!(iterator, double_ended_iterator);
    });
}

#[test]
fn test_trace_duplicate_location() {
    pub mod x {
        use super::*;

        pub mod a {
            use super::*;

            #[derive(Error, Debug)]
            #[error("...")]
            pub struct Error {
                pub path: PathBuf,
                pub source: std::io::Error,
            }

            pub fn read_to_string(path: PathBuf) -> Result<String, Ec> {
                Ok(std::fs::read_to_string(&path).map_err(|source| Error { path, source })?)
            }
        }

        pub mod b {
            use super::*;

            #[derive(Error, Debug)]
            pub enum Error {
                #[error("...")]
                Io(#[from] a::Ec),
            }

            pub fn func() -> Result<String, Ec> {
                Ok(a::read_to_string(PathBuf::from("/x"))?)
            }
        }
    }

    init_subscriber();
    let ec_str;
    let ec_trace_str;

    // let trace context drop to catch also 'on_end()' event
    {
        let ec = x::b::func().unwrap_err();
        let trace = ec.trace();
        ec_str = ec.to_string();
        ec_trace_str = trace.to_string();
    }

    let mdata = DATA.lock();
    let data = mdata.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(data.on_new_span, 2);
    assert_eq!(data.on_try_record, 3);
    assert_eq!(data.on_record, 2);
    assert_eq!(data.on_start, 1);
    assert_eq!(data.on_end, 1);
    assert_eq_file!(ec_str, ec_trace_str, data);
}
