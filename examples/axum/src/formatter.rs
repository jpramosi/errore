use std::fmt;

use errore::{formatter::Formatter, Metadata, TraceContext};

/// A formatter that strips off any metadata of an error span.
#[derive(Clone, Debug, Default)]
pub struct ErrorResponseFormatter;

impl Formatter for ErrorResponseFormatter {
    fn format_span(
        &self,
        span: &(dyn Metadata + 'static),
        _ctx: &TraceContext,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        span.display(f)
    }
}
