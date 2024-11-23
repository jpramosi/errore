use core::fmt::Debug;
use core::marker::{Send, Sync};

use crate::span::SpanContext;
use crate::trace::{TraceContext, TraceContextBuilder, TraceRecord};

const TRACE_RESERVE: usize = 10;
const EXT_RESERVE: usize = 5;

/// A handler for error events.
///
/// A `Subscriber` implements a behavior for recording or collecting traces of
/// created, propagated or converted errors.
#[allow(unused_mut)]
#[allow(unused_variables)]
pub trait Subscriber: Sync + Send {
    /// Notifies this subscriber that the propagation of the error has been started.
    ///
    /// This handler is called before any other handler
    /// and is ideal for initializing data for a context.
    fn on_start(&self, builder: &mut TraceContextBuilder, rec: &TraceRecord) {
        builder
            .reserve_trace(TRACE_RESERVE)
            .reserve_extensions(EXT_RESERVE);
    }

    /// Notifies this subscriber that the propagation of the error has been completed
    /// and that no more trace records will be appended.
    fn on_end(&self, ctx: &mut TraceContext) {}

    /// Visits the construction of a new error [`Span`](crate::span::Span) instance.
    ///
    /// A new span is constructed if:
    /// - the error context is explicitly created
    /// - one span type is converted to another
    fn on_new_span(&self, ctx: &mut SpanContext) {}

    /// Visits the construction of an [`TraceRecord`] instance.
    ///
    /// This handler will be called before a record is inserted into the trace context.
    ///
    /// A new record is inserted if:
    /// - an error is created
    /// - a conversion from one error to another happens
    /// - the error is propagated to the caller
    ///
    /// It can also happen that multiple events call this handler for the same location,
    /// but perform different operations.
    fn on_try_record(&self, ctx: &mut SpanContext) {}

    /// Notifies this subscriber that a trace record has been verified and successfully recorded.
    fn on_record(&self, ctx: &mut TraceContext) {}
}

/// Default error subscriber.
#[derive(Clone, Debug, Default)]
pub struct ErrorSubscriber;

impl Subscriber for ErrorSubscriber {}
