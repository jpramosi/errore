extern crate alloc;

use core::convert;
use core::convert::Infallible;
use core::error::Error;
use core::fmt;
use core::hint;
use core::iter::FusedIterator;
use core::ops::ControlFlow;
use core::ops::Deref;
use core::ops::DerefMut;
use core::ops::FromResidual;
use core::ops::Residual;
use core::ops::Try;

use crate::data::Metadata;
use crate::dlog;
use crate::trace::{TraceRecord, Traceable};

/// Shorthand macro for [`Result::Err`] in order to create an error context from a related enum or struct type.
#[macro_export]
macro_rules! err {
    ($enum_error:expr) => {
        Err(Ec::new($enum_error))
    };
}

/// `Result` is a type that represents either success ([`Ok`](Result::Ok)) or failure ([`Err`](Result::Err)).
///
/// On Failure it additionally records the error for later retrieval.
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Result<T, E> {
    /// Contains the success value
    Ok(T),
    /// Contains the error value
    Err(E),
}

impl<T, E, F> FromResidual<core::result::Result<Infallible, E>> for Result<T, F>
where
    E: fmt::Display + Error,
    F: From<E> + Error + Traceable + Metadata,
{
    #[track_caller]
    fn from_residual(r: core::result::Result<Infallible, E>) -> Self {
        match r {
            core::result::Result::Err(err) => {
                let mut e: F = err.into();
                let rec = TraceRecord::new(
                    &e,
                    e.trace_ref()
                        .expect("Trace must be available in 'FromResidual'"),
                );
                dlog!(
                    "insert ({}) to ({})\n\tin {}",
                    core::any::type_name::<E>(),
                    rec.name,
                    rec.location
                );
                e.insert(rec);
                Self::Err(e)
            }
        }
    }
}

impl<T, E, F> FromResidual<Result<!, E>> for Result<T, F>
where
    E: fmt::Display + Error + Traceable + Metadata,
    F: From<E> + Error + Traceable + Metadata,
{
    #[track_caller]
    fn from_residual(r: Result<!, E>) -> Self {
        match r {
            Result::Err(err) => {
                let _err_name = err.name();
                // 'take_trace()' is called within 'From' trait implementation
                let mut e: F = err.into();
                let rec = TraceRecord::new(
                    &e,
                    e.trace_ref()
                        .expect("Trace must be available in 'FromResidual'"),
                );
                dlog!(
                    "insert ({}) to ({})\n\tin {}",
                    _err_name,
                    rec.name,
                    rec.location
                );
                e.insert(rec);
                Self::Err(e)
            }
        }
    }
}

impl<T, E, F> FromResidual<Result<!, E>> for core::result::Result<T, F>
where
    E: fmt::Display + Error + Traceable + Metadata,
    F: From<E>,
{
    #[track_caller]
    fn from_residual(r: Result<!, E>) -> Self {
        match r {
            Result::Err(mut e) => {
                let mut rec = TraceRecord::new(
                    &e,
                    e.trace_ref()
                        .expect("Trace must be available in 'FromResidual'"),
                );
                rec.name = core::any::type_name::<F>();
                dlog!(
                    "insert ({}) to ({})\n\tin {}",
                    e.name(),
                    rec.name,
                    rec.location
                );
                e.insert(rec);
                Self::Err(e.into())
            }
        }
    }
}

impl<T, E> Try for Result<T, E>
where
    E: fmt::Display + Error + Traceable + Metadata,
{
    type Output = T;
    type Residual = Result<!, E>;

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(o) => ControlFlow::Continue(o),
            Self::Err(e) => ControlFlow::Break(Result::Err(e)),
        }
    }

    fn from_output(o: Self::Output) -> Self {
        Self::Ok(o)
    }
}

impl<T, E> Residual<T> for Result<!, E>
where
    E: fmt::Display + Error + Traceable + Metadata,
{
    type TryType = Result<T, E>;
}

