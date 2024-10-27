#![allow(clippy::needless_raw_string_hashes, clippy::uninlined_format_args)]

use errore::prelude::*;

fn assert<T: std::fmt::Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_braced() {
    #[derive(Display, Debug)]
    #[display("braced error: {msg}")]
    struct Display {
        msg: String,
    }

    let msg = "T".to_owned();
    assert("braced error: T", Display { msg: msg.clone() });
}

#[test]
fn test_braced_unused() {
    #[derive(Display, Debug)]
    #[display("braced error")]
    struct Display {
        extra: usize,
    }

    assert("braced error", Display { extra: 0 });
}

#[test]
fn test_tuple() {
    #[derive(Display, Debug)]
    #[display("tuple error: {0}")]
    struct Display(usize);

    assert("tuple error: 0", Display(0));
}

#[test]
fn test_unit() {
    #[derive(Display, Debug)]
    #[display("unit error")]
    struct Display;

    assert("unit error", Display);
}

#[test]
fn test_enum() {
    #[derive(Display, Debug)]
    enum Display {
        #[display("braced error: {id}")]
        Braced { id: usize },
        #[display("tuple error: {0}")]
        Tuple(usize),
        #[display("unit error")]
        Unit,
    }

    assert("braced error: 0", Display::Braced { id: 0 });
    assert("tuple error: 0", Display::Tuple(0));
    assert("unit error", Display::Unit);
}

#[test]
fn test_constants() {
    #[derive(Display, Debug)]
    #[display("{MSG}: {id:?} (code {CODE:?})")]
    struct Display {
        id: &'static str,
    }

    const MSG: &str = "failed to do";
    const CODE: usize = 9;

    assert("failed to do: \"\" (code 9)", Display { id: "" });
}

#[test]
fn test_inherit() {
    #[derive(Display, Debug)]
    #[display("{0}")]
    enum Display {
        Some(&'static str),
        #[display("other error")]
        Other(&'static str),
    }

    assert("some error", Display::Some("some error"));
    assert("other error", Display::Other("..."));
}

#[test]
fn test_brace_escape() {
    #[derive(Display, Debug)]
    #[display("fn main() {{}}")]
    struct Display;

    assert("fn main() {}", Display);
}

#[test]
fn test_expr() {
    #[derive(Display, Debug)]
    #[display("1 + 1 = {}", 1 + 1)]
    struct Display;
    assert("1 + 1 = 2", Display);
}

#[test]
fn test_nested() {
    #[derive(Display, Debug)]
    #[display("!bool = {}", not(.0))]
    struct Display(bool);

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn not(bool: &bool) -> bool {
        !*bool
    }

    assert("!bool = false", Display(true));
}

#[test]
fn test_match() {
    #[derive(Display, Debug)]
    #[display("{}: {0}", match .1 {
        Some(n) => format!("error occurred with {}", n),
        None => "there was an empty error".to_owned(),
    })]
    struct Display(String, Option<usize>);

    assert(
        "error occurred with 1: ...",
        Display("...".to_owned(), Some(1)),
    );
    assert(
        "there was an empty error: ...",
        Display("...".to_owned(), None),
    );
}

#[test]
fn test_nested_display() {
    // Same behavior as the one in `test_match`, but without String allocations.
    #[derive(Display, Debug)]
    #[display("{}", {
        struct Msg<'a>(&'a String, &'a Option<usize>);
        impl<'a> std::fmt::Display for Msg<'a> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self.1 {
                    Some(n) => write!(formatter, "error occurred with {}", n),
                    None => write!(formatter, "there was an empty error"),
                }?;
                write!(formatter, ": {}", self.0)
            }
        }
        Msg(.0, .1)
    })]
    struct Display(String, Option<usize>);

    assert(
        "error occurred with 1: ...",
        Display("...".to_owned(), Some(1)),
    );
    assert(
        "there was an empty error: ...",
        Display("...".to_owned(), None),
    );
}

#[test]
fn test_mixed() {
    #[derive(Display, Debug)]
    #[display("a={a} :: b={} :: c={c} :: d={d}", 1, c = 2, d = 3)]
    struct Display {
        a: usize,
        d: usize,
    }

    assert("a=0 :: b=1 :: c=2 :: d=3", Display { a: 0, d: 0 });
}

#[test]
fn test_ints() {
    #[derive(Display, Debug)]
    enum Display {
        #[display("error {0}")]
        Tuple(usize, usize),
        #[display("error {0}", '?')]
        Struct { v: usize },
    }

    assert("error 9", Display::Tuple(9, 0));
    assert("error ?", Display::Struct { v: 0 });
}

