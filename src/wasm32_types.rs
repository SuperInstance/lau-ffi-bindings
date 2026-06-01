use serde::{Serialize, Deserialize};

/// Pointer size for wasm32 targets (32-bit, 4 bytes).
pub const WASM32_POINTER_SIZE: usize = 4;

/// Memory page size for WebAssembly (64KB).
pub const WASM32_PAGE_SIZE: usize = 65536;

/// Maximum memory addressable in wasm32 (4GB linear memory).
pub const WASM32_MAX_MEMORY: usize = 4 * 1024 * 1024 * 1024;

/// A 32-bit pointer type for wasm32 targets.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WasmPtr(u32);

impl WasmPtr {
    /// Null pointer.
    pub const NULL: WasmPtr = WasmPtr(0);

    /// Create a new WasmPtr from a u32 address.
    pub fn new(addr: u32) -> Self {
        WasmPtr(addr)
    }

    /// Get the raw address.
    pub fn addr(&self) -> u32 {
        self.0
    }

    /// Check if this is a null pointer.
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Offset this pointer by `n` bytes.
    pub fn offset(&self, n: u32) -> Self {
        WasmPtr(self.0 + n)
    }

    /// Align up to the given alignment.
    pub fn align_up(&self, align: u32) -> Self {
        WasmPtr((self.0 + align - 1) & !(align - 1))
    }
}

/// Types mapping for wasm32 targets.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    /// wasm32 pointer (i32)
    Ptr,
    /// Reference type (externref)
    ExternRef,
    /// Reference type (funcref)  
    FuncRef,
}

impl WasmType {
    /// Size in bytes of each wasm type.
    pub fn size(&self) -> usize {
        match self {
            WasmType::I32 | WasmType::F32 | WasmType::Ptr => 4,
            WasmType::I64 | WasmType::F64 => 8,
            WasmType::ExternRef | WasmType::FuncRef => WASM32_POINTER_SIZE,
        }
    }

    /// Alignment in bytes.
    pub fn alignment(&self) -> usize {
        self.size()
    }
}

/// wasm32 memory import descriptor.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMemoryImport {
    pub module_name: String,
    pub field_name: String,
    pub initial_pages: u32,
    pub maximum_pages: Option<u32>,
    pub shared: bool,
}

/// wasm32 function export descriptor.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunctionExport {
    pub name: String,
    pub params: Vec<WasmType>,
    pub results: Vec<WasmType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_ptr_null() {
        assert!(WasmPtr::NULL.is_null());
        assert!(WasmPtr::new(0).is_null());
        assert!(!WasmPtr::new(1).is_null());
    }

    #[test]
    fn test_wasm_ptr_offset() {
        let p = WasmPtr::new(100);
        assert_eq!(p.offset(10).addr(), 110);
    }

    #[test]
    fn test_wasm_ptr_align_up() {
        let p = WasmPtr::new(5);
        assert_eq!(p.align_up(4).addr(), 8);
        assert_eq!(p.align_up(1).addr(), 5);
    }

    #[test]
    fn test_wasm_type_sizes() {
        assert_eq!(WasmType::I32.size(), 4);
        assert_eq!(WasmType::I64.size(), 8);
        assert_eq!(WasmType::F32.size(), 4);
        assert_eq!(WasmType::F64.size(), 8);
        assert_eq!(WasmType::Ptr.size(), 4);
    }

    #[test]
    fn test_wasm32_constants() {
        assert_eq!(WASM32_POINTER_SIZE, 4);
        assert_eq!(WASM32_PAGE_SIZE, 65536);
        assert_eq!(WASM32_MAX_MEMORY, 4 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_wasm_function_export() {
        let exp = WasmFunctionExport {
            name: "add".to_string(),
            params: vec![WasmType::I32, WasmType::I32],
            results: vec![WasmType::I32],
        };
        assert_eq!(exp.params.len(), 2);
        assert_eq!(exp.results.len(), 1);
    }

    #[test]
    fn test_wasm_memory_import() {
        let imp = WasmMemoryImport {
            module_name: "env".to_string(),
            field_name: "memory".to_string(),
            initial_pages: 1,
            maximum_pages: Some(10),
            shared: false,
        };
        assert_eq!(imp.initial_pages, 1);
    }
}
