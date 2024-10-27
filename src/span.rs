extern crate alloc;

use alloc::string::ToString;
use alloc::sync::Arc;
use core::error::Error;
use core::marker::{Send, Sync};
use core::sync::atomic::AtomicBool;
use core::{fmt, sync::atomic::Ordering};

use crate::data::{Id, Metadata};
use crate::dlog;
use crate::downcast::Downcasted;
use crate::extensions::{Extensions, ExtensionsMut};
use crate::extract::{Extract, Extractable};
use crate::global::{for_each_subscriber, get_formatter};
use crate::trace::{TraceAccess, TraceContext, TraceRecord, TraceRecordIterator};

/// The `Span` represents a parent which a [`TraceRecord`] type is referring back to.
/// Multiple of these records can be assigned to a span.
///
/// A single span comprises a set of records propagated from one error type.
pub struct Span<T>
where
    T: Error + Metadata,
{
    format_span: Arc<AtomicBool>,
    #[doc(hidden)]
    /// The context that is moved to the last emitted span.
    pub ctx: Option<TraceContext>,
    /// The record that was created with the span.
    pub(crate) record: TraceRecord,
    #[doc(hidden)]
    /// The error type that was created with the span.
    pub inner: Arc<T>,
}

/// An utility struct for [`Subscriber::on_new_span`](../subscriber/trait.Subscriber.html#method.on_new_span)
/// that will be used to access the fields of a `Span`.
#[derive(Debug)]
pub struct SpanContext<'a> {
    ctx: &'a mut TraceContext,
    /// The associated record for this span.
    pub record: &'a TraceRecord,
}

impl<'a> SpanContext<'a> {
    #[doc(hidden)]
    #[inline]
    pub fn new(ctx: &'a mut TraceContext, record: &'a TraceRecord) -> Self {
        Self { ctx, record }
    }
}

impl<'a> fmt::Display for SpanContext<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.record, f)
    }
}

impl<'a> TraceAccess for SpanContext<'a> {
    #[inline]
    fn id(&self) -> Id {
        self.ctx.id()
    }

    #[inline]
    fn len(&self) -> usize {
        self.ctx.len()
    }

    #[inline]
    fn iter(&self) -> TraceRecordIterator {
        self.ctx.iter()
    }

    #[inline]
    fn extensions(&self) -> Extensions<'_> {
        self.ctx.extensions()
    }

    #[inline]
    fn extensions_mut(&mut self) -> ExtensionsMut<'_> {
        self.ctx.extensions_mut()
    }
}

impl<'s> Extract for SpanContext<'s> {
    #[inline]
    fn get<'a, E>(&'a self) -> Option<Downcasted<'a, E>>
    where
        E: Error + Extractable + 'static,
    {
        self.ctx.get::<E>()
    }

    #[inline]
    fn has<'a, E>(&'a self) -> bool
    where
        E: Error + Extractable + 'static,
    {
        self.ctx.has::<E>()
    }
}

impl<T> fmt::Debug for Span<T>
where
    T: Error + Metadata,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field(
                "records",
                &self.ctx.as_ref().map(|f| f.trace.len()).unwrap_or(0),
            )
            .field("location", &self.record.location.to_string())
            .field("name", &self.record.name)
            .field("target", &self.record.target)
            .field("target_id", &self.record.target_id)
            .field("id", &self.record.id)
            .field("is_transparent", &self.record.is_transparent)
            .finish()
    }
}

impl<T> Drop for Span<T>
where
    T: Error + Metadata,
{
    fn drop(&mut self) {
        // The error chain is going to be dropped with all its records.
        // Since only the last span owns the trace context in the error chain,
        // the handler will be only executed once.
        dlog!("{:#?}", self);
        if let Some(ctx) = self.ctx.as_mut() {
            dlog!("{}", self.record.id);

            // If the trace is taken with Traceable::take_trace(),
            // no handler will be called.
            // Even if the function is not public, it can still be called
            // by a user mistake.
            ctx.dropped.store(true, Ordering::Relaxed);

            // Execute handler as early as possible.
            for_each_subscriber(|s| s.on_end(ctx));
        }
    }
}

