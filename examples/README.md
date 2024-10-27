## Examples of how to use Errore

This directory contains a collection of examples that demonstrate the use of the error framework:

- **[actix](https://github.com/jpramosi/errore/tree/master/examples/actix)** / **[axum](https://github.com/jpramosi/errore/tree/master/examples/axum)**:
    Demonstrates how to integrate errore in a backend server with a tokio runtime which consists of:
    + Using errore's `Result` type along with `std::result::Result` on routes and other asynchronous functions
    + Registration of a custom `Subscriber` and `Formatter`
    + Error inspection & filtering within a subscriber
    + Trait implementation to make errors compatible with responses
- **[basic](https://github.com/jpramosi/errore/tree/master/examples/basic)**:
    An example that shows the basic usage of errore:
    + Declaration of errors
    + Propagation of errors in functions
    + Error grouping & nesting
    + Various error inspection methods
- **[display](https://github.com/jpramosi/errore/tree/master/examples/display)**:
    Examplary code to show how to use the display derive macro
- **[formatter](https://github.com/jpramosi/errore/tree/master/examples/formatter)**:
    Shows the registration of a custom formatter
- **[optional](https://github.com/jpramosi/errore/tree/master/examples/optional)**:
    Demonstrates the _optional_ integration of errore in a public library crate with a feature gate
- **[subscriber](https://github.com/jpramosi/errore/tree/master/examples/subscriber)**:
    An example that illustrates the use of a `Subscriber` with `Extensions`
- **[tokio](https://github.com/jpramosi/errore/tree/master/examples/tokio)**:
    Demonstrates the use of different methods for error handling with tokio and errore
- **[tracing](https://github.com/jpramosi/errore/tree/master/examples/tracing)**:
    Shows how to integrate errore in an [OpenTelemetry](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/attributes-registry/exception.md) conform tracing application

<br>

All examples can be executed with:
```bash
# cargo run --package example-<NAME>
cargo run --package example-basic
# For debugging purposes, errore`s internal logging implementation can be activated with:
RUST_LOG=debug cargo run --package example-basic --features errore/debug-std
```
