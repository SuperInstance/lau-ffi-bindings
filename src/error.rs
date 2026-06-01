use std::ffi::CString;
use std::os::raw::c_char;

/// Standard error codes for FFI boundaries.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Success = 0,
    NullPointer = -1,
    InvalidHandle = -2,
    OutOfBounds = -3,
    InvalidArgument = -4,
    OutOfMemory = -5,
    EncodingError = -6,
    IOError = -7,
    Unknown = -99,
}

impl ErrorCode {
    /// Get the numeric code.
    pub fn code(&self) -> i32 {
        *self as i32
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::Success => "Success",
            ErrorCode::NullPointer => "Null pointer",
            ErrorCode::InvalidHandle => "Invalid handle",
            ErrorCode::OutOfBounds => "Out of bounds",
            ErrorCode::InvalidArgument => "Invalid argument",
            ErrorCode::OutOfMemory => "Out of memory",
            ErrorCode::EncodingError => "Encoding error",
            ErrorCode::IOError => "I/O error",
            ErrorCode::Unknown => "Unknown error",
        }
    }

    /// Convert from an i32 code.
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => ErrorCode::Success,
            -1 => ErrorCode::NullPointer,
            -2 => ErrorCode::InvalidHandle,
            -3 => ErrorCode::OutOfBounds,
            -4 => ErrorCode::InvalidArgument,
            -5 => ErrorCode::OutOfMemory,
            -6 => ErrorCode::EncodingError,
            -7 => ErrorCode::IOError,
            _ => ErrorCode::Unknown,
        }
    }
}

/// Error type that can cross FFI boundaries.
#[derive(Debug, Clone)]
pub struct FfiError {
    pub code: ErrorCode,
    pub message: String,
}

impl FfiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    /// Get the error code.
    pub fn code(&self) -> i32 {
        self.code.code()
    }

    /// Get the error message as a C string pointer (valid while self lives).
    pub fn message_ptr(&self) -> *const c_char {
        // Note: In real usage you'd cache the CString. This is a simplified version.
        match CString::new(self.message.as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null(),
        }
    }

    /// Create a null pointer error.
    pub fn null_pointer(msg: &str) -> Self {
        Self::new(ErrorCode::NullPointer, msg)
    }

    /// Create an invalid handle error.
    pub fn invalid_handle(msg: &str) -> Self {
        Self::new(ErrorCode::InvalidHandle, msg)
    }

    /// Create an out of bounds error.
    pub fn out_of_bounds(msg: &str) -> Self {
        Self::new(ErrorCode::OutOfBounds, msg)
    }
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FfiError({}, {})", self.code.code(), self.message)
    }
}

impl std::error::Error for FfiError {}

/// Result type for FFI operations.
pub type FfiResult<T> = Result<T, FfiError>;

/// Trait for converting results to FFI-compatible error codes.
pub trait IntoFfiCode {
    fn into_code(self) -> i32;
}

impl<T> IntoFfiCode for FfiResult<T> {
    fn into_code(self) -> i32 {
        match self {
            Ok(_) => ErrorCode::Success as i32,
            Err(e) => e.code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_roundtrip() {
        assert_eq!(ErrorCode::from_code(0), ErrorCode::Success);
        assert_eq!(ErrorCode::from_code(-1), ErrorCode::NullPointer);
        assert_eq!(ErrorCode::from_code(-2), ErrorCode::InvalidHandle);
        assert_eq!(ErrorCode::from_code(-99), ErrorCode::Unknown);
        assert_eq!(ErrorCode::from_code(-100), ErrorCode::Unknown);
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::Success.code(), 0);
        assert_eq!(ErrorCode::NullPointer.code(), -1);
        assert_eq!(ErrorCode::OutOfBounds.code(), -3);
    }

    #[test]
    fn test_ffi_error_creation() {
        let err = FfiError::new(ErrorCode::InvalidArgument, "bad input");
        assert_eq!(err.code(), -4);
        assert_eq!(err.message, "bad input");
    }

    #[test]
    fn test_ffi_error_display() {
        let err = FfiError::new(ErrorCode::OutOfMemory, "oom");
        assert_eq!(format!("{}", err), "FfiError(-5, oom)");
    }

    #[test]
    fn test_ffi_result_ok() {
        let result: FfiResult<i32> = Ok(42);
        assert_eq!(result.into_code(), 0);
    }

    #[test]
    fn test_ffi_result_err() {
        let result: FfiResult<i32> = Err(FfiError::null_pointer("ptr was null"));
        assert_eq!(result.into_code(), -1);
    }

    #[test]
    fn test_convenience_constructors() {
        let e1 = FfiError::null_pointer("npe");
        assert_eq!(e1.code, ErrorCode::NullPointer);
        let e2 = FfiError::invalid_handle("bad handle");
        assert_eq!(e2.code, ErrorCode::InvalidHandle);
        let e3 = FfiError::out_of_bounds("idx 5");
        assert_eq!(e3.code, ErrorCode::OutOfBounds);
    }

    #[test]
    fn test_error_code_descriptions() {
        assert_eq!(ErrorCode::Success.description(), "Success");
        assert_eq!(ErrorCode::NullPointer.description(), "Null pointer");
    }
}
