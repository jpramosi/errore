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

<div class="hide-warning">

**auth.rs**
```rust ignore
use std::{fs, path::PathBuf};

// if 'errore::result::Result' is not needed, a simple wildcard import can be used:
// use errore::*;
use errore::prelude::*;

/// Errors for any failed authentication.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid email or password")]
    ReadPassword(#[from] std::io::Error),
    #[error("Invalid email or password")]
    InvalidCredentials,
}

// Automatically generated:
// pub struct Ec(pub Span<Error>)

fn read_password(email: &str) -> Result<String, Ec> {
    Ok(fs::read_to_string(PathBuf::from(email))?)
}

pub fn verify(email: &str, password: &str) -> Result<(), Ec> {
    if read_password(email)? != password {
        return err!(Error::InvalidCredentials);
    }
    Ok(())
}
```

</div>
<br>

<div class="hide-warning">

**account.rs**
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

<div class="hide-warning">

**main.rs**
```rust ignore
mod account;
mod auth;

use errore::prelude::*;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(ec) = account::login("root@errore.dev", "123") {
        // print formatted error chain
        println!("{}", ec.trace());

        // print trace records
        println!("\nTrace records:");
        for tr in &ec {
            println!("{}", tr);
        }

        // print the origin of the error
        // (the deepest 'Display' trait implementation will be used)
        println!("\nError display:\n{}", ec);

        // error extraction with 'match':
        // useful for handling multiple errors
        match ec.error() {
            account::Error::Authentication(ec) => match ec.error() {
                auth::Error::ReadPassword(error) => {
                    println!(
                        "\nError extraction with 'match':\nOS error code {}: {}",
                        error.raw_os_error().unwrap_or_default(),
                        error.kind()
                    )
                }
                _ => {}
            },
            _ => {}
        }

        // error extraction with 'get()':
        // useful for deeply nested errors
        if let Some(auth_error) = ec.get::<auth::Error>() {
            match &*auth_error {
                auth::Error::ReadPassword(error) => println!(
                    "\nError extraction with 'get()':\nOS error code {}: {}",
                    error.raw_os_error().unwrap_or_default(),
                    error.kind()
                ),
                _ => {}
            }
        }
    }
}
```

</div>
<br>

Examplary error output:

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

For more examples please see [here](https://github.com/jpramosi/errore/tree/master/examples).

<br>

# Features

- Tracing capability with rich metadata such as file location and line number without [`backtrace`](https://doc.rust-lang.org/std/backtrace/index.html)
- Generates trait implementations for [`metadata`](https://docs.rs/errore/latest/errore/trait.Metadata.html) and error conversion
- Customizable [`Subscriber`](https://github.com/jpramosi/errore/tree/master/examples/subscriber)
  and [`Formatter`](https://github.com/jpramosi/errore/tree/master/examples/formatter) interface
- Support for user attached data with [`Extensions`](https://docs.rs/errore/latest/errore/struct.ExtensionsMut.html) at subscriber
- Partial API compatibility with [`thiserror`](https://crates.io/crates/thiserror) that allows to optionally
  enable `errore` in public distributed libraries.
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
- For private libraries `errore` can be used _as is_. Errors are best declared on a per module basis.
  <br>See [Example](https://github.com/jpramosi/errore/tree/master/examples/basic)
- For general best-practices with `errore` the various [examples](https://github.com/jpramosi/errore/tree/master/examples)
  can serve as a good foundation

# Feature flags

- `ctor`: Utilizes *link_sections* provided by the [`ctor`](https://crates.io/crates/ctor) and [`inventory`](https://crates.io/crates/inventory)
   crates to offer a better implementation of the metadata and subscriber relevant code. The fallback implementation is based on lazy static variables.
   This feature can be disabled at `no-std` projects on build failures.
- `debug-no-std`: Enables internal debug logging with the [`defmt`](https://crates.io/crates/defmt) crate.
- `debug-std`: Enables internal debug logging with the [`log`](https://crates.io/crates/log) crate.
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
