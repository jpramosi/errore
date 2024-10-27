extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt::{self, Debug};
use core::marker::{Send, Sync};

use crate::data::{Id, Metadata};
use crate::trace::{TraceAccess, TraceContext, TraceRecord};

/// A handler to format [`TraceRecord`] and [`TraceContext`] types.
#[allow(unused_mut)]
#[allow(unused_variables)]
pub trait Formatter: Sync + Send {
    /// Formats a [`TraceRecord`] and passes it to the internal [`Display`](std::fmt::Display) trait implementation.
    fn format_record(&self, rec: &TraceRecord, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> {} at {}",
            rec.name,
            rec.error_ref().map(|e| e.to_string()).unwrap_or_default(),
            rec.location
        )
    }

    /// Formats span related data and passes it to the internal [`Display`](std::fmt::Display) trait implementation.
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

    /// Formats a [`TraceContext`] type and passes it to the internal [`Display`](std::fmt::Display) trait implementation.
    fn format_trace(&self, ctx: &TraceContext, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Default)]
        struct Fmt {
            node: bool,
            fmt_error: bool,
            last_node: bool,
        }

        // Map trace records with additional data.
        let mut last_id: Id = Id::default();
        let trace = ctx.iter();
        let mut nodes = trace
            .map(|tr| {
                let r = (
                    tr,
                    Fmt {
                        node: last_id != tr.id,
                        // Always format the first error and avoid duplicate messages,
                        // when using the transparent attribute.
                        fmt_error: last_id.is_null() || !tr.is_transparent,
                        last_node: false,
                    },
                );
                last_id = tr.id;
                r
            })
            .collect::<alloc::vec::Vec<(&TraceRecord, Fmt)>>();

        // Mark last node.
        for (_, f) in nodes.iter_mut().rev() {
            if f.node {
                f.last_node = true;
                break;
            }
        }

        // Write the last emitted error.
        // The error message is skipped because it is included in the trace itself.
        writeln!(f, "Error: {}", ctx.last().name)?;

        for i in 0..nodes.len() {
            let (tr, n) = nodes.get(i).unwrap();
            let next = nodes.get(i + 1);
            let is_last = i == nodes.len() - 1;
            let newline = if is_last { "" } else { "\n" };

            // Switch direction symbols.
            let lvl0_node;
            let lvl0_continue;
            let mut lvl1_node = "├";
            let mut lvl0_newline = "";
            if let Some(next) = next {
                if n.last_node {
                    lvl0_node = "╰";
                    lvl0_continue = " ";
                } else {
                    lvl0_node = "├";
                    lvl0_continue = "│";
                }
                if next.1.node {
                    lvl1_node = "╰";
                    lvl0_newline = "\n│";
                }
            } else {
                lvl0_node = "╰";
                lvl0_continue = " ";
                lvl1_node = "╰";
            }

            // Write head node on first error or when the span type changes.
            if n.node {
                let node_msg = format!("{}─▶ <{}>", lvl0_node, tr.name);
                write!(f, "{}", node_msg)?;

                if n.fmt_error {
                    let error_msg = match tr.error_ref() {
                        Some(v) => v.to_string(),
                        None => String::new(),
                    };

                    #[cfg(feature = "std")]
                    {
                        // Indent message approximately to a width of 80 characters.
                        let msg = textwrap::wrap(
                            &error_msg,
                            textwrap::Options::new(70).subsequent_indent(&format!(
                                "│   │{}",
                                " ".repeat(node_msg.len().checked_sub(10).unwrap_or(0))
                            )),
                        );

                        // Write wrapped lines.
                        write!(f, " ")?;
                        for m in msg {
                            write!(f, "{}\n", m)?;
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        write!(f, "{}\n", error_msg)?;
                    }
                } else {
                    // write!(f, " [transparent]\n")?;
                    write!(f, "\n")?;
                }
            }

            // Write location for every record.
            write!(
                f,
                "{}   {}╴ {}{}{}",
                lvl0_continue, lvl1_node, tr.location, lvl0_newline, newline
            )?;
        }

        fmt::Result::Ok(())
    }
}

/// Default error formatter.
#[derive(Clone, Debug, Default)]
pub struct ErrorFormatter;

impl Formatter for ErrorFormatter {}