#[test]
fn test_trailing_comma() {
    #[derive(Display, Debug)]
    #[display(
        "error {0}",
    )]
    #[rustfmt::skip]
    struct Display(char);

    assert("error ?", Display('?'));
}

#[test]
fn test_field() {
    #[derive(Debug)]
    struct Inner {
        data: usize,
    }

    #[derive(Display, Debug)]
    #[display("{}", .0.data)]
    struct Display(Inner);

    assert("0", Display(Inner { data: 0 }));
}

#[test]
fn test_nested_tuple_field() {
    #[derive(Debug)]
    struct Inner(usize);

    #[derive(Display, Debug)]
    #[display("{}", .0.0)]
    struct Display(Inner);

    assert("0", Display(Inner(0)));
}

#[test]
fn test_macro_rules() {
    // Regression test for https://github.com/dtolnay/thiserror/issues/86

    macro_rules! decl_error {
        ($variant:ident($value:ident)) => {
            mod a {
                use super::*;

                #[derive(Debug, Display)]
                pub enum Display0 {
                    #[display("{0:?}")]
                    $variant($value),
                }
            }

            mod b {
                use super::*;

                #[derive(Debug, Display)]
                #[display("{0:?}")]
                pub enum Display1 {
                    $variant($value),
                }
            }
        };
    }

    decl_error!(Repro(u8));

    assert("0", a::Display0::Repro(0));
    assert("0", b::Display1::Repro(0));
}

#[test]
fn test_raw() {
    #[derive(Display, Debug)]
    #[display("braced raw error: {r#fn}")]
    struct Display {
        r#fn: &'static str,
    }

    assert("braced raw error: T", Display { r#fn: "T" });
}

#[test]
fn test_raw_enum() {
    #[derive(Display, Debug)]
    enum Display {
        #[display("braced raw error: {r#fn}")]
        Braced { r#fn: &'static str },
    }

    assert("braced raw error: T", Display::Braced { r#fn: "T" });
}

#[test]
fn test_raw_conflict() {
    #[derive(Display, Debug)]
    enum Display {
        #[display("braced raw error: {r#func}, {func}", func = "U")]
        Braced { r#func: &'static str },
    }

    assert("braced raw error: T, U", Display::Braced { r#func: "T" });
}

#[test]
fn test_keyword() {
    #[derive(Display, Debug)]
    #[display("error: {type}", type = 1)]
    struct Display;

    assert("error: 1", Display);
}

#[test]
fn test_str_special_chars() {
    #[derive(Display, Debug)]
    pub enum Display {
        #[display("brace left {{")]
        BraceLeft,
        #[display("brace left 2 \x7B\x7B")]
        BraceLeft2,
        #[display("brace left 3 \u{7B}\u{7B}")]
        BraceLeft3,
        #[display("brace right }}")]
        BraceRight,
        #[display("brace right 2 \x7D\x7D")]
        BraceRight2,
        #[display("brace right 3 \u{7D}\u{7D}")]
        BraceRight3,
        #[display(
            "new_\
line"
        )]
        NewLine,
        #[display("escape24 \u{78}")]
        Escape24,
    }

    assert("brace left {", Display::BraceLeft);
    assert("brace left 2 {", Display::BraceLeft2);
    assert("brace left 3 {", Display::BraceLeft3);
    assert("brace right }", Display::BraceRight);
    assert("brace right 2 }", Display::BraceRight2);
    assert("brace right 3 }", Display::BraceRight3);
    assert("new_line", Display::NewLine);
    assert("escape24 x", Display::Escape24);
}

#[test]
fn test_raw_str() {
    #[derive(Display, Debug)]
    pub enum Display {
        #[display(r#"raw brace left {{"#)]
        BraceLeft,
        #[display(r#"raw brace left 2 \x7B"#)]
        BraceLeft2,
        #[display(r#"raw brace right }}"#)]
        BraceRight,
        #[display(r#"raw brace right 2 \x7D"#)]
        BraceRight2,
    }

    assert(r#"raw brace left {"#, Display::BraceLeft);
    assert(r#"raw brace left 2 \x7B"#, Display::BraceLeft2);
    assert(r#"raw brace right }"#, Display::BraceRight);
    assert(r#"raw brace right 2 \x7D"#, Display::BraceRight2);
}

#[test]
fn test_display_fallback() {
    #[derive(Display, Debug)]
    enum Display {
        #[display("Field1")]
        Field1,
        Field2,
        #[display("Field3")]
        Field3,
    }

    assert("Field1", Display::Field1);
    assert("Display::Field2", Display::Field2);
    assert("Field3", Display::Field3);
}
