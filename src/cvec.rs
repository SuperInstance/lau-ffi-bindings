use std::marker::PhantomData;
use std::mem::ManuallyDrop;

/// A C-compatible vector that owns its data and can be passed across FFI.
/// Layout: `data` pointer, `len`, `capacity` — compatible with `Vec` on most platforms.
#[repr(C)]
#[derive(Debug)]
pub struct CVec<T> {
    data: *mut T,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T> CVec<T> {
    /// Create an empty CVec.
    pub fn new() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            _marker: PhantomData,
        }
    }

    /// Create from a Rust Vec.
    pub fn from_vec(mut v: Vec<T>) -> Self {
        let data = v.as_mut_ptr();
        let len = v.len();
        let capacity = v.capacity();
        std::mem::forget(v);
        Self {
            data,
            len,
            capacity,
            _marker: PhantomData,
        }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *const T {
        self.data
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Access element by index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index is bounds-checked, data is valid
            unsafe { Some(&*self.data.add(index)) }
        } else {
            None
        }
    }

    /// Get as a slice (unsafe — caller must ensure the memory is valid).
    pub unsafe fn as_slice(&self) -> &[T] {
        std::slice::from_raw_parts(self.data, self.len)
    }

    /// Convert back into a Vec, taking ownership.
    pub fn into_vec(self) -> Vec<T> {
        let this = ManuallyDrop::new(self);
        if this.capacity == 0 {
            Vec::new()
        } else {
            // SAFETY: data, len, capacity came from a valid Vec
            unsafe { Vec::from_raw_parts(this.data, this.len, this.capacity) }
        }
    }

    /// Get the raw parts (data pointer, length, capacity) for FFI.
    pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
        let this = ManuallyDrop::new(self);
        (this.data, this.len, this.capacity)
    }

    /// Reconstruct from raw parts.
    pub unsafe fn from_raw_parts(data: *mut T, len: usize, capacity: usize) -> Self {
        Self {
            data,
            len,
            capacity,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for CVec<T> {
    fn drop(&mut self) {
        if self.capacity > 0 && !self.data.is_null() {
            // SAFETY: data/len/capacity came from a valid Vec
            unsafe { drop(Vec::from_raw_parts(self.data, self.len, self.capacity)); }
        }
    }
}

// SAFETY: CVec owns its data and can be sent across threads if T can.
unsafe impl<T: Send> Send for CVec<T> {}
unsafe impl<T: Sync> Sync for CVec<T> {}

impl<T> Default for CVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for CVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self::from_vec(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvec_from_vec_roundtrip() {
        let v = vec![1u32, 2, 3, 4, 5];
        let cv = CVec::from_vec(v.clone());
        assert_eq!(cv.len(), 5);
        let back = cv.into_vec();
        assert_eq!(v, back);
    }

    #[test]
    fn test_cvec_get() {
        let cv = CVec::from_vec(vec![10u32, 20, 30]);
        assert_eq!(cv.get(0), Some(&10));
        assert_eq!(cv.get(2), Some(&30));
        assert_eq!(cv.get(3), None);
    }

    #[test]
    fn test_cvec_empty() {
        let cv: CVec<u8> = CVec::new();
        assert!(cv.is_empty());
        assert_eq!(cv.len(), 0);
        let v = cv.into_vec();
        assert!(v.is_empty());
    }

    #[test]
    fn test_cvec_into_raw_parts() {
        let cv = CVec::from_vec(vec![42u64]);
        let (ptr, len, cap) = cv.into_raw_parts();
        assert_eq!(len, 1);
        assert!(cap >= 1);
        unsafe {
            assert_eq!(*ptr, 42);
            drop(Vec::from_raw_parts(ptr, len, cap));
        }
    }

    #[test]
    fn test_cvec_as_slice() {
        let cv = CVec::from_vec(vec![1u8, 2, 3]);
        let slice = unsafe { cv.as_slice() };
        assert_eq!(slice, &[1, 2, 3]);
    }
}
