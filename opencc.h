#ifndef FERROUS_OPENCC_FFI_H
#define FERROUS_OPENCC_FFI_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Common return codes for FFI functions.
enum class OpenCCResult : int32_t {
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
};

/// All built-in `OpenCC` configurations
enum class BuiltinConfig : int32_t {
  /// Simplified to Traditional
  S2t = 0,
  /// Traditional to Simplified
  T2s = 1,
  /// Simplified to Traditional (Taiwan)
  S2tw = 2,
  /// Traditional (Taiwan) to Simplified
  Tw2s = 3,
  /// Simplified to Traditional (Hong Kong)
  S2hk = 4,
  /// Traditional (Hong Kong) to Simplified
  Hk2s = 5,
  /// Simplified to Traditional (Taiwan) (including vocabulary conversion)
  S2twp = 6,
  /// Traditional (Taiwan) (including vocabulary conversion) to Simplified
  Tw2sp = 7,
  /// Traditional to Traditional (Taiwan)
  T2tw = 8,
  /// Traditional (Taiwan) to Traditional
  Tw2t = 9,
  /// Traditional to Traditional (Hong Kong)
  T2hk = 10,
  /// Traditional (Hong Kong) to Traditional
  Hk2t = 11,
  /// Japanese Shinjitai to Traditional
  Jp2t = 12,
  /// Traditional to Japanese Shinjitai
  T2jp = 13,
};

/// Opaque handle for `OpenCC`.
struct OpenCCHandle;

extern "C" {

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
OpenCCResult opencc_create(BuiltinConfig config, OpenCCHandle **out_handle);

/// Destroys the `OpenCC` instance and releases all resources.
///
/// # Safety
/// - `handle_ptr` must be a valid pointer.
/// - After calling this function, `handle_ptr` becomes invalid and should not be used again.
void opencc_destroy(OpenCCHandle *handle_ptr);

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
char *opencc_convert(const OpenCCHandle *handle_ptr, const char *text);

/// Frees the memory of the returned string.
///
/// # Safety
/// - `s_ptr` must be a valid pointer returned by `opencc_convert`, or `NULL`.
/// - `s_ptr` can only be freed once; double freeing causes undefined behavior.
/// - After calling this function, `s_ptr` becomes invalid and should not be used again.
/// - Passing a pointer not allocated by `opencc_convert` causes undefined behavior.
void opencc_free_string(char *s_ptr);

}  // extern "C"

#endif  // FERROUS_OPENCC_FFI_H