impl<T, E> Result<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<i32, &str> = Ok(-3);
    /// assert_eq!(x.is_ok(), true);
    ///
    /// let x: Result<i32, &str> = Err("Some error message");
    /// assert_eq!(x.is_ok(), false);
    /// ```
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, Self::Ok(_))
    }

    /// Returns `true` if the result is [`Ok`] and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.is_ok_and(|x| x > 1), true);
    ///
    /// let x: Result<u32, &str> = Ok(0);
    /// assert_eq!(x.is_ok_and(|x| x > 1), false);
    ///
    /// let x: Result<u32, &str> = Err("hey");
    /// assert_eq!(x.is_ok_and(|x| x > 1), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_ok_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Self::Err(_) => false,
            Self::Ok(x) => f(x),
        }
    }

    /// Returns `true` if the result is [`Err`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<i32, &str> = Ok(-3);
    /// assert_eq!(x.is_err(), false);
    ///
    /// let x: Result<i32, &str> = Err("Some error message");
    /// assert_eq!(x.is_err(), true);
    /// ```
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Err`] and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// let x: Result<u32, Error> = Err(Error::new(ErrorKind::NotFound, "!"));
    /// assert_eq!(x.is_err_and(|x| x.kind() == ErrorKind::NotFound), true);
    ///
    /// let x: Result<u32, Error> = Err(Error::new(ErrorKind::PermissionDenied, "!"));
    /// assert_eq!(x.is_err_and(|x| x.kind() == ErrorKind::NotFound), false);
    ///
    /// let x: Result<u32, Error> = Ok(123);
    /// assert_eq!(x.is_err_and(|x| x.kind() == ErrorKind::NotFound), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_err_and(self, f: impl FnOnce(E) -> bool) -> bool {
        match self {
            Self::Ok(_) => false,
            Self::Err(e) => f(e),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Result<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Ok(x) => Some(x),
            Self::Err(_) => None,
        }
    }

    /// Converts from `Result<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.err(), None);
    ///
    /// let x: Result<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.err(), Some("Nothing here"));
    /// ```
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            Self::Ok(_) => None,
            Self::Err(x) => Some(x),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `&Result<T, E>` to `Result<&T, &E>`.
    ///
    /// Produces a new `Result`, containing a reference
    /// into the original, leaving the original in place.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.as_ref(), Ok(&2));
    ///
    /// let x: Result<u32, &str> = Err("Error");
    /// assert_eq!(x.as_ref(), Err(&"Error"));
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Result<&T, &E> {
        match *self {
            Self::Ok(ref x) => Result::<&T, &E>::Ok(x),
            Self::Err(ref x) => Result::<&T, &E>::Err(x),
        }
    }

    /// Converts from `&mut Result<T, E>` to `Result<&mut T, &mut E>`.
    ///
    /// # Examples
    ///
    /// ```
    /// fn mutate(r: &mut Result<i32, i32>) {
    ///     match r.as_mut() {
    ///         Ok(v) => *v = 42,
    ///         Err(e) => *e = 0,
    ///     }
    /// }
    ///
    /// let mut x: Result<i32, i32> = Ok(2);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap(), 42);
    ///
    /// let mut x: Result<i32, i32> = Err(13);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap_err(), 0);
    /// ```
    #[inline]
    pub const fn as_mut(&mut self) -> Result<&mut T, &mut E> {
        match *self {
            Self::Ok(ref mut x) => Result::<&mut T, &mut E>::Ok(x),
            Self::Err(ref mut x) => Result::<&mut T, &mut E>::Err(x),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////

    /// Maps a `Result<T, E>` to `Result<U, E>` by applying a function to a
    /// contained [`Ok`] value, leaving an [`Err`] value untouched.
    ///
    /// This function can be used to compose the results of two functions.
    ///
    /// # Examples
    ///
    /// Print the numbers on each line of a string multiplied by two.
    ///
    /// ```
    /// let line = "1\n2\n3\n4\n";
    ///
    /// for num in line.lines() {
    ///     match num.parse::<i32>().map(|i| i * 2) {
    ///         Ok(n) => println!("{n}"),
    ///         Err(..) => {}
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Result<U, E> {
        match self {
            Self::Ok(t) => Result::<U, E>::Ok(op(t)),
            Self::Err(e) => Result::<U, E>::Err(e),
        }
    }

    /// Returns the provided default (if [`Err`]), or
    /// applies a function to the contained value (if [`Ok`]).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`map_or_else`]: Result::map_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<_, &str> = Ok("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let x: Result<&str, _> = Err("bar");
    /// assert_eq!(x.map_or(42, |v| v.len()), 42);
    /// ```
    #[inline]
    #[must_use = "if you don't need the returned value, use `if let` instead"]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            Self::Ok(t) => f(t),
            Self::Err(_) => default,
        }
    }

    /// Maps a `Result<T, E>` to `U` by applying fallback function `default` to
    /// a contained [`Err`] value, or function `f` to a contained [`Ok`] value.
    ///
    /// This function can be used to unpack a successful result
    /// while handling an error.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let k = 21;
    ///
    /// let x : Result<_, &str> = Ok("foo");
    /// assert_eq!(x.map_or_else(|e| k * 2, |v| v.len()), 3);
    ///
    /// let x : Result<&str, _> = Err("bar");
    /// assert_eq!(x.map_or_else(|e| k * 2, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            Self::Ok(t) => f(t),
            Self::Err(e) => default(e),
        }
    }

    /// Maps a `Result<T, E>` to `Result<T, F>` by applying a function to a
    /// contained [`Err`] value, leaving an [`Ok`] value untouched.
    ///
    /// This function can be used to pass through a successful result while handling
    /// an error.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// fn stringify(x: u32) -> String { format!("error code: {x}") }
    ///
    /// let x: Result<u32, u32> = Ok(2);
    /// assert_eq!(x.map_err(stringify), Ok(2));
    ///
    /// let x: Result<u32, u32> = Err(13);
    /// assert_eq!(x.map_err(stringify), Err("error code: 13".to_string()));
    /// ```
    #[inline]
    pub fn map_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, F> {
        match self {
            Self::Ok(t) => Result::<T, F>::Ok(t),
            Self::Err(e) => Result::<T, F>::Err(op(e)),
        }
    }

    /// Calls a function with a reference to the contained value if [`Ok`].
    ///
    /// Returns the original result.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: u8 = "4"
    ///     .parse::<u8>()
    ///     .inspect(|x| println!("original: {x}"))
    ///     .map(|x| x.pow(3))
    ///     .expect("failed to parse number");
    /// ```
    #[inline]
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Self::Ok(ref t) = self {
            f(t);
        }

        self
    }

    /// Calls a function with a reference to the contained value if [`Err`].
    ///
    /// Returns the original result.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs, io};
    ///
    /// fn read() -> io::Result<String> {
    ///     fs::read_to_string("address.txt")
    ///         .inspect_err(|e| eprintln!("failed to read file: {e}"))
    /// }
    /// ```
    #[inline]
    pub fn inspect_err<F: FnOnce(&E)>(self, f: F) -> Self {
        if let Self::Err(ref e) = self {
            f(e);
        }

        self
    }

    /// Converts from `Result<T, E>` (or `&Result<T, E>`) to `Result<&<T as Deref>::Target, &E>`.
    ///
    /// Coerces the [`Ok`] variant of the original [`Result`] via [`Deref`]
    /// and returns the new [`Result`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<String, u32> = Ok("hello".to_string());
    /// let y: Result<&str, &u32> = Ok("hello");
    /// assert_eq!(x.as_deref(), y);
    ///
    /// let x: Result<String, u32> = Err(42);
    /// let y: Result<&str, &u32> = Err(&42);
    /// assert_eq!(x.as_deref(), y);
    /// ```
    #[inline]
    pub fn as_deref(&self) -> Result<&T::Target, &E>
    where
        T: Deref,
    {
        self.as_ref().map(|t| t.deref())
    }

    /// Converts from `Result<T, E>` (or `&mut Result<T, E>`) to `Result<&mut <T as DerefMut>::Target, &mut E>`.
    ///
    /// Coerces the [`Ok`] variant of the original [`Result`] via [`DerefMut`]
    /// and returns the new [`Result`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = "HELLO".to_string();
    /// let mut x: Result<String, u32> = Ok("hello".to_string());
    /// let y: Result<&mut str, &mut u32> = Ok(&mut s);
    /// assert_eq!(x.as_deref_mut().map(|x| { x.make_ascii_uppercase(); x }), y);
    ///
    /// let mut i = 42;
    /// let mut x: Result<String, u32> = Err(42);
    /// let y: Result<&mut str, &mut u32> = Err(&mut i);
    /// assert_eq!(x.as_deref_mut().map(|x| { x.make_ascii_uppercase(); x }), y);
    /// ```
    #[inline]
    pub fn as_deref_mut(&mut self) -> Result<&mut T::Target, &mut E>
    where
        T: DerefMut,
    {
        self.as_mut().map(|t| t.deref_mut())
    }

    /////////////////////////////////////////////////////////////////////////
    // Iterator constructors
    /////////////////////////////////////////////////////////////////////////

    /// Returns an iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the result is [`Result::Ok`], otherwise none.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(7);
    /// assert_eq!(x.iter().next(), Some(&7));
    ///
    /// let x: Result<u32, &str> = Err("nothing!");
    /// assert_eq!(x.iter().next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.as_ref().ok(),
        }
    }

    /// Returns a mutable iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the result is [`Result::Ok`], otherwise none.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: Result<u32, &str> = Ok(7);
    /// match x.iter_mut().next() {
    ///     Some(v) => *v = 40,
    ///     None => {},
    /// }
    /// assert_eq!(x, Ok(40));
    ///
    /// let mut x: Result<u32, &str> = Err("nothing!");
    /// assert_eq!(x.iter_mut().next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.as_mut().ok(),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Extract a value
    /////////////////////////////////////////////////////////////////////////

    /// Returns the contained [`Ok`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`Err`]
    /// case explicitly, or call [`unwrap_or`], [`unwrap_or_else`], or
    /// [`unwrap_or_default`].
    ///
    /// [`unwrap_or`]: Result::unwrap_or
    /// [`unwrap_or_else`]: Result::unwrap_or_else
    /// [`unwrap_or_default`]: Result::unwrap_or_default
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Err`], with a panic message including the
    /// passed message, and the content of the [`Err`].
    ///
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// x.expect("Testing expect"); // panics with `Testing expect: emergency failure`
    /// ```
    ///
    /// # Recommended Message Style
    ///
    /// We recommend that `expect` messages are used to describe the reason you
    /// _expect_ the `Result` should be `Ok`.
    ///
    /// ```should_panic
    /// let path = std::env::var("IMPORTANT_PATH")
    ///     .expect("env variable `IMPORTANT_PATH` should be set by `wrapper_script.sh`");
    /// ```
    ///
    /// **Hint**: If you're having trouble remembering how to phrase expect
    /// error messages remember to focus on the word "should" as in "env
    /// variable should be set by blah" or "the given binary should be available
    /// and executable by the current user".
    ///
    /// For more detail on expect message styles and the reasoning behind our recommendation please
    /// refer to the section on ["Common Message
    /// Styles"](../../std/error/index.html#common-message-styles) in the
    /// [`std::error`](../../std/error/index.html) module docs.
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Self::Ok(t) => t,
            Self::Err(e) => unwrap_failed(msg, &e),
        }
    }

    /// Returns the contained [`Ok`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`Err`]
    /// case explicitly, or call [`unwrap_or`], [`unwrap_or_else`], or
    /// [`unwrap_or_default`].
    ///
    /// [`unwrap_or`]: Result::unwrap_or
    /// [`unwrap_or_else`]: Result::unwrap_or_else
    /// [`unwrap_or_default`]: Result::unwrap_or_default
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Err`], with a panic message provided by the
    /// [`Err`]'s value.
    ///
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.unwrap(), 2);
    /// ```
    ///
    /// ```should_panic
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// x.unwrap(); // panics with `emergency failure`
    /// ```
    #[inline(always)]
    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Self::Ok(t) => t,
            Self::Err(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
        }
    }

    /// Returns the contained [`Ok`] value or a default
    ///
    /// Consumes the `self` argument then, if [`Ok`], returns the contained
    /// value, otherwise if [`Err`], returns the default value for that
    /// type.
    ///
    /// # Examples
    ///
    /// Converts a string to an integer, turning poorly-formed strings
    /// into 0 (the default value for integers). [`parse`] converts
    /// a string to any other type that implements [`FromStr`], returning an
    /// [`Err`] on error.
    ///
    /// ```
    /// let good_year_from_input = "1909";
    /// let bad_year_from_input = "190blarg";
    /// let good_year = good_year_from_input.parse().unwrap_or_default();
    /// let bad_year = bad_year_from_input.parse().unwrap_or_default();
    ///
    /// assert_eq!(1909, good_year);
    /// assert_eq!(0, bad_year);
    /// ```
    ///
    /// [`parse`]: str::parse
    /// [`FromStr`]: std::str::FromStr
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::Ok(x) => x,
            Self::Err(_) => Default::default(),
        }
    }

    /// Returns the contained [`Err`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Ok`], with a panic message including the
    /// passed message, and the content of the [`Ok`].
    ///
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// let x: Result<u32, &str> = Ok(10);
    /// x.expect_err("Testing expect_err"); // panics with `Testing expect_err: 10`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Self::Ok(t) => unwrap_failed(msg, &t),
            Self::Err(e) => e,
        }
    }

    /// Returns the contained [`Err`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Ok`], with a custom panic message provided
    /// by the [`Ok`]'s value.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// let x: Result<u32, &str> = Ok(2);
    /// x.unwrap_err(); // panics with `2`
    /// ```
    ///
    /// ```
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// assert_eq!(x.unwrap_err(), "emergency failure");
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap_err(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Self::Ok(t) => unwrap_failed("called `Result::unwrap_err()` on an `Ok` value", &t),
            Self::Err(e) => e,
        }
    }

    /// Returns the contained [`Ok`] value, but never panics.
    ///
    /// Unlike [`unwrap`], this method is known to never panic on the
    /// result types it is implemented for. Therefore, it can be used
    /// instead of `unwrap` as a maintainability safeguard that will fail
    /// to compile if the error type of the `Result` is later changed
    /// to an error that can actually occur.
    ///
    /// [`unwrap`]: Result::unwrap
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(never_type)]
    /// # #![feature(unwrap_infallible)]
    ///
    /// fn only_good_news() -> Result<String, !> {
    ///     Ok("this is fine".into())
    /// }
    ///
    /// let s: String = only_good_news().into_ok();
    /// println!("{s}");
    /// ```
    #[inline]
    pub fn into_ok(self) -> T
    where
        E: Into<!>,
    {
        match self {
            Self::Ok(x) => x,
            Self::Err(e) => e.into(),
        }
    }

    /// Returns the contained [`Err`] value, but never panics.
    ///
    /// Unlike [`unwrap_err`], this method is known to never panic on the
    /// result types it is implemented for. Therefore, it can be used
    /// instead of `unwrap_err` as a maintainability safeguard that will fail
    /// to compile if the ok type of the `Result` is later changed
    /// to a type that can actually occur.
    ///
    /// [`unwrap_err`]: Result::unwrap_err
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(never_type)]
    /// # #![feature(unwrap_infallible)]
    ///
    /// fn only_bad_news() -> Result<!, String> {
    ///     Err("Oops, it failed".into())
    /// }
    ///
    /// let error: String = only_bad_news().into_err();
    /// println!("{error}");
    /// ```
    #[inline]
    pub fn into_err(self) -> E
    where
        T: Into<!>,
    {
        match self {
            Self::Ok(x) => x.into(),
            Self::Err(e) => e,
        }
    }

    ////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////

    /// Returns `res` if the result is [`Ok`], otherwise returns the [`Err`] value of `self`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`and_then`], which is
    /// lazily evaluated.
    ///
    /// [`and_then`]: Result::and_then
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// let y: Result<&str, &str> = Err("late error");
    /// assert_eq!(x.and(y), Err("late error"));
    ///
    /// let x: Result<u32, &str> = Err("early error");
    /// let y: Result<&str, &str> = Ok("foo");
    /// assert_eq!(x.and(y), Err("early error"));
    ///
    /// let x: Result<u32, &str> = Err("not a 2");
    /// let y: Result<&str, &str> = Err("late error");
    /// assert_eq!(x.and(y), Err("not a 2"));
    ///
    /// let x: Result<u32, &str> = Ok(2);
    /// let y: Result<&str, &str> = Ok("different result type");
    /// assert_eq!(x.and(y), Ok("different result type"));
    /// ```
    #[inline]
    pub fn and<U>(self, res: Result<U, E>) -> Result<U, E> {
        match self {
            Self::Ok(_) => res,
            Self::Err(e) => Result::<U, E>::Err(e),
        }
    }

    /// Calls `op` if the result is [`Ok`], otherwise returns the [`Err`] value of `self`.
    ///
    ///
    /// This function can be used for control flow based on `Result` values.
    ///
    /// # Examples
    ///
    /// ```
    /// fn sq_then_to_string(x: u32) -> Result<String, &'static str> {
    ///     x.checked_mul(x).map(|sq| sq.to_string()).ok_or("overflowed")
    /// }
    ///
    /// assert_eq!(Ok(2).and_then(sq_then_to_string), Ok(4.to_string()));
    /// assert_eq!(Ok(1_000_000).and_then(sq_then_to_string), Err("overflowed"));
    /// assert_eq!(Err("not a number").and_then(sq_then_to_string), Err("not a number"));
    /// ```
    ///
    /// Often used to chain fallible operations that may return [`Err`].
    ///
    /// ```
    /// use std::{io::ErrorKind, path::Path};
    ///
    /// // Note: on Windows "/" maps to "C:\"
    /// let root_modified_time = Path::new("/").metadata().and_then(|md| md.modified());
    /// assert!(root_modified_time.is_ok());
    ///
    /// let should_fail = Path::new("/bad/path").metadata().and_then(|md| md.modified());
    /// assert!(should_fail.is_err());
    /// assert_eq!(should_fail.unwrap_err().kind(), ErrorKind::NotFound);
    /// ```
    #[inline]
    pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, op: F) -> Result<U, E> {
        match self {
            Self::Ok(t) => op(t),
            Self::Err(e) => Result::<U, E>::Err(e),
        }
    }

    /// Returns `res` if the result is [`Err`], otherwise returns the [`Ok`] value of `self`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: Result::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// let y: Result<u32, &str> = Err("late error");
    /// assert_eq!(x.or(y), Ok(2));
    ///
    /// let x: Result<u32, &str> = Err("early error");
    /// let y: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.or(y), Ok(2));
    ///
    /// let x: Result<u32, &str> = Err("not a 2");
    /// let y: Result<u32, &str> = Err("late error");
    /// assert_eq!(x.or(y), Err("late error"));
    ///
    /// let x: Result<u32, &str> = Ok(2);
    /// let y: Result<u32, &str> = Ok(100);
    /// assert_eq!(x.or(y), Ok(2));
    /// ```
    #[inline]
    pub fn or<F>(self, res: Result<T, F>) -> Result<T, F> {
        match self {
            Self::Ok(v) => Result::<T, F>::Ok(v),
            Self::Err(_) => res,
        }
    }

    /// Calls `op` if the result is [`Err`], otherwise returns the [`Ok`] value of `self`.
    ///
    /// This function can be used for control flow based on result values.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// fn sq(x: u32) -> Result<u32, u32> { Ok(x * x) }
    /// fn err(x: u32) -> Result<u32, u32> { Err(x) }
    ///
    /// assert_eq!(Ok(2).or_else(sq).or_else(sq), Ok(2));
    /// assert_eq!(Ok(2).or_else(err).or_else(sq), Ok(2));
    /// assert_eq!(Err(3).or_else(sq).or_else(err), Ok(9));
    /// assert_eq!(Err(3).or_else(err).or_else(err), Err(3));
    /// ```
    #[inline]
    pub fn or_else<F, O: FnOnce(E) -> Result<T, F>>(self, op: O) -> Result<T, F> {
        match self {
            Self::Ok(t) => Result::<T, F>::Ok(t),
            Self::Err(e) => op(e),
        }
    }

    /// Returns the contained [`Ok`] value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`unwrap_or_else`]: Result::unwrap_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let default = 2;
    /// let x: Result<u32, &str> = Ok(9);
    /// assert_eq!(x.unwrap_or(default), 9);
    ///
    /// let x: Result<u32, &str> = Err("error");
    /// assert_eq!(x.unwrap_or(default), default);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Ok(t) => t,
            Self::Err(_) => default,
        }
    }

    /// Returns the contained [`Ok`] value or computes it from a closure.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// fn count(x: &str) -> usize { x.len() }
    ///
    /// assert_eq!(Ok(2).unwrap_or_else(count), 2);
    /// assert_eq!(Err("foo").unwrap_or_else(count), 3);
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, op: F) -> T {
        match self {
            Self::Ok(t) => t,
            Self::Err(e) => op(e),
        }
    }

    /// Returns the contained [`Ok`] value, consuming the `self` value,
    /// without checking that the value is not an [`Err`].
    ///
    /// # Safety
    ///
    /// Calling this method on an [`Err`] is *[undefined behavior]*.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, 2);
    /// ```
    ///
    /// ```no_run
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// unsafe { x.unwrap_unchecked(); } // Undefined behavior!
    /// ```
    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_unchecked(self) -> T {
        debug_assert!(self.is_ok());
        match self {
            Self::Ok(t) => t,
            // SAFETY: the safety contract must be upheld by the caller.
            Self::Err(_) => unsafe { hint::unreachable_unchecked() },
        }
    }

    /// Returns the contained [`Err`] value, consuming the `self` value,
    /// without checking that the value is not an [`Ok`].
    ///
    /// # Safety
    ///
    /// Calling this method on an [`Ok`] is *[undefined behavior]*.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let x: Result<u32, &str> = Ok(2);
    /// unsafe { x.unwrap_err_unchecked() }; // Undefined behavior!
    /// ```
    ///
    /// ```
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// assert_eq!(unsafe { x.unwrap_err_unchecked() }, "emergency failure");
    /// ```
    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_err_unchecked(self) -> E {
        debug_assert!(self.is_err());
        match self {
            // SAFETY: the safety contract must be upheld by the caller.
            Self::Ok(_) => unsafe { hint::unreachable_unchecked() },
            Self::Err(e) => e,
        }
    }
}

