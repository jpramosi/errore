extern crate alloc;

use alloc::string::ToString;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::error::Error;
use core::hash::{Hash, Hasher};
use core::marker::{Send, Sync};
use core::ops::Index;
use core::sync::atomic::{self, AtomicBool, Ordering};
use core::{cmp, fmt};

#[cfg(feature = "std")]
type AtomicHash = portable_atomic::AtomicU64;

// Typically, the standard library is disabled on some embedded architectures.
// Some don't handle 64bit atomic types, so use a 32bit type here for general coverage.
#[cfg(not(feature = "std"))]
type AtomicHash = portable_atomic::AtomicU32;

use crate::data::{Id, Metadata};
use crate::dlog;
use crate::downcast::Downcasted;
use crate::extensions::{Extensions, ExtensionsInner, ExtensionsMut};
use crate::extract::{Extract, Extractable};
use crate::global::{for_each_subscriber, get_formatter};
use crate::location::Location;

const TRACE_RESERVE: usize = 32;
const EXT_RESERVE: usize = 10;

/// The record represents an entity where an error was created, propagated or
/// converted in the error chain.
#[derive(Clone)]
pub struct TraceRecord {
    /// The location of the error.
    pub location: Location,
    /// The name of the error derived from [`Metadata::name`].
    pub name: &'static str,
    /// The target of the error derived from [`Metadata::target`].
    pub target: &'static str,
    /// The target id of the error derived from [`Metadata::target_id`].
    pub target_id: Id,
    /// The id of the error derived from [`Metadata::id`].
    pub id: Id,
    /// Indicates whether the inherited inner error forwards its [`Display`](core::fmt::Display) implementation.
    pub is_transparent: bool,
    /// The inherited error.
    pub(crate) inner: Option<Weak<dyn Error + Send + Sync>>,
    /// Flag to switch between formatting methods.
    pub(crate) format_span: Arc<AtomicBool>,
}

impl fmt::Debug for TraceRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // dynamic traits of fields inner have no specific type here
        f.debug_struct("TraceRecord")
            .field("location", &self.location.to_string())
            .field("name", &self.name)
            .field("target", &self.target)
            .field("target_id", &self.target_id)
            .field("id", &self.id)
            .field("is_transparent", &self.is_transparent)
            .finish()
    }
}

impl fmt::Display for TraceRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_span.store(false, atomic::Ordering::Relaxed);
        let r = get_formatter().format_record(self, f);
        self.format_span.store(true, atomic::Ordering::Relaxed);
        r
    }
}

impl TraceRecord {
    #[track_caller]
    pub(crate) fn new<T>(error: &T, ctx: &TraceContext) -> Self
    where
        T: Metadata + Traceable,
    {
        Self {
            location: core::panic::Location::caller().into(),
            name: error.name(),
            target: error.target(),
            target_id: *error.target_id(),
            id: *error.id(),
            is_transparent: error.is_transparent(),
            inner: Some(Arc::downgrade(&error.inner())),
            format_span: ctx.format_span.clone(),
        }
    }

    /// Returns the inherited error.
    ///
    /// The error has the same lifetime as [`TraceContext`],
    /// so the function will return a valid value if the context still exists.
    #[inline]
    pub fn error_ref(&self) -> Option<Arc<dyn Error + Send + Sync>> {
        self.inner
            .as_ref()
            .expect("Weak reference upgrade should never fail. Please open an issue.")
            .upgrade()
    }
}

impl Hash for TraceRecord {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
        self.name.hash(state);
    }
}

impl Eq for TraceRecord {}

impl PartialEq for TraceRecord {
    #[inline]
    fn eq(&self, other: &TraceRecord) -> bool {
        self.location == other.location && self.name == other.name
    }
}

impl Ord for TraceRecord {
    #[inline]
    fn cmp(&self, other: &TraceRecord) -> cmp::Ordering {
        match Ord::cmp(&self.location, &other.location) {
            cmp::Ordering::Equal => Ord::cmp(&self.name, &other.name),
            cmp => cmp,
        }
    }
}

impl PartialOrd for TraceRecord {
    #[inline]
    fn partial_cmp(&self, other: &TraceRecord) -> Option<cmp::Ordering> {
        match PartialOrd::partial_cmp(&self.location, &other.location) {
            Option::Some(cmp::Ordering::Equal) => PartialOrd::partial_cmp(&self.name, &other.name),
            cmp => cmp,
        }
    }
}

/// A context for holding data used for error tracing.
///
/// This construct is always moved to the last emitted [`Span`](crate::span::Span).
pub struct TraceContext {
    /// The collected trace records emitted by creation, conversion or propagation of errors.
    pub(crate) trace: Vec<TraceRecord>,
    /// Track last trace record to avoid subsequent duplicates.
    pub(crate) last_record: Arc<AtomicHash>,
    /// Flag to switch between formatting methods.
    pub(crate) format_span: Arc<AtomicBool>,
    /// A flag which indicates whether the first `Span` is dropped.
    pub(crate) dropped: Arc<AtomicBool>,
    /// User attached data container.
    pub(crate) extensions: ExtensionsInner,
}

impl fmt::Debug for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TraceContext")
            .field("trace", &self.trace)
            .field("format_span", &self.format_span)
            .finish()
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.trace.is_empty() {
            return write!(f, "TraceContext empty");
        }
        self.format_span.store(false, atomic::Ordering::Relaxed);
        let r = get_formatter().format_trace(self, f);
        self.format_span.store(true, atomic::Ordering::Relaxed);
        r
    }
}

