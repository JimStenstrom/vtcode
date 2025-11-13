use std::sync::{Arc, Mutex};

/// Generic in-memory buffer that stores items for later retrieval.
///
/// This helper is used by both [`crate::MemoryTelemetry`] and [`crate::MemoryErrorReporter`]
/// to eliminate code duplication. It provides thread-safe storage with a
/// drain-on-take semantic.
#[derive(Debug, Clone)]
pub(crate) struct MemoryBuffer<T> {
    items: Arc<Mutex<Vec<T>>>,
}

impl<T> Default for MemoryBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MemoryBuffer<T> {
    /// Creates a new empty buffer.
    pub(crate) fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds an item to the buffer.
    pub(crate) fn push(&self, item: T) {
        let mut items = self.items.lock().expect("memory buffer lock poisoned");
        items.push(item);
    }

    /// Returns all items, draining the buffer.
    pub(crate) fn take(&self) -> Vec<T> {
        let mut items = self.items.lock().expect("memory buffer lock poisoned");
        std::mem::take(&mut *items)
    }
}
