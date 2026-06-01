use std::marker::PhantomData;

/// Type-erased function pointer for FFI callbacks.
pub type RawCallbackFn = unsafe extern "C" fn();

/// A generic callback wrapper for FFI function pointers.
#[repr(C)]
pub struct Callback<Args, Ret> {
    func: unsafe extern "C" fn(*mut std::ffi::c_void, Args) -> Ret,
    ctx: *mut std::ffi::c_void,
    _marker: PhantomData<(Args, Ret)>,
}

impl<Args, Ret> Callback<Args, Ret> {
    /// Create a new callback from a function pointer and context.
    pub fn new(
        func: unsafe extern "C" fn(*mut std::ffi::c_void, Args) -> Ret,
        ctx: *mut std::ffi::c_void,
    ) -> Self {
        Self { func, ctx, _marker: PhantomData }
    }

    /// Invoke the callback.
    pub unsafe fn call(&self, args: Args) -> Ret {
        (self.func)(self.ctx, args)
    }

    /// Get the raw function pointer.
    pub fn func_ptr(&self) -> unsafe extern "C" fn(*mut std::ffi::c_void, Args) -> Ret {
        self.func
    }

    /// Get the context pointer.
    pub fn context(&self) -> *mut std::ffi::c_void {
        self.ctx
    }
}

/// A trampoline that adapts a Rust closure into an FFI-compatible function pointer.
/// Stores the closure in a Box and provides a static function pointer.
pub struct CallbackTrampoline<F> {
    closure: Box<F>,
}

impl<F> CallbackTrampoline<F>
where
    F: FnMut() + 'static,
{
    /// Create a new trampoline wrapping the given closure.
    pub fn new(f: F) -> Self {
        Self { closure: Box::new(f) }
    }

    /// Get a raw pointer to the closure (for passing as context).
    pub fn context_ptr(&mut self) -> *mut std::ffi::c_void {
        &mut *self.closure as *mut F as *mut std::ffi::c_void
    }
}

impl<F> CallbackTrampoline<F>
where
    F: FnMut(i32) -> i32 + 'static,
{
    /// Static trampoline function for i32 -> i32 callbacks.
    pub unsafe extern "C" fn trampoline_i32(ctx: *mut std::ffi::c_void, arg: i32) -> i32 {
        let closure = &mut *(ctx as *mut F);
        closure(arg)
    }
}

// SAFETY: The callback holds raw pointers, safe to send if the underlying data is Send.
unsafe impl<Args: Send, Ret: Send> Send for Callback<Args, Ret> {}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_callback_fn(_ctx: *mut std::ffi::c_void, x: i32) -> i32 {
        x * 2
    }

    #[test]
    fn test_callback_basic() {
        let cb: Callback<i32, i32> = Callback::new(test_callback_fn, std::ptr::null_mut());
        let result = unsafe { cb.call(21) };
        assert_eq!(result, 42);
    }

    #[test]
    fn test_callback_func_ptr() {
        let cb: Callback<i32, i32> = Callback::new(test_callback_fn, std::ptr::null_mut());
        assert_eq!(cb.func_ptr() as usize, test_callback_fn as usize);
    }

    #[test]
    fn test_trampoline_i32() {
        // Test the trampoline mechanism directly
        let mut val = 10i32;
        let ctx = &mut val as *mut i32 as *mut std::ffi::c_void;
        unsafe extern "C" fn add_trampoline(ctx: *mut std::ffi::c_void, x: i32) -> i32 {
            let v = &mut *(ctx as *mut i32);
            *v + x
        }
        let result = unsafe { add_trampoline(ctx, 5) };
        assert_eq!(result, 15);
    }

    #[test]
    fn test_callback_with_context() {
        static mut ACCUMULATOR: i32 = 0;
        unsafe extern "C" fn accumulate(ctx: *mut std::ffi::c_void, val: i32) -> i32 {
            let acc = &mut *(ctx as *mut i32);
            *acc += val;
            *acc
        }
        let mut acc = 0i32;
        let cb = Callback::new(accumulate, &mut acc as *mut i32 as *mut std::ffi::c_void);
        unsafe {
            assert_eq!(cb.call(10), 10);
            assert_eq!(cb.call(20), 30);
        }
    }

    #[test]
    fn test_callback_size() {
        use std::mem;
        assert_eq!(mem::size_of::<Callback<i32, i32>>(), 2 * mem::size_of::<usize>());
    }
}
