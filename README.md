# lau-ffi-bindings

> Cross-language FFI infrastructure — safe calling between Rust, C, WebAssembly, and other languages.

Part of the **PLATO/LAU ecosystem** — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

---

## What This Does

`lau-ffi-bindings` provides the foreign-function interface layer for the `lau-*` crate family. It makes Rust libraries callable from C, WebAssembly, and any language that speaks C ABIs. Specifically:

- **Safe C string handling** — `SafeCString` wraps `CString` with null-safety, round-tripping, and serialization
- **C-compatible vectors** — `CVec<T>` is a `#[repr(C)]` owned buffer (data pointer + len + capacity) that converts to/from `Vec<T>`
- **Raw pointer safety** — `SafePtr<T>` wraps `NonNull<T>` with checked construction, read/write, and reference access
- **Struct layout computation** — `StructLayoutBuilder` computes field offsets, alignment padding, and total size matching C ABI rules
- **WebAssembly target types** — `WasmPtr<T>` with 32-bit addressing, `WasmSlice`, `WasmSliceMut`, and typed Wasm buffer views
- **Callback interfaces** — `Callback<Args, Ret>` with closure wrapping, `CallbackTrampoline` for FFI function pointers, and C-compatible trampoline signatures
- **Opaque handles** — `OpaqueHandle<T>` (u64 ID) with `HandleTable<T>` for safe cross-language handle-based access without exposing pointers
- **Error propagation** — `FfiResult<T>`, `FfiError` with error codes, error message strings, and thread-local last-error for C-style error retrieval
- **Zero-copy data transfer** — `ZeroCopySlice<T>` borrows a `Vec<T>` and exposes a raw pointer + length for zero-allocation FFI reads; `ZeroCopyMapper` provides a simple memory-mapped file abstraction
- **C header generation** — `HeaderGenerator` emits `.h` files with includes, typedefs, forward declarations, structs (with `_Static_assert` size checks), and function declarations wrapped in `extern "C"`

Everything compiles without any external C libraries or Wasm runtimes. The types are pure Rust that model FFI patterns.

---

## The Key Idea

Crossing the FFI boundary is where bugs live — null pointers, dangling references, mismatched layouts, leaked memory. This crate encodes the rules into Rust's type system:

1. **No null pointers escape** — `SafePtr::new()` returns `Option`, `SafeCString::from_raw()` returns `Option`
2. **Ownership is explicit** — `CVec` owns its data (Drop reclaims), `ZeroCopySlice` borrows (no allocation), `OpaqueHandle` is a non-pointer ID
3. **Layouts are computed, not guessed** — `StructLayoutBuilder` inserts padding and computes offsets exactly like a C compiler would
4. **Headers are generated from code** — `HeaderGenerator` produces C headers that match the Rust `#[repr(C)]` types, with static assertions to catch drift

---

## Install

```bash
cargo add lau-ffi-bindings
```

### Dependencies

| Crate | Version | Why |
|---|---|---|
| `serde` | 1 | Serialization of layouts, handles, errors, Wasm types |
| `nalgebra` | 0.33 | Matrix types available for layout computation |
| `serde_json` | 1 | *(dev-only)* test serialization round-trips |

No C compiler or Wasm toolchain required.

---

## Quick Start

### C strings

```rust
use lau_ffi_bindings::SafeCString;

let s = SafeCString::new("hello").unwrap();
let ptr = s.as_ptr();                    // *const c_char — pass to C
let s2 = SafeCString::from_raw(ptr).unwrap();  // bring it back
assert_eq!(s2.to_string_lossy(), "hello");
```

### C-compatible vectors

```rust
use lau_ffi_bindings::CVec;

let v = vec![1u32, 2, 3, 4, 5];
let mut cv = CVec::from_vec(v);
assert_eq!(cv.len(), 5);
assert_eq!(cv.get(2), Some(&3));

let back: Vec<u32> = cv.into_vec();  // reclaim ownership
```

### Struct layout computation

