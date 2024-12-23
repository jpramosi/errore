// Taken from https://github.com/tokio-rs/tracing/blob/master/tracing-subscriber/src/registry/extensions.rs
extern crate alloc;

use alloc::boxed::Box;
use core::marker::{Send, Sync};
use core::{
    any::{Any, TypeId},
    fmt,
};

use hashbrown::HashMap;

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// An immutable, read-only reference to a Context's extensions.
#[derive(Debug)]
pub struct Extensions<'a> {
    inner: &'a ExtensionsInner,
}

impl<'a> Extensions<'a> {
    #[inline]
    pub(crate) fn new(inner: &'a ExtensionsInner) -> Self {
        Self { inner }
    }

    /// Immutably borrows a type previously inserted into this `Extensions`.
    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }
}

/// An mutable reference to a Context's extensions.
#[derive(Debug)]
pub struct ExtensionsMut<'a> {
    inner: &'a mut ExtensionsInner,
}

impl<'a> ExtensionsMut<'a> {
    #[inline]
    pub(crate) fn new(inner: &'a mut ExtensionsInner) -> Self {
        Self { inner }
    }

    /// Insert a type into this `Extensions`.
    ///
    /// Note that extensions are _not_
    /// _global_-specific—they are _[context](crate::trace::TraceContext)_-specific. This means that
    /// another context cannot access and mutate extensions that
    /// a different context recorded.
    ///
    /// The best place to insert data into an `Extensions` instance is the handler
    /// [`Subscriber::on_start`](crate::subscriber::Subscriber::on_start).
    ///
    /// <div class="warning">
    /// Extensions should generally be newtypes, rather than common
    /// built-in types like String, to avoid accidental
    /// cross-crate clobbering.
    /// </div>
    ///
    /// ## Panics
    ///
    /// If `T` is already present in `Extensions`, then this method will panic.
    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        assert!(self.replace(val).is_none())
    }

    /// Replaces an existing `T` into this extensions.
    ///
    /// If `T` is not present, `Option::None` will be returned.
    #[inline]
    pub fn replace<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.inner.insert(val)
    }

    /// Get a mutable reference to a type previously inserted on this `ExtensionsMut`.
    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.get_mut::<T>()
    }

    /// Remove a type from this `Extensions`.
    ///
    /// If a extension of this type existed, it will be returned.
    #[inline]
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.inner.remove::<T>()
    }
}

/// A type map of context extensions.
///
/// [ExtensionsInner] is used by [`TraceContext`](crate::trace::TraceContext) to store
/// context-specific data. A given [`Subscriber`](crate::subscriber::Subscriber) can read and write
/// data that it is interested in recording and emitting.
#[derive(Default)]
pub(crate) struct ExtensionsInner {
    map: AnyMap,
}

impl ExtensionsInner {
    /// Reserves capacity for at least additional more elements.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    ///
    /// [`usize`]: https://doc.rust-lang.org/std/primitive.usize.html
    #[inline]
    pub(crate) fn reserve(&mut self, capacity: usize) {
        self.map.reserve(capacity);
    }

    /// Create an empty `Extensions`.
    #[inline]
    pub(crate) fn new() -> ExtensionsInner {
        ExtensionsInner { map: AnyMap::new() }
    }

    /// Insert a type into this `Extensions`.
    ///
    /// If a extension of this type already existed, it will
    /// be returned.
    pub(crate) fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                #[allow(warnings)]
                {
                    (boxed as Box<dyn Any + 'static>)
                        .downcast()
                        .ok()
                        .map(|boxed| *boxed)
                }
            })
    }

    /// Get a reference to a type previously inserted on this `Extensions`.
    pub(crate) fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    /// Get a mutable reference to a type previously inserted on this `Extensions`.
    pub(crate) fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    /// Remove a type from this `Extensions`.
    ///
    /// If a extension of this type existed, it will be returned.
    pub(crate) fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map.remove(&TypeId::of::<T>()).and_then(|boxed| {
            #[allow(warnings)]
            {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            }
        })
    }

    /// Clear the `ExtensionsInner` in-place, dropping any elements in the map but
    /// retaining allocated capacity.
    ///
    /// This permits the hash map allocation to be pooled by the registry so
    /// that future spans will not need to allocate new hashmaps.
    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }
}

impl fmt::Debug for ExtensionsInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extensions")
            .field("len", &self.map.len())
            .field("capacity", &self.map.capacity())
            .finish()
    }
}

/// Marks an object as extendable with user supplied extensions.
pub trait Extension {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MyType(i32);

    #[test]
    fn test_extensions() {
        let mut extensions = ExtensionsInner::new();

        extensions.insert(5i32);
        extensions.insert(MyType(10));

        assert_eq!(extensions.get(), Some(&5i32));
        assert_eq!(extensions.get_mut(), Some(&mut 5i32));

        assert_eq!(extensions.remove::<i32>(), Some(5i32));
        assert!(extensions.get::<i32>().is_none());

        assert_eq!(extensions.get::<bool>(), None);
        assert_eq!(extensions.get(), Some(&MyType(10)));
    }

    #[test]
    fn clear_retains_capacity() {
        let mut extensions = ExtensionsInner::new();
        extensions.insert(5i32);
        extensions.insert(MyType(10));
        extensions.insert(true);

        assert_eq!(extensions.map.len(), 3);
        let prev_capacity = extensions.map.capacity();
        extensions.clear();

        assert_eq!(
            extensions.map.len(),
            0,
            "after clear(), extensions map should have length 0"
        );
        assert_eq!(
            extensions.map.capacity(),
            prev_capacity,
            "after clear(), extensions map should retain prior capacity"
        );
    }

    #[test]
    fn clear_drops_elements() {
        use alloc::sync::Arc;
        struct DropMePlease(Arc<()>);
        struct DropMeTooPlease(Arc<()>);

        let mut extensions = ExtensionsInner::new();
        let val1 = DropMePlease(Arc::new(()));
        let val2 = DropMeTooPlease(Arc::new(()));

        let val1_dropped = Arc::downgrade(&val1.0);
        let val2_dropped = Arc::downgrade(&val2.0);
        extensions.insert(val1);
        extensions.insert(val2);

        assert!(val1_dropped.upgrade().is_some());
        assert!(val2_dropped.upgrade().is_some());

        extensions.clear();
        assert!(
            val1_dropped.upgrade().is_none(),
            "after clear(), val1 should be dropped"
        );
        assert!(
            val2_dropped.upgrade().is_none(),
            "after clear(), val2 should be dropped"
        );
    }
}
