//! # lau-ffi-bindings
//!
//! Cross-language FFI infrastructure — safe calling between Rust, C, WebAssembly,
//! and other languages. Provides C FFI types, struct layout compatibility,
//! WebAssembly target basics, callback interfaces, opaque handles, error propagation,
//! zero-copy data transfer, and C header generation.

pub mod cstring;
pub mod cvec;
pub mod raw_ptr;
pub mod layout;
pub mod wasm32_types;
pub mod callback;
pub mod handle;
pub mod error;
pub mod zero_copy;
pub mod header_gen;

pub use cstring::SafeCString;
pub use cvec::CVec;
pub use raw_ptr::SafePtr;
pub use layout::{StructLayout, FieldLayout, compute_padding, align_to};
pub use wasm32_types::WasmPtr;
pub use callback::{Callback, CallbackTrampoline};
pub use handle::{OpaqueHandle, HandleTable};
pub use error::{FfiResult, FfiError, ErrorCode};
pub use zero_copy::ZeroCopySlice;
pub use header_gen::HeaderGenerator;