```rust
use lau_ffi_bindings::{StructLayoutBuilder, compute_padding, align_to};

let mut builder = StructLayoutBuilder::new("Particle");
builder.add_field("x", "f64", 8, 8);
builder.add_field("y", "f64", 8, 8);
builder.add_field("mass", "f32", 4, 4);
builder.add_field("active", "bool", 1, 1);
let layout = builder.build();

assert_eq!(layout.fields[0].offset, 0);   // x at 0
assert_eq!(layout.fields[1].offset, 8);   // y at 8
assert_eq!(layout.total_size, 24);         // 8+8+4+1 + 3 padding to align to 8
```

### WebAssembly pointers

```rust
use lau_ffi_bindings::{WasmPtr, WasmSlice};

let ptr = WasmPtr::<f32>::new(1024);
assert_eq!(ptr.addr(), 1024);

let slice = WasmSlice::new(0, 256);  // offset 0, length 256
assert_eq!(slice.len(), 256);
```

### Opaque handles

```rust
use lau_ffi_bindings::{HandleTable, OpaqueHandle};

let mut table = HandleTable::new();
let handle: OpaqueHandle<String> = table.insert("hello".to_string());

// From C: pass handle as u64, look up on the Rust side
let value = table.get(handle).unwrap();
assert_eq!(value, "hello");

table.remove(handle);
assert!(table.get(handle).is_none());
```

### Error propagation

```rust
use lau_ffi_bindings::{FfiResult, FfiError, ErrorCode};

fn do_thing(x: i32) -> FfiResult<i32> {
    if x < 0 {
        Err(FfiError::new(ErrorCode::InvalidArgument, "x must be non-negative"))
    } else {
        Ok(x * 2)
    }
}

assert!(do_thing(-1).is_err());
assert_eq!(do_thing(5).unwrap(), 10);
```

### Zero-copy transfer

```rust
use lau_ffi_bindings::ZeroCopySlice;

let data = vec![1.0f32, 2.0, 3.0, 4.0];
let zc = ZeroCopySlice::new(&data);

// Pass zc.as_ptr() and zc.len() to C — no copy, no allocation
assert_eq!(zc.len(), 4);
assert_eq!(unsafe { *zc.as_ptr() }, 1.0);
```

### C header generation

```rust
use lau_ffi_bindings::{HeaderGenerator, StructLayoutBuilder};

let mut gen = HeaderGenerator::new();
gen.add_include("stdint.h");
gen.add_typedef("int32_t", "AgentId");
gen.add_function("void", "agent_init", &[
    ("AgentId".to_string(), "id".to_string()),
]);
let header = gen.generate("AGENT_H");
// Produces a complete .h file with include guard, extern "C", etc.
```

---

## API Reference

### SafeCString

```rust
pub struct SafeCString { /* wraps CString */ }
impl SafeCString {
    pub fn new(s: &str) -> Result<Self, NulError>;
    pub fn from_raw(ptr: *const c_char) -> Option<Self>;
    pub fn as_ptr(&self) -> *const c_char;
    pub fn as_ptr_mut(&mut self) -> *mut c_char;
    pub fn to_string_lossy(&self) -> String;
    pub fn as_bytes(&self) -> &[u8];
    pub fn into_inner(self) -> CString;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
impl From<CString> for SafeCString { ... }
impl TryFrom<&str> for SafeCString { ... }
impl Display for SafeCString { ... }
```

### CVec\<T\>

```rust
#[repr(C)]
pub struct CVec<T> { /* data: *mut T, len: usize, capacity: usize */ }
impl<T> CVec<T> {
    pub fn new() -> Self;
    pub fn from_vec(v: Vec<T>) -> Self;
    pub fn as_ptr(&self) -> *const T;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, index: usize) -> Option<&T>;
    pub unsafe fn as_slice(&self) -> &[T];
    pub fn into_vec(self) -> Vec<T>;
    pub fn into_raw_parts(self) -> (*mut T, usize, usize);
    pub unsafe fn from_raw_parts(data: *mut T, len: usize, capacity: usize) -> Self;
}
impl<T: Send> Send for CVec<T> {}
impl<T: Sync> Sync for CVec<T> {}
impl<T> From<Vec<T>> for CVec<T> { ... }
```

