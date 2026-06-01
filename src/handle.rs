use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

/// A type-safe opaque handle wrapping a raw pointer.
/// Prevents misuse of raw pointers across FFI boundaries.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OpaqueHandle<T> {
    id: u64,
    _marker: PhantomData<T>,
}

impl<T> Clone for OpaqueHandle<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, _marker: PhantomData }
    }
}

impl<T> Copy for OpaqueHandle<T> {}

impl<T> OpaqueHandle<T> {
    /// Create a new unique handle.
    pub fn new() -> Self {
        Self {
            id: NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed),
            _marker: PhantomData,
        }
    }

    /// Create a handle from a specific ID (for reconstruction from FFI).
    pub fn from_id(id: u64) -> Self {
        Self { id, _marker: PhantomData }
    }

    /// Get the handle's ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Check if this is a valid (non-zero) handle.
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}

impl<T> Default for OpaqueHandle<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle table that maps opaque handles to their underlying data.
/// Enables safe management of FFI resources.
pub struct HandleTable<T> {
    entries: HashMap<u64, T>,
}

impl<T> HandleTable<T> {
    /// Create a new empty handle table.
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Insert a value and return an opaque handle for it.
    pub fn insert(&mut self, value: T) -> OpaqueHandle<T> {
        let handle = OpaqueHandle::new();
        self.entries.insert(handle.id(), value);
        handle
    }

    /// Get a reference to the value behind a handle.
    pub fn get(&self, handle: OpaqueHandle<T>) -> Option<&T> {
        self.entries.get(&handle.id())
    }

    /// Get a mutable reference to the value behind a handle.
    pub fn get_mut(&mut self, handle: OpaqueHandle<T>) -> Option<&mut T> {
        self.entries.get_mut(&handle.id())
    }

    /// Remove and return the value behind a handle (drop across FFI).
    pub fn remove(&mut self, handle: OpaqueHandle<T>) -> Option<T> {
        self.entries.remove(&handle.id())
    }

    /// Check if a handle exists.
    pub fn contains(&self, handle: OpaqueHandle<T>) -> bool {
        self.entries.contains_key(&handle.id())
    }

    /// Number of active handles.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> Default for HandleTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_unique_ids() {
        let h1: OpaqueHandle<u8> = OpaqueHandle::new();
        let h2: OpaqueHandle<u8> = OpaqueHandle::new();
        assert_ne!(h1.id(), h2.id());
    }

    #[test]
    fn test_handle_is_valid() {
        let h: OpaqueHandle<u8> = OpaqueHandle::new();
        assert!(h.is_valid());
        let zero = OpaqueHandle::<u8>::from_id(0);
        assert!(!zero.is_valid());
    }

    #[test]
    fn test_handle_table_insert_get() {
        let mut table: HandleTable<String> = HandleTable::new();
        let h = table.insert("hello".to_string());
        assert_eq!(table.get(h), Some(&"hello".to_string()));
    }

    #[test]
    fn test_handle_table_remove() {
        let mut table: HandleTable<Vec<u8>> = HandleTable::new();
        let h = table.insert(vec![1, 2, 3]);
        assert!(table.contains(h));
        let val = table.remove(h);
        assert_eq!(val, Some(vec![1, 2, 3]));
        assert!(!table.contains(h));
        assert!(table.is_empty());
    }

    #[test]
    fn test_handle_table_get_mut() {
        let mut table: HandleTable<i32> = HandleTable::new();
        let h = table.insert(10);
        if let Some(v) = table.get_mut(h) {
            *v = 20;
        }
        assert_eq!(table.get(h), Some(&20));
    }

    #[test]
    fn test_handle_table_multiple() {
        let mut table: HandleTable<u32> = HandleTable::new();
        let h1 = table.insert(1);
        let h2 = table.insert(2);
        let h3 = table.insert(3);
        assert_eq!(table.len(), 3);
        assert_eq!(table.get(h1), Some(&1));
        assert_eq!(table.get(h2), Some(&2));
        assert_eq!(table.get(h3), Some(&3));
    }

    #[test]
    fn test_handle_type_safety() {
        // Different types produce different handle types
        let h_int: OpaqueHandle<i32> = OpaqueHandle::new();
        let h_str: OpaqueHandle<String> = OpaqueHandle::new();
        // These are different types and can't be mixed up
        assert!(h_int.is_valid());
        assert!(h_str.is_valid());
    }

    #[test]
    fn test_handle_from_id() {
        let h: OpaqueHandle<f64> = OpaqueHandle::from_id(42);
        assert_eq!(h.id(), 42);
    }
}
