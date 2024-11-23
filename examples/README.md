## Examples of how to use Errore

This directory contains a collection of examples that demonstrate the use of the error framework:

- **[actix](https://github.com/jpramosi/errore/tree/master/examples/actix/src)** / **[axum](https://github.com/jpramosi/errore/tree/master/examples/axum/src)**:
    Demonstrates how to integrate errore in a backend server with a tokio runtime which consists of:
    + Using errore's [`Result`](https://docs.rs/errore/latest/errore/result/enum.Result.html) type along with [`std::result::Result`](https://doc.rust-lang.org/std/result/) on routes and other asynchronous functions
    + Registration of a custom [`Subscriber`](https://docs.rs/errore/latest/errore/subscriber/trait.Subscriber.html) and [`Formatter`](https://docs.rs/errore/latest/errore/formatter/trait.Formatter.html)
    + Error inspection & filtering within a subscriber
    + Trait implementation to make errors compatible with responses
- **[basic](https://github.com/jpramosi/errore/tree/master/examples/basic/src)**:
    An example that shows the basic usage of errore:
    + Declaration of errors
    + Propagation of errors in functions
    + Error grouping & nesting
    + Various error inspection methods
- **[display](https://github.com/jpramosi/errore/blob/master/examples/display/src/main.rs)**:
    Examplary code to show how to use the display derive macro
- **[formatter](https://github.com/jpramosi/errore/blob/master/examples/formatter/src/main.rs)**:
    Shows the registration of a custom [`Formatter`](https://docs.rs/errore/latest/errore/formatter/trait.Formatter.html)
- **[optional](https://github.com/jpramosi/errore/tree/master/examples/optional)**:
    Demonstrates the _optional_ integration of errore in a public library crate with a feature gate
- **[subscriber](https://github.com/jpramosi/errore/blob/master/examples/subscriber/src/main.rs)**:
    An example that illustrates the use of a [`Subscriber`](https://docs.rs/errore/latest/errore/subscriber/trait.Subscriber.html) with [`Extensions`](https://docs.rs/errore/latest/errore/struct.ExtensionsMut.html)
- **[tokio](https://github.com/jpramosi/errore/tree/master/examples/tokio/src)**:
    Demonstrates the use of different methods for error handling with tokio and errore
- **[tracing](https://github.com/jpramosi/errore/blob/master/examples/tracing/src/main.rs)**:
    Shows how to integrate errore in an [OpenTelemetry](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/attributes-registry/exception.md) conform tracing application

<br>

All examples can be executed with:
```bash
# cargo run --package example-<NAME>
cargo run --package example-basic
# For debugging purposes, errore`s internal logging implementation can be activated with:
RUST_LOG=debug cargo run --package example-basic --features errore/debug-std
```