impl<T, E> Result<&T, E> {
    /// Maps a `Result<&T, E>` to a `Result<T, E>` by copying the contents of the
    /// `Ok` part.
    ///
    /// # Examples
    ///
    /// ```
    /// let val = 12;
    /// let x: Result<&i32, i32> = Ok(&val);
    /// assert_eq!(x, Ok(&12));
    /// let copied = x.copied();
    /// assert_eq!(copied, Ok(12));
    /// ```
    #[inline]
    pub fn copied(self) -> Result<T, E>
    where
        T: Copy,
    {
        self.map(|&t| t)
    }

    /// Maps a `Result<&T, E>` to a `Result<T, E>` by cloning the contents of the
    /// `Ok` part.
    ///
    /// # Examples
    ///
    /// ```
    /// let val = 12;
    /// let x: Result<&i32, i32> = Ok(&val);
    /// assert_eq!(x, Ok(&12));
    /// let cloned = x.cloned();
    /// assert_eq!(cloned, Ok(12));
    /// ```
    #[inline]
    pub fn cloned(self) -> Result<T, E>
    where
        T: Clone,
    {
        self.map(|t| {
            let g = t.clone();
            g
        })
    }
}

impl<T, E> Result<&mut T, E> {
    /// Maps a `Result<&mut T, E>` to a `Result<T, E>` by copying the contents of the
    /// `Ok` part.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut val = 12;
    /// let x: Result<&mut i32, i32> = Ok(&mut val);
    /// assert_eq!(x, Ok(&mut 12));
    /// let copied = x.copied();
    /// assert_eq!(copied, Ok(12));
    /// ```
    #[inline]
    pub fn copied(self) -> Result<T, E>
    where
        T: Copy,
    {
        self.map(|&mut t| t)
    }

