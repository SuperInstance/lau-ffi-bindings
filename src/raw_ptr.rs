use std::marker::PhantomData;
use std::ptr::NonNull;

/// A safe wrapper for raw pointers across FFI boundaries.
#[derive(Debug)]
pub struct SafePtr<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> SafePtr<T> {
    /// Create from a non-null raw pointer. Returns None for null.
    pub fn new(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|p| Self { ptr: p, _marker: PhantomData })
    }

    /// Create from a reference.
    pub fn from_ref(r: &T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(r as *const T as *mut T) },
            _marker: PhantomData,
        }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Read the value (copies it).
    pub fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { self.ptr.as_ptr().read() }
    }

    /// Write a value.
    pub fn write(&mut self, val: T) {
        unsafe { self.ptr.as_ptr().write(val); }
    }

    /// Get a reference to the pointed-to value.
    pub unsafe fn as_ref(&self) -> &T {
        &*self.ptr.as_ptr()
    }

    /// Get a mutable reference.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.ptr.as_ptr()
    }
}

impl<T> Clone for SafePtr<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, _marker: PhantomData }
    }
}

impl<T> Copy for SafePtr<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_ptr_null() {
        assert!(SafePtr::<u8>::new(std::ptr::null_mut()).is_none());
    }

    #[test]
    fn test_safe_ptr_read_write() {
        let mut val: u32 = 42;
        let mut ptr = SafePtr::new(&mut val).unwrap();
        assert_eq!(ptr.read(), 42);
        ptr.write(99);
        assert_eq!(val, 99);
    }

    #[test]
    fn test_safe_ptr_from_ref() {
        let val = 123u32;
        let ptr = SafePtr::from_ref(&val);
        assert_eq!(unsafe { *ptr.as_ref() }, 123);
    }
}
