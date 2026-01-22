//! FFI interface for `Ferrous OpenCC`.

use std::{
    ffi::{
        CStr,
        CString,
        c_char,
    },
    panic::{
        AssertUnwindSafe,
        catch_unwind,
    },
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

use crate::{
    OpenCC,
    config::BuiltinConfig,
};

/// Common return codes for FFI functions.
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum OpenCCResult {
    /// Operation succeeded.
    Success = 0,

    /// Invalid handle passed.
    InvalidHandle = 1,

    /// Invalid argument passed.
    InvalidArgument = 2,

    /// Failed to create `OpenCC` instance (e.g., config file not found).
    CreationFailed = 3,

    /// An unexpected error occurred (usually a panic).
    InternalError = 4,
}

/// Opaque handle for `OpenCC`.
pub struct OpenCCHandle {
    /// The core `OpenCC` instance.
    instance: OpenCC,

    /// An atomic flag to prevent double-free.
    is_destroyed: AtomicBool,
}

/// Creates an `OpenCC` instance from embedded resources.
///
/// # Arguments
/// - `config`: Enum value representing the built-in configuration, e.g., `S2t`.
/// - `out_handle`: A pointer to `*mut OpenCCHandle` to receive the successfully created handle.
///
/// # Returns
/// - `OpenCCResult::Success` on success, and `out_handle` will be set to a valid handle.
/// - Other `OpenCCResult` variants indicate failure, and `out_handle` will be set to `NULL`.
///
/// # Safety
/// - `out_handle` must point to a valid `*mut OpenCCHandle` memory location.
/// - The returned handle must be freed via `opencc_destroy` when no longer needed to avoid resource
///   leaks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_create(
    config: BuiltinConfig,
    out_handle: *mut *mut OpenCCHandle,
) -> OpenCCResult {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if out_handle.is_null() {
            return OpenCCResult::InvalidArgument;
        }
        unsafe { *out_handle = std::ptr::null_mut() };

        OpenCC::from_config(config).map_or(OpenCCResult::CreationFailed, |instance| {
            let handle = Box::new(OpenCCHandle {
                instance,
                is_destroyed: AtomicBool::new(false),
            });
            unsafe { *out_handle = Box::into_raw(handle) };
            OpenCCResult::Success
        })
    }));
    result.unwrap_or(OpenCCResult::InternalError)
}

/// Destroys the `OpenCC` instance and releases all resources.
///
/// # Safety
/// - `handle_ptr` must be a valid pointer.
/// - After calling this function, `handle_ptr` becomes invalid and should not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_destroy(handle_ptr: *mut OpenCCHandle) {
    if handle_ptr.is_null() {
        return;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let handle = unsafe { &*handle_ptr };

        if handle
            .is_destroyed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            unsafe { drop(Box::from_raw(handle_ptr)) };
        }
    }));

    if result.is_err() {
        // No logging library, print directly
        eprintln!("Panic occurred inside opencc_destroy!");
    }
}

/// Converts a string according to the loaded configuration.
///
/// # Arguments
/// - `handle_ptr`: Pointer to a valid `OpenCCHandle` instance.
/// - `text`: Pointer to the string to be converted.
///
/// # Returns
/// - On success, returns a pointer to the new, converted UTF-8 string.
/// - Returns `NULL` if the handle is invalid, input text is `NULL`, or an internal error occurs.
///
/// # Note
/// The returned string is allocated on the heap. You must call `opencc_free_string`
/// to free it after use, otherwise memory leaks will occur.
///
/// # Safety
/// - `handle_ptr` must point to a valid, undestroyed `OpenCCHandle`.
/// - `text` must point to a valid, null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_convert(
    handle_ptr: *const OpenCCHandle,
    text: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if handle_ptr.is_null() {
            return std::ptr::null_mut();
        }

        let handle = unsafe { &*handle_ptr };

        if handle.is_destroyed.load(Ordering::SeqCst) {
            return std::ptr::null_mut();
        }

        if text.is_null() {
            return std::ptr::null_mut();
        }
        let c_str = unsafe { CStr::from_ptr(text) };
        let Ok(r_str) = c_str.to_str() else {
            return std::ptr::null_mut();
        };

        let converted_string = handle.instance.convert(r_str);

        CString::new(converted_string).map_or(std::ptr::null_mut(), CString::into_raw)
    }));

    result.unwrap_or_else(|_| {
        // No logging library, print directly
        eprintln!("Panic occurred inside opencc_convert!");
        std::ptr::null_mut()
    })
}

/// Frees the memory of the returned string.
///
/// # Safety
/// - `s_ptr` must be a valid pointer returned by `opencc_convert`, or `NULL`.
/// - `s_ptr` can only be freed once; double freeing causes undefined behavior.
/// - After calling this function, `s_ptr` becomes invalid and should not be used again.
/// - Passing a pointer not allocated by `opencc_convert` causes undefined behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_free_string(s_ptr: *mut c_char) {
    if s_ptr.is_null() {
        return;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        unsafe { drop(CString::from_raw(s_ptr)) };
    }));

    if result.is_err() {
        // No logging library, print directly
        eprintln!("Panic occurred inside opencc_free_string!");
    }
}