impl Drop for TraceContext {
    fn drop(&mut self) {
        dlog!("TraceContext");
        // This will just serve as a fallback.
        // See Span::drop for more details.
        if !self.dropped.load(Ordering::Relaxed) {
            for_each_subscriber(|s| s.on_end(self));
        }
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct TraceRecordIterator<'a> {
    index: Option<&'a Vec<TraceRecord>>,
    pos: usize,
    len: usize,
}

impl<'a> TraceRecordIterator<'a> {
    /// Returns the number of elements in the trace context, also referred to
    /// as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        match self.index {
            Some(index) => index.len(),
            None => 0,
        }
    }
}

impl<'a> Iterator for TraceRecordIterator<'a> {
    type Item = &'a TraceRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        match self.index {
            Some(index) => index.get(pos),
            None => None,
        }
    }
}

impl<'a> DoubleEndedIterator for TraceRecordIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        if self.pos > self.len {
            return None;
        }
        match self.index {
            Some(index) => index.get(self.len - self.pos),
            None => None,
        }
    }
}

impl<'a> Index<usize> for TraceRecordIterator<'a> {
    type Output = TraceRecord;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.index.unwrap().get(index).unwrap()
    }
}

impl<'a> IntoIterator for &'a TraceContext {
    type Item = &'a TraceRecord;

    type IntoIter = TraceRecordIterator<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl TraceContext {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            trace: Vec::with_capacity(TRACE_RESERVE),
            last_record: Arc::new(AtomicHash::new(0)),
            format_span: Arc::new(AtomicBool::new(true)),
            dropped: Arc::new(AtomicBool::new(false)),
            extensions: ExtensionsInner::with_capacity(EXT_RESERVE),
        }
    }

    /// Gets the origin error.
    #[inline]
    pub fn first(&self) -> &TraceRecord {
        self.trace
            .first()
            .expect("Context must have at least one or more records. Please open an issue.")
    }

    /// Gets the top-level error.
    #[inline]
    pub fn last(&self) -> &TraceRecord {
        self.trace
            .last()
            .expect("Context must have at least one or more records. Please open an issue.")
    }

    #[doc(hidden)]
    #[inline]
    pub fn insert(&mut self, record: TraceRecord) -> bool {
        #[cfg(feature = "std")]
        let hash = {
            let mut hasher = std::hash::DefaultHasher::new();
            record.hash(&mut hasher);
            hasher.finish()
        };

        #[cfg(not(feature = "std"))]
        let hash = {
            use hash32::Hasher;
            let mut hasher = hash32::FnvHasher::default();
            record.hash(&mut hasher);
            hasher.finish32()
        };

        if self.last_record.swap(hash, Ordering::Relaxed) != hash {
            self.trace.push(record);
            return true;
        }

        return false;
    }
}

/// Interface to interact with the collected traces.
pub trait TraceAccess {
    /// Gets the instance id of this context.
    fn id(&self) -> Id;

    /// Returns the number of elements in the trace context, also referred to
    /// as its 'length'.
    fn len(&self) -> usize;

    /// Returns an iterator over the slice of the trace records.
    ///
    /// The iterator yields all items from start to end.
    fn iter(&self) -> TraceRecordIterator;

    /// Returns a reference to this context's `Extensions`.
    ///
    /// The extensions may be used by the subscriber to store additional data
    /// describing the context.
    fn extensions(&self) -> Extensions<'_>;

    /// Returns a mutable reference to this context's `Extensions`.
    ///
    /// The extensions may be used by the subscriber to store additional data
    /// describing the context.
    fn extensions_mut(&mut self) -> ExtensionsMut<'_>;
}

impl TraceAccess for TraceContext {
    #[inline]
    fn id(&self) -> Id {
        return (core::ptr::addr_of!(self) as u64).into();
    }

    #[inline]
    fn len(&self) -> usize {
        self.trace.len()
    }

    #[inline]
    fn iter(&self) -> TraceRecordIterator {
        TraceRecordIterator {
            index: Some(&self.trace),
            pos: 0,
            len: self.trace.len(),
        }
    }

    #[inline]
    fn extensions(&self) -> Extensions<'_> {
        Extensions::new(&self.extensions)
    }

    #[inline]
    fn extensions_mut(&mut self) -> ExtensionsMut<'_> {
        ExtensionsMut::new(&mut self.extensions)
    }
}

impl Extract for TraceContext {
    #[inline]
    fn get<'a, E>(&'a self) -> Option<Downcasted<'a, E>>
    where
        E: Error + Extractable + 'static,
    {
        for e in self {
            let origin = match e.error_ref() {
                Some(v) => v,
                None => continue,
            };
            if origin.is::<E>() {
                return Some(Downcasted::<E>::new(origin));
            }
        }
        return None;
    }

    #[inline]
    fn has<'a, E>(&'a self) -> bool
    where
        E: Error + Extractable + 'static,
    {
        for e in self {
            let origin = match e.error_ref() {
                Some(v) => v,
                None => continue,
            };
            if origin.is::<E>() {
                return true;
            }
        }
        return false;
    }
}

/// Interface to implement tracing related functionality.
pub trait Traceable {
    /// Returns a trace context reference.
    // This function should only be used in public API.
    // Use `Traceable::trace_ref` for internal context access.
    fn trace(&self) -> &TraceContext;

    #[doc(hidden)]
    /// Returns a trace context reference.
    fn trace_ref(&self) -> Option<&TraceContext>;

    #[doc(hidden)]
    /// Takes the trace context.
    fn take_trace(&mut self) -> Option<TraceContext>;

    #[doc(hidden)]
    /// Returns the inherited error.
    fn inner(&self) -> Arc<dyn Error + Send + Sync>;

    #[doc(hidden)]
    /// Inserts a new trace record to the error chain.
    fn insert(&mut self, record: TraceRecord) -> bool;
}
