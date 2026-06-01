use std::marker::PhantomData;
use std::mem::ManuallyDrop;

/// A zero-copy slice view for passing data across FFI without copying.
/// The consumer must NOT modify or free the data.
#[repr(C)]
pub struct ZeroCopySlice<'a, T> {
    ptr: *const T,
    len: usize,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> ZeroCopySlice<'a, T> {
    /// Create from a Rust slice — no copying.
    pub fn from_slice(slice: &'a [T]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
            _marker: PhantomData,
        }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reconstruct as a Rust slice (unsafe — caller ensures lifetime/validity).
    pub unsafe fn as_slice(&self) -> &'a [T] {
        std::slice::from_raw_parts(self.ptr, self.len)
    }
}

impl<'a, T> Clone for ZeroCopySlice<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, len: self.len, _marker: PhantomData }
    }
}

impl<'a, T> Copy for ZeroCopySlice<'a, T> {}

/// A mutable zero-copy slice for read-write access across FFI.
#[repr(C)]
pub struct ZeroCopySliceMut<'a, T> {
    ptr: *mut T,
    len: usize,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> ZeroCopySliceMut<'a, T> {
    /// Create from a mutable Rust slice.
    pub fn from_slice(slice: &'a mut [T]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            _marker: PhantomData,
        }
    }

    /// Get the raw pointer.
    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reconstruct as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &'a mut [T] {
        std::slice::from_raw_parts_mut(self.ptr, self.len)
    }
}

/// Zero-copy transfer of a Vec: sender gives up ownership, receiver gets a pointer
/// and must call `reclaim` when done.
pub struct ZeroCopyTransfer<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> ZeroCopyTransfer<T> {
    /// Create from a Vec. The Vec is forgotten (not dropped).
    pub fn from_vec(vec: Vec<T>) -> Self {
        let mut md = ManuallyDrop::new(vec);
        Self {
            ptr: md.as_mut_ptr(),
            len: md.len(),
            capacity: md.capacity(),
        }
    }

    /// Get the raw pointer (read-only view).
    pub fn ptr(&self) -> *const T {
        self.ptr
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reclaim the data back into a Vec. Must be called exactly once to avoid leaks.
    pub unsafe fn reclaim(self) -> Vec<T> {
        let this = ManuallyDrop::new(self);
        Vec::from_raw_parts(this.ptr, this.len, this.capacity)
    }

    /// Reclaim as a slice (does not take ownership back — will leak if not reclaimed separately).
    pub unsafe fn as_slice(&self) -> &[T] {
        std::slice::from_raw_parts(self.ptr, self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_slice_from_slice() {
        let data = vec![1u32, 2, 3, 4, 5];
        let zcs = ZeroCopySlice::from_slice(&data);
        assert_eq!(zcs.len(), 5);
        assert!(!zcs.is_empty());
        let slice = unsafe { zcs.as_slice() };
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_zero_copy_slice_empty() {
        let data: Vec<u8> = vec![];
        let zcs = ZeroCopySlice::from_slice(&data);
        assert!(zcs.is_empty());
        assert_eq!(zcs.len(), 0);
    }

    #[test]
    fn test_zero_copy_mut() {
        let mut data = vec![10u32, 20, 30];
        let mut zcm = ZeroCopySliceMut::from_slice(&mut data);
        let slice = unsafe { zcm.as_mut_slice() };
        slice[0] = 99;
        assert_eq!(data[0], 99);
    }

    #[test]
    fn test_zero_copy_transfer_roundtrip() {
        let v = vec![100u64, 200, 300];
        let transfer = ZeroCopyTransfer::from_vec(v);
        assert_eq!(transfer.len(), 3);
        let slice = unsafe { transfer.as_slice() };
        assert_eq!(slice[0], 100);
        // Need to reclaim from a fresh one since as_slice borrows
        let v2 = vec![100u64, 200, 300];
        let t2 = ZeroCopyTransfer::from_vec(v2);
        let reclaimed = unsafe { t2.reclaim() };
        assert_eq!(reclaimed, vec![100, 200, 300]);
    }

    #[test]
    fn test_zero_copy_no_allocation() {
        let data = [1i32, 2, 3];
        let zcs = ZeroCopySlice::from_slice(&data);
        // Pointer should point to the original data
        assert_eq!(zcs.as_ptr(), data.as_ptr());
    }

    #[test]
    fn test_zero_copy_slice_copy_trait() {
        let data = vec![1u8, 2, 3];
        let zcs1 = ZeroCopySlice::from_slice(&data);
        let zcs2 = zcs1; // Copy
        assert_eq!(zcs1.as_ptr(), zcs2.as_ptr());
        assert_eq!(zcs1.len(), zcs2.len());
    }
}