impl<T> Span<T>
where
    T: Error + Metadata + 'static + Send + Sync,
{
    #[doc(hidden)]
    #[track_caller]
    pub fn new(ctx: Option<TraceContext>, inner: T) -> Self {
        let inner_owned = Arc::new(inner);
        let inner_ref = Arc::downgrade(&inner_owned);

        let insert;
        let mut ctx = match ctx {
            Some(v) => {
                insert = false;
                v
            }
            None => {
                // With "fn from_residual(r: core::result::Result<Infallible, E>)"
                // the trace can look like it has duplicates, but is actually caused by two different
                // operations in the same function.
                // At the moment it is not required to keep this data around from the particular operation.
                // Instead an IndexSet is used for the trace to only allow unique items.
                insert = true;
                TraceContext::new()
            }
        };

        let record = TraceRecord {
            location: core::panic::Location::caller().into(),
            name: inner_owned.name(),
            target: inner_owned.target(),
            target_id: *inner_owned.target_id(),
            id: *inner_owned.id(),
            is_transparent: inner_owned.is_transparent(),
            inner: Some(inner_ref),
            format_span: ctx.format_span.clone(),
        };

        dlog!("span {:#?}", record);
        let mut span_ctx = SpanContext::new(&mut ctx, &record);
        if insert {
            for_each_subscriber(|s| s.on_start(&mut span_ctx));
        }
        for_each_subscriber(|s| s.on_new_span(&mut span_ctx));

        if insert {
            dlog!(
                "insert ({})\n\tin {}",
                span_ctx.record.name,
                span_ctx.record.location
            );
            for_each_subscriber(|s| s.on_try_record(&mut span_ctx));

            // keep in sync with impl/src/expand/error.rs
            if ctx.insert(record.clone()) {
                for_each_subscriber(|s| s.on_record(&mut ctx));
            }
        }

        Self {
            format_span: ctx.format_span.clone(),
            ctx: Some(ctx),
            record,
            inner: inner_owned,
        }
    }
}

impl<'a, T> IntoIterator for &'a Span<T>
where
    T: Error + Metadata + 'static,
{
    type Item = &'a TraceRecord;

    type IntoIter = TraceRecordIterator<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match &self.ctx {
            Some(ctx) => ctx.iter(),
            None => Self::IntoIter::default(),
        }
    }
}

impl<T> Metadata for Span<T>
where
    T: Error + Metadata + 'static,
{
    #[inline]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[inline]
    fn id(&self) -> &'static Id {
        self.inner.id()
    }

    #[inline]
    fn target(&self) -> &'static str {
        self.inner.target()
    }

    #[inline]
    fn target_id(&self) -> &'static Id {
        self.inner.target_id()
    }

    #[inline]
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.inner.is_transparent()
    }
}

impl<T> fmt::Display for Span<T>
where
    T: Error + Metadata + 'static,
{
    // This method has also an influence on the procedural macro generated struct 'Ec'.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.format_span.load(Ordering::Relaxed) {
            return self.display(f);
        }

        // Since only the last span owns the trace context in the error chain,
        // the handler will be only executed once.
        if let Some(ctx) = &self.ctx {
            dlog!(
                "format_span transparent={} Span<{}>",
                self.is_transparent() as i32,
                core::any::type_name::<T>()
            );
            get_formatter().format_span(self, ctx, f)
        } else {
            dlog!(
                "display     transparent={} Span<{}>",
                self.is_transparent() as i32,
                core::any::type_name::<T>()
            );
            self.display(f)
        }
    }
}

impl<T> Extract for Span<T>
where
    T: Error + Metadata + 'static,
{
    #[inline]
    fn get<'a, E>(&'a self) -> Option<Downcasted<'a, E>>
    where
        E: Error + Extractable + 'static,
    {
        self.ctx.as_ref().map(|f| f.get::<E>()).unwrap_or(None)
    }

    #[inline]
    fn has<'a, E>(&'a self) -> bool
    where
        E: Error + Extractable + 'static,
    {
        self.ctx.as_ref().map(|f| f.has::<E>()).unwrap_or(false)
    }
}

impl<T> Error for Span<T> where T: Error + Metadata + 'static {}
