#[doc(hidden)]
#[inline]
pub fn type_name_of<T>(_: T) -> &'static str {
    core::any::type_name::<T>()
}

// https://stackoverflow.com/questions/38088067/equivalent-of-func-or-function-in-rust
#[doc(hidden)]
#[macro_export]
macro_rules! function {
    () => {{
        use $crate::__private::type_name_of;
        fn f() {}

        type_name_of(f)
            .rsplit("::")
            .find(|&part| part != "f" && part != "{{closure}}")
            .expect("Short function name")
    }};
}

#[cfg(feature = "debug-std")]
#[doc(hidden)]
#[macro_export]
macro_rules! dlog {
    () => {
        dlog!("",)
    };
    ($fmt_string:expr) => {
        dlog!($fmt_string,)
    };
    ($fmt_string:expr, $( $arg:expr ),*) => {{
        #![allow(unused_imports)]
        use $crate::function;
        $crate::__private::log::debug!("[{}:{}] {}() {}", file!(), line!(), function!(), format_args!($fmt_string, $( $arg ),*))
    }};
}

#[cfg(feature = "debug-no-std")]
#[doc(hidden)]
#[macro_export]
macro_rules! dlog {
    () => {
        dlog!("",)
    };
    ($fmt_string:expr) => {
        dlog!($fmt_string,)
    };
    ($fmt_string:expr, $( $arg:expr ),*) => {{
        #![allow(unused_imports)]
        use $crate::function;
        $crate::__private::defmt::debug!("[{}:{}] {}() {}", file!(), line!(), function!(), format_args!($fmt_string, $( $arg ),*))
    }};
}

#[cfg(all(not(feature = "debug-std"), not(feature = "debug-no-std")))]
#[doc(hidden)]
#[macro_export]
macro_rules! dlog {
    () => {
        dlog!("",)
    };
    ($fmt_string:expr) => {
        dlog!($fmt_string,)
    };
    ($fmt_string:expr, $( $arg:expr ),*) => {{}};
}
