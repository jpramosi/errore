use std::fmt;

use errore::{formatter::Formatter, Metadata, TraceContext, TraceRecord};

#[derive(Clone, Debug, Default)]
pub struct MyFormatter;

impl Formatter for MyFormatter {
    fn format_record(&self, rec: &TraceRecord, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> {} at {}",
            rec.name,
            rec.error_ref().map(|e| e.to_string()).unwrap_or_default(),
            rec.location
        )
    }

    fn format_span(
        &self,
        span: &(dyn Metadata + 'static),
        ctx: &TraceContext,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}: ", span.name())?;
        span.display(f)?;
        write!(f, "\n    at {}", ctx.first().location)
    }

    fn format_trace(&self, ctx: &TraceContext, f: &mut fmt::Formatter) -> fmt::Result {
        for rec in ctx {
            write!(f, "{}", rec)?;
        }
        Ok(())
    }
}

fn main() {
    errore::formatter!(MyFormatter);
}