    /// Maps a `Result<&mut T, E>` to a `Result<T, E>` by cloning the contents of the
    /// `Ok` part.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut val = 12;
    /// let x: Result<&mut i32, i32> = Ok(&mut val);
    /// assert_eq!(x, Ok(&mut 12));
    /// let cloned = x.cloned();
    /// assert_eq!(cloned, Ok(12));
    /// ```
    #[inline]
    pub fn cloned(self) -> Result<T, E>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }
}

// This is a separate function to reduce the code size of the methods
#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

impl<T, E> Result<Option<T>, E> {
    /// Transposes a `Result` of an `Option` into an `Option` of a `Result`.
    ///
    /// `Ok(None)` will be mapped to `None`.
    /// `Ok(Some(_))` and `Err(_)` will be mapped to `Some(Ok(_))` and `Some(Err(_))`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Result<Option<i32>, SomeErr> = Ok(Some(5));
    /// let y: Option<Result<i32, SomeErr>> = Some(Ok(5));
    /// assert_eq!(x.transpose(), y);
    /// ```
    #[inline]
    pub fn transpose(self) -> Option<Result<T, E>> {
        match self {
            Result::<Option<T>, E>::Ok(Some(x)) => Some(Result::<T, E>::Ok(x)),
            Result::<Option<T>, E>::Ok(None) => None,
            Result::<Option<T>, E>::Err(e) => Some(Result::<T, E>::Err(e)),
        }
    }
}

