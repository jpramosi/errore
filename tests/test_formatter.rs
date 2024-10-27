use std::fmt;

use errore::formatter::Formatter;
use errore::*;
use test_utils::*;

use crate::formatter::ErrorFormatter;

#[derive(Clone, Debug, Default)]
pub struct TestFormatter;

impl Formatter for TestFormatter {
    fn format_record(&self, rec: &errore::TraceRecord, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "format_record(): {}", rec.name)
    }

    fn format_span(
        &self,
        span: &(dyn errore::Metadata + 'static),
        ctx: &errore::TraceContext,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "format_span(): name:{} id:{}",
            ctx.last().name,
            span.id()
        )
    }

    fn format_trace(&self, ctx: &errore::TraceContext, f: &mut fmt::Formatter) -> fmt::Result {
        for tr in ctx.iter() {
            writeln!(f, "format_trace(): {}", tr)?;
        }
        Ok(())
    }
}

#[test]
fn test_register_formatter() {
    errore::formatter!(TestFormatter);

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct Error;

    let ec = Ec::new(Error);
    let trace = ec.trace();
    assert_eq_text!(
        trace.last().to_string(),
        "format_record(): errore::test_formatter::Error"
    );
    assert_eq_text!(
        ec.to_string(),
        "format_span(): name:errore::test_formatter::Error id:17724353069196222837"
    );
    assert_eq_text!(
        trace.to_string(),
        "format_trace(): format_record(): errore::test_formatter::Error"
    );

    // overwrite formatter
    errore::formatter!(ErrorFormatter);
}