### SafePtr\<T\>

```rust
pub struct SafePtr<T> { /* NonNull<T> */ }
impl<T> SafePtr<T> {
    pub fn new(ptr: *mut T) -> Option<Self>;
    pub fn from_ref(r: &T) -> Self;
    pub fn as_ptr(&self) -> *mut T;
    pub fn read(&self) -> T where T: Copy;
    pub fn write(&mut self, val: T);
    pub unsafe fn as_ref(&self) -> &T;
    pub unsafe fn as_mut(&mut self) -> &mut T;
}
impl<T> Clone for SafePtr<T> {}
impl<T> Copy for SafePtr<T> {}
```

### Layout

```rust
pub struct FieldLayout { pub name: String, pub type_name: String, pub size: usize, pub alignment: usize, pub offset: usize }
pub struct StructLayout { pub name: String, pub fields: Vec<FieldLayout>, pub total_size: usize, pub alignment: usize, pub padding: usize }

pub fn align_to(offset: usize, alignment: usize) -> usize;
pub fn compute_padding(offset: usize, alignment: usize) -> usize;

pub struct StructLayoutBuilder { /* ... */ }
impl StructLayoutBuilder {
    pub fn new(name: &str) -> Self;
    pub fn add_field(&mut self, name: &str, type_name: &str, size: usize, alignment: usize);
    pub fn build(self) -> StructLayout;
}
```

### Wasm Types

```rust
pub struct WasmPtr<T> { addr: u32, _marker: PhantomData<T> }
impl<T> WasmPtr<T> {
    pub fn new(addr: u32) -> Self;
    pub fn addr(&self) -> u32;
    pub fn is_null(&self) -> bool;
    pub fn null() -> Self;
    pub unsafe fn read_from(&self, memory: &[u8]) -> T where T: Copy;
    pub unsafe fn write_to(&self, memory: &mut [u8], val: &T) where T: Copy;
}

pub struct WasmSlice { offset: u32, len: u32 }
impl WasmSlice {
    pub fn new(offset: u32, len: u32) -> Self;
    pub fn offset/len(&self) -> u32;
    pub fn read_into(&self, memory: &[u8], out: &mut [u8]);
    pub fn write_from(&self, memory: &mut [u8], data: &[u8]);
}

pub struct WasmSliceMut { offset: u32, len: u32 }
pub struct WasmBuffer { base: u32, size: u32 }
pub enum WasmType { I32, I64, F32, F64, Void }
```

### Callback

```rust
pub struct Callback<Args, Ret> { /* wraps Box<dyn Fn(Args) -> Ret + Send> */ }
impl<Args, Ret> Callback<Args, Ret> {
    pub fn new<F>(f: F) -> Self where F: Fn(Args) -> Ret + Send + 'static;
    pub fn call(&self, args: Args) -> Ret;
    pub fn into_raw(self) -> *mut Self;
    pub unsafe fn from_raw(ptr: *mut Self) -> Option<Self>;
}

pub struct CallbackTrampoline;
impl CallbackTrampoline {
    pub unsafe fn trampoline_2_i32(ret: *mut Callback<(i32, i32), i32>, a: i32, b: i32) -> i32;
    pub unsafe fn trampoline_void(ret: *mut Callback<(), ()>);
    pub fn c_signature(args: usize, ret: bool) -> String;
}
```

### Handle

```rust
pub struct OpaqueHandle<T> { id: u64, _marker: PhantomData<T> }
impl<T> OpaqueHandle<T> {
    pub fn id(&self) -> u64;
}

pub struct HandleTable<T> { entries: HashMap<u64, T>, next_id: u64 }
impl<T> HandleTable<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, value: T) -> OpaqueHandle<T>;
    pub fn get(&self, handle: OpaqueHandle<T>) -> Option<&T>;
    pub fn get_mut(&mut self, handle: OpaqueHandle<T>) -> Option<&mut T>;
    pub fn remove(&mut self, handle: OpaqueHandle<T>) -> Option<T>;
    pub fn len/is_empty(&self) -> _;
    pub fn contains(&self, handle: OpaqueHandle<T>) -> bool;
}
```

