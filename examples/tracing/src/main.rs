use errore::{subscriber::Subscriber, Extract, Id, TraceContext};
use tracing::{error, info, trace, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod parser {
    use errore::prelude::*;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Tag is missing:\n'{line}'\n\t")]
        NoTag { line: String },
        #[error("No header was found")]
        NoHeader,
    }

    pub fn parse() -> Result<(), Ec> {
        err!(Error::NoTag {
            line: "<div>text<".into()
        })
    }
}

/// A minimal error subscriber that follows the OpenTelemetry
/// [specification](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/attributes-registry/error.md).
#[derive(Clone, Default, Debug)]
pub struct TracingSubscriber;

impl Subscriber for TracingSubscriber {
    fn on_end(&self, ctx: &mut TraceContext) {
        let err = ctx.last();

        // Optionally filter by target/crate with identifier (preferred way).
        if err.target_id != Id::from_target("example_tracing") {
            return;
        }

        // Log a simple error.
        info!(error.r#type = err.name, "{}", err);
    }
}

/// An error subscriber for debugging purposes that follows the OpenTelemetry
/// [specification](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/attributes-registry/exception.md).
#[derive(Clone, Default, Debug)]
pub struct TracingDebugSubscriber;

impl Subscriber for TracingDebugSubscriber {
    // Optionally log every occurrence or interaction with an error.
    // Most of the time the 'on_end()' handler should be enough.
    fn on_record(&self, ctx: &mut TraceContext) {
        let rec = ctx.last();

        // Optionally filter by target/crate with strings instead of an identifier.
        // Instead of string matching, use `target_id = Id::from_target()` whenever it is possible.
        if rec.target != "example_tracing" {
            return;
        }

        // Optionally handle specific error types.
        // If no error instance is needed, 'ctx.has()' can be used.
        if let Some(err) = ctx.get::<parser::Error>() {
            use parser::*;
            match &*err {
                Error::NoTag { line } => {
                    error!(error.r#type = rec.name, line = line, "{}", rec);
                }
                Error::NoHeader => {
                    warn!(error.r#type = rec.name, "{}", rec);
                }
            }
            return;
        }

        // Log a simple error.
        trace!(error.r#type = rec.name, "{}", rec);
    }

    fn on_end(&self, ctx: &mut TraceContext) {
        let rec = ctx.last();

        // Optionally filter by target/crate with identifier (preferred way).
        if rec.target_id != Id::from_target("example_tracing") {
            return;
        }

        // Log a detailed report.
        info!(
            error.r#type = rec.name,
            exception.message = rec.to_string(),
            exception.stacktrace = ctx.to_string(),
            exception.r#type = rec.name,
        );
    }
}

fn main() {
    // Looks pretty ugly with stdout, but on an observability frontend it's fine!
    // See also https://github.com/open-telemetry/opentelemetry-rust
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    errore::subscriber!(TracingDebugSubscriber);

    let _ = parser::parse();
}