impl<T, E> Result<Result<T, E>, E> {
    /// Converts from `Result<Result<T, E>, E>` to `Result<T, E>`
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(result_flattening)]
    /// let x: Result<Result<&'static str, u32>, u32> = Ok(Ok("hello"));
    /// assert_eq!(Ok("hello"), x.flatten());
    ///
    /// let x: Result<Result<&'static str, u32>, u32> = Ok(Err(6));
    /// assert_eq!(Err(6), x.flatten());
    ///
    /// let x: Result<Result<&'static str, u32>, u32> = Err(6);
    /// assert_eq!(Err(6), x.flatten());
    /// ```
    ///
    /// Flattening only removes one level of nesting at a time:
    ///
    /// ```
    /// #![feature(result_flattening)]
    /// let x: Result<Result<Result<&'static str, u32>, u32>, u32> = Ok(Ok(Ok("hello")));
    /// assert_eq!(Ok(Ok("hello")), x.flatten());
    /// assert_eq!(Ok("hello"), x.flatten().flatten());
    /// ```
    #[inline]
    pub fn flatten(self) -> Result<T, E> {
        self.and_then(convert::identity)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Trait implementations
/////////////////////////////////////////////////////////////////////////////

impl<T, E> Clone for Result<T, E>
where
    T: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Result::Ok(x) => Result::Ok(x.clone()),
            Result::Err(x) => Result::Err(x.clone()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Result::Ok(to), Result::Ok(from)) => to.clone_from(from),
            (Result::Err(to), Result::Err(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<T, E> IntoIterator for Result<T, E> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Returns a consuming iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the result is [`Result::Ok`], otherwise none.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(5);
    /// let v: Vec<u32> = x.into_iter().collect();
    /// assert_eq!(v, [5]);
    ///
    /// let x: Result<u32, &str> = Err("nothing!");
    /// let v: Vec<u32> = x.into_iter().collect();
    /// assert_eq!(v, []);
    /// ```
    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { inner: self.ok() }
    }
}

impl<'a, T, E> IntoIterator for &'a Result<T, E> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, E> IntoIterator for &'a mut Result<T, E> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

/////////////////////////////////////////////////////////////////////////////
// The Result Iterators
/////////////////////////////////////////////////////////////////////////////

/// An iterator over a reference to the [`Ok`] variant of a [`Result`].
///
/// The iterator yields one value if the result is [`Ok`], otherwise none.
///
/// Created by [`Result::iter`].
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    inner: Option<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.inner.take()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.inner.is_some() { 1 } else { 0 };
        (n, Some(n))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.inner.take()
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> Clone for Iter<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        Iter { inner: self.inner }
    }
}

/// An iterator over a mutable reference to the [`Ok`] variant of a [`Result`].
///
/// Created by [`Result::iter_mut`].
#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    inner: Option<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        self.inner.take()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.inner.is_some() { 1 } else { 0 };
        (n, Some(n))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> {
        self.inner.take()
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<T> FusedIterator for IterMut<'_, T> {}

/// An iterator over the value in a [`Ok`] variant of a [`Result`].
///
/// The iterator yields one value if the result is [`Ok`], otherwise none.
///
/// This struct is created by the [`into_iter`] method on
/// [`Result`] (provided by the [`IntoIterator`] trait).
///
/// [`into_iter`]: IntoIterator::into_iter
#[derive(Clone, Debug)]
pub struct IntoIter<T> {
    inner: Option<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.inner.take()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.inner.is_some() { 1 } else { 0 };
        (n, Some(n))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.inner.take()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}