### Error

```rust
pub enum ErrorCode { Success, InvalidArgument, NullPointer, OutOfBounds, AllocationFailed, IOError, Unknown }
pub struct FfiError { pub code: ErrorCode, pub message: String }
impl FfiError {
    pub fn new(code: ErrorCode, message: &str) -> Self;
    pub fn code(&self) -> &ErrorCode;
    pub fn message(&self) -> &str;
}

pub type FfiResult<T> = Result<T, FfiError>;

// C-style thread-local last error
pub fn set_last_error(err: FfiError) -> ErrorCode;
pub fn get_last_error() -> Option<FfiError>;
pub fn get_last_error_code() -> ErrorCode;
pub fn clear_last_error();
```

### ZeroCopySlice

```rust
pub struct ZeroCopySlice<'a, T> { data: *const T, len: usize, _marker: PhantomData<&'a T> }
impl<'a, T> ZeroCopySlice<'a, T> {
    pub fn new(data: &'a [T]) -> Self;
    pub fn as_ptr(&self) -> *const T;
    pub fn len/is_empty(&self) -> _;
    pub unsafe fn as_slice(&self) -> &[T];
}

pub struct ZeroCopyMapper { data: Vec<u8>, size: usize }
impl ZeroCopyMapper {
    pub fn from_vec(data: Vec<u8>) -> Self;
    pub fn as_ptr(&self) -> *const u8;
    pub fn size(&self) -> usize;
    pub fn as_slice(&self) -> &[u8];
}
```

### HeaderGenerator

```rust
pub struct HeaderGenerator { /* includes, typedefs, structs, functions, etc. */ }
impl HeaderGenerator {
    pub fn new() -> Self;
    pub fn add_include(&mut self, header: &str);
    pub fn add_typedef(&mut self, underlying: &str, name: &str);
    pub fn add_forward_declaration(&mut self, name: &str);
    pub fn add_struct(&mut self, layout: &StructLayout);
    pub fn add_function(&mut self, ret_type: &str, name: &str, params: &[(String, String)]);
    pub fn generate(&self, guard: &str) -> String;
}
```

---

## How It Works

### Architecture

```
┌──────────────────────────────────────────────────┐
│  C / C++ / Python / Wasm caller                  │
└──────────────┬───────────────────────────────────┘
               │  C ABI (pointers, u64 handles)
┌──────────────▼───────────────────────────────────┐
│  FFI Boundary Layer                               │
│  SafeCString  ·  CVec<T>  ·  SafePtr<T>          │
│  Callback  ·  OpaqueHandle  ·  FfiError           │
└──────────────┬───────────────────────────────────┘
               │  Safe Rust types
┌──────────────▼───────────────────────────────────┐
│  Rust library (lau-* crates)                      │
└──────────────────────────────────────────────────┘
```

### C String Lifecycle

```
Rust String → SafeCString::new() → as_ptr() → [FFI] → from_raw() → SafeCString
```

`SafeCString` ensures no embedded NUL bytes (checked at construction), and `from_raw` returns `None` for null pointers instead of UB.

### CVec Ownership Transfer

```
Vec<T> → CVec::from_vec() → [memory forgotten by Vec] → [FFI] → CVec::into_vec() → Vec<T>
```

`from_vec` calls `mem::forget` on the source Vec so its destructor doesn't run. The `CVec`'s `Drop` impl reconstructs the Vec and drops it properly. If transferred to C, the C code must call back to `cvec_free` to avoid leaks.

### Struct Layout Computation

`StructLayoutBuilder` mimics what a C compiler does:

1. For each field, compute `padding = align_to(current_offset, field.alignment) - current_offset`
2. Place the field at `current_offset + padding`
3. Advance `current_offset` by `padding + field.size`
4. After all fields, add tail padding so `total_size` is a multiple of `max_alignment`

This produces the same layout as `#[repr(C)]` on the Rust side.

### Opaque Handles

Instead of passing raw pointers to C (which can be dereferenced, freed, or reused), `HandleTable` issues `u64` IDs:

