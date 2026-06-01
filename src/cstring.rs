use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use serde::{Serialize, Deserialize};

/// A safe wrapper around `CString` for FFI boundary crossing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeCString {
    inner: CString,
}

impl SafeCString {
    /// Create from a Rust string.
    pub fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        Ok(Self { inner: CString::new(s)? })
    }

    /// Create from a raw C string pointer. Returns `None` if pointer is null.
    pub fn from_raw(ptr: *const c_char) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: caller must ensure ptr is a valid null-terminated C string
        unsafe {
            Some(Self { inner: CStr::from_ptr(ptr).to_owned() })
        }
    }

    /// Get the raw pointer for passing across FFI.
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    /// Get a mutable raw pointer.
    pub fn as_ptr_mut(&mut self) -> *mut c_char {
        self.inner.as_ptr() as *mut c_char
    }

    /// Convert to a Rust String. Returns None if the C string is not valid UTF-8.
    pub fn to_string_lossy(&self) -> String {
        self.inner.to_string_lossy().into_owned()
    }

    /// Convert to bytes (excluding the null terminator).
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    /// Get the underlying CString.
    pub fn into_inner(self) -> CString {
        self.inner
    }

    /// Get the byte length (excluding null terminator).
    pub fn len(&self) -> usize {
        self.inner.as_bytes().len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.inner.as_bytes().is_empty()
    }
}

impl std::fmt::Display for SafeCString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl From<CString> for SafeCString {
    fn from(inner: CString) -> Self {
        Self { inner }
    }
}

impl TryFrom<&str> for SafeCString {
    type Error = std::ffi::NulError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstring_roundtrip() {
        let s = SafeCString::new("hello world").unwrap();
        let ptr = s.as_ptr();
        let s2 = SafeCString::from_raw(ptr).unwrap();
        assert_eq!(s, s2);
        assert_eq!(s2.to_string_lossy(), "hello world");
    }

    #[test]
    fn test_cstring_from_null() {
        assert!(SafeCString::from_raw(std::ptr::null()).is_none());
    }

    #[test]
    fn test_cstring_empty() {
        let s = SafeCString::new("").unwrap();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_cstring_bytes() {
        let s = SafeCString::new("abc").unwrap();
        assert_eq!(s.as_bytes(), b"abc");
    }

    #[test]
    fn test_cstring_nul_error() {
        let result = SafeCString::new("hello\0world");
        assert!(result.is_err());
    }

    #[test]
    fn test_cstring_display() {
        let s = SafeCString::new("test").unwrap();
        assert_eq!(format!("{}", s), "test");
    }

    #[test]
    fn test_cstring_clone_equality() {
        let s = SafeCString::new("clone me").unwrap();
        let c = s.clone();
        assert_eq!(s, c);
    }

    #[test]
    fn test_cstring_serde_roundtrip() {
        let s = SafeCString::new("serde test").unwrap();
        let json = serde_json::to_string(&s).unwrap();
        let s2: SafeCString = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }
}
