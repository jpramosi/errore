[![github]](https://github.com/jpramosi/errore)&ensp;[![crates-io]](https://crates.io/crates/errore)&ensp;[![docs-rs]](https://docs.rs/errore)&ensp;![nightly]

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[nightly]: https://img.shields.io/badge/nightly_+1.82.0-red?style=for-the-badge&labelColor=555555&logo=rust

<br>


This library provides a framework to handle and trace errors across modules and crates.

At the moment errore is in development and breaking changes are to be expected.

<br>

# Example

<br>

At first glance, errore its error definition looks quite similar to thiserror\`s:
<div class="hide-warning">

```rust ignore
use errore::prelude::*;

use crate::auth;

/// Errors for account related operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Authentication(#[from] auth::Ec),
    #[error("Submitted captcha '{hash}' is wrong")]
    WrongCaptcha { hash: String },
    #[error("Captcha session '{session}' was not found or is expired")]
    InvalidCaptcha { session: String },
}

// Automatically generated:
// pub struct Ec(pub Span<Error>)

pub fn login(email: &str, password: &str) -> Result<(), Ec> {
    auth::verify(email, password)?;
    // errors can also be defined without err!() macro
    Err(Ec::new(Error::WrongCaptcha {
        hash: "abc123".into(),
    }))
}
```
</div>
<br>

However, it is possible to extract additional information from the error, as shown in this sample error output:

```log
Error: example_basic::account::Authentication
├─▶ <example_basic::auth::ReadPassword> Invalid email or password
│   ├╴ examples/basic/src/auth.rs:20:8
│   ╰╴ examples/basic/src/auth.rs:24:8
│
╰─▶ <example_basic::account::Authentication>
    ╰╴ examples/basic/src/account.rs:20:5

Trace records:
<example_basic::auth::ReadPassword> Invalid email or password at examples/basic/src/auth.rs:20:8
<example_basic::auth::ReadPassword> Invalid email or password at examples/basic/src/auth.rs:24:8
<example_basic::account::Authentication> Invalid email or password at examples/basic/src/account.rs:20:5

Error display:
example_basic::account::Authentication: Invalid email or password
    at examples/basic/src/auth.rs:20:8

Error extraction with 'match':
OS error code 2: entity not found

Error extraction with 'get()':
OS error code 2: entity not found
```

The complete example can be seen [here](https://github.com/jpramosi/errore/tree/master/examples/basic).
For other examples please see [here](https://github.com/jpramosi/errore/tree/master/examples).

<br>

# Features

- [`Tracing`](https://docs.rs/errore/latest/errore/struct.TraceContext.html) capability with rich metadata such as file location and line number without [`backtrace`](https://doc.rust-lang.org/std/backtrace/index.html)
- Generates trait implementations for [`metadata`](https://docs.rs/errore/latest/errore/trait.Metadata.html) and error conversion
- Customizable [`Subscriber`](https://github.com/jpramosi/errore/tree/master/examples/subscriber)
  and [`Formatter`](https://github.com/jpramosi/errore/tree/master/examples/formatter) interface
- Support for user attached data with [`Extensions`](https://docs.rs/errore/latest/errore/struct.ExtensionsMut.html) at subscriber
- Partial API compatibility with [`thiserror`](https://crates.io/crates/thiserror) that allows to optionally
  enable `errore` in public distributed libraries on stable rust.
  <br>See [`example`](https://github.com/jpramosi/errore/tree/master/examples/optional)
- Usable in application and library code
- [`no-std`](https://github.com/jpramosi/errore/tree/master/tests/no-std) support & `wasm`compatible

# Limitations & Disadvantages

- Invasive code changes with [`Result`](https://docs.rs/errore/latest/errore/result/enum.Result.html) instrumentation are required
- [Nightly compiler](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust) is required
- Only one error per module can be defined
- No recursive or self-referencing fields
- Error conversion with attribute macro `#from` requires a trait implementation of `std::error::Error` for the type
- Generics with traits in error fields need to be declared with the `where` keyword
- Some edge cases cannot be expressed with generics (for e.g. nesting)
- No [`anyhow`](https://crates.io/crates/anyhow) support (shouldn't be a problem if `errore` is used)

# Recommendations

- For public libraries an optional feature flag for errore is advisable.
  For the best results [`thiserror`](https://crates.io/crates/thiserror) should be used.
  <br>See [Example](https://github.com/jpramosi/errore/tree/master/examples/optional)
- For private libraries `errore` can be used _as is_. Errors can be declared on a per module basis or as one global type.
  <br>See [Example](https://github.com/jpramosi/errore/tree/master/examples/basic)
- For general best-practices with `errore` the various [examples](https://github.com/jpramosi/errore/tree/master/examples)
  can serve as a good foundation

# Feature flags

- `ctor`: Utilizes *link_sections* provided by the [`ctor`](https://crates.io/crates/ctor) and [`inventory`](https://crates.io/crates/inventory)
   crates to offer a better implementation of the metadata and subscriber relevant code. The fallback implementation is based on lazy static variables.
   This feature can be disabled at `no-std` projects on build failures.
- `debug-no-std`: Enables internal logging with the [`defmt`](https://crates.io/crates/defmt) crate to debug `errore` itself.
- `debug-std`: Enables internal logging with the [`log`](https://crates.io/crates/log) crate to debug `errore` itself.
- `std`: Enables standard library support. If the `std` feature is not enabled, the `alloc` crate is required.

# Thanks to

- @dtolnay - Maintainer of several great crates including [`thiserror`](https://crates.io/crates/thiserror) which is used as errore`s foundation
- [tracing](https://crates.io/crates/tracing) / [error-stack](https://crates.io/crates/error-stack) / [error_set](https://crates.io/crates/error-set)
  maintainers & contributors for the inspiring codebase and ideas

<style>
.hide-warning .ignore.example-wrap {
    border-left: unset !important;
}

.hide-warning .tooltip {
    display: none !important;
}
</style>