```rust
let handle = table.insert(my_object);
// C receives handle.id() as uint64_t
// C never sees a pointer — only the Rust side can resolve it
```

Handle IDs are monotonically increasing (wrapping at u64::MAX), so stale handles from removed entries can't accidentally alias new ones (though the table does check existence).

### Error Propagation (C-style)

For functions that return error codes (not `FfiResult`):

```rust
fn c_api_do_thing(x: i32) -> ErrorCode {
    match do_thing(x) {
        Ok(_) => ErrorCode::Success,
        Err(e) => set_last_error(e),  // stores in thread-local
    }
}
// C caller: if result != Success, call get_last_error_message()
```

### Zero-Copy Borrowing

`ZeroCopySlice<T>` wraps a `&[T]` and exposes `as_ptr()` + `len()`:

```rust
let data: Vec<f32> = /* ... */;
let zc = ZeroCopySlice::new(&data);
// zc.as_ptr() points directly into data's buffer — no copy
// zc borrows data, so data can't be mutated while zc is alive
```

### Header Generation

`HeaderGenerator` produces a complete `.h` file:

1. Include guard (`#ifndef / #define / #endif`)
2. `#include` directives
3. `extern "C"` wrapper for C++ compatibility
4. `typedef` statements
5. Forward declarations (`struct Foo;`)
6. Struct definitions with `_Static_assert(sizeof(struct Foo) == N, "...")` for ABI verification
7. Function declarations

---

## The Math

### Memory Alignment

Alignment is the power-of-two boundary that a data type's address must be a multiple of. For a type with alignment A, valid addresses are 0, A, 2A, 3A, ...

The alignment formula:

$$\text{aligned}(offset, A) = \lceil offset / A \rceil \times A = (offset + A - 1) \ \& \ !(A - 1)$$

Padding between two fields:

$$\text{padding}(offset, A) = \text{aligned}(offset, A) - offset$$

### Struct Size

For a struct with fields $f_1, f_2, ..., f_n$ with sizes $s_i$ and alignments $a_i$:

$$\text{offset}_1 = 0, \quad \text{offset}_i = \text{aligned}(\text{offset}_{i-1} + s_{i-1},\ a_i)$$

$$\text{total\_size} = \text{aligned}(\text{offset}_n + s_n,\ \max(a_1, ..., a_n))$$

The tail padding ensures arrays of the struct maintain alignment.

### Handle Address Space

With 64-bit IDs and monotonically increasing generation:

$$\text{handle\_id}_{n+1} = \text{handle\_id}_n + 1 \pmod{2^{64}}$$

At 1 billion handle creations per second, the space lasts ~584 years. In practice, handle tables are scoped to process lifetimes.

### Zero-Copy Transfer Cost

For a slice of N elements of type T:

| Method | Allocations | Copy cost |
|---|---|---|
| Marshal + unmarshal | 2 | O(N) |
| `ZeroCopySlice` | 0 | O(1) |

The zero-copy approach is strictly better for large buffers — the pointer and length are the only data that crosses the boundary.

---

## Testing

**62 tests** across 10 modules:

| Module | Tests | What's covered |
|---|---|---|
| `cstring` | 8 | Round-trip, null pointer, empty, bytes, NUL error, display, clone, serde |
| `cvec` | 5 | Vec round-trip, indexing, empty, raw parts, slice access |
| `raw_ptr` | 3 | Null, read/write, from_ref |
| `layout` | 8 | Align/padding math, struct builder (simple, nested, array fields), serialization |
| `wasm32_types` | 6 | WasmPtr read/write, WasmSlice, WasmBuffer, WasmType signatures |
| `callback` | 5 | Call, closure capture, trampoline, C signature generation |
| `handle` | 6 | Insert, get, get_mut, remove, contains, uniqueness |
| `error` | 8 | Success/error creation, last-error (set/get/clear), error codes |
| `zero_copy` | 8 | Borrowing, empty, mapper, serialization, thread-safety marker |
| `header_gen` | 5 | Basic generation, struct output, function declarations, void params, forward declarations |

Run:

```bash
cargo test
```

---

## License

MIT
