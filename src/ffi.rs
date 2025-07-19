//! Ferrous OpenCC 的 FFI 接口。

use crate::OpenCC;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

/// FFI 函数的通用返回码。
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum OpenCCResult {
    /// 操作成功。
    Success = 0,

    /// 传入的句柄无效。
    InvalidHandle = 1,

    /// 传入的参数无效。
    InvalidArgument = 2,

    /// OpenCC 实例创建失败（找不到配置文件之类的）。
    CreationFailed = 3,

    /// 发生了一个未预料的错误（通常是 panic）。
    InternalError = 4,
}

/// OpenCC 的不透明句柄。
pub struct OpenCCHandle {
    /// 核心的 OpenCC 实例。
    instance: OpenCC,

    /// 一个原子标志，用于防止双重释放。
    is_destroyed: AtomicBool,
}

/// 从嵌入的资源创建 OpenCC 实例。
///
/// # 参数
/// - `config_name`: 一个指向字符串的指针，代表配置文件的名称。
/// - `out_handle`: 一个指向 `*mut OpenCCHandle` 的指针，用于接收成功创建的句柄。
///
/// # 返回
/// - `OpenCCResult::Success` 表示成功，`out_handle` 将被设置为有效的句柄。
/// - 其他 `OpenCCResult` 枚举值表示失败，`out_handle` 将被设置为 `NULL`。
///
/// # Safety
/// - `config_name` 必须指向一个有效的、以空字符结尾的 C 字符串。
/// - `out_handle` 必须指向一个有效的 `*mut OpenCCHandle` 内存位置。
/// - 返回的句柄必须在不再需要时通过 `opencc_destroy` 释放，以避免资源泄漏。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_create(
    config_name: *const c_char,
    out_handle: *mut *mut OpenCCHandle,
) -> OpenCCResult {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if out_handle.is_null() {
            return OpenCCResult::InvalidArgument;
        }
        unsafe { *out_handle = std::ptr::null_mut() };

        if config_name.is_null() {
            return OpenCCResult::InvalidArgument;
        }

        let c_str = unsafe { CStr::from_ptr(config_name) };
        let r_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return OpenCCResult::InvalidArgument,
        };

        match OpenCC::from_config_name(r_str) {
            Ok(instance) => {
                let handle = Box::new(OpenCCHandle {
                    instance,
                    is_destroyed: AtomicBool::new(false),
                });
                unsafe { *out_handle = Box::into_raw(handle) };
                OpenCCResult::Success
            }
            Err(_) => OpenCCResult::CreationFailed,
        }
    }));
    result.unwrap_or(OpenCCResult::InternalError)
}

/// 销毁 OpenCC 实例，并释放所有资源。
///
/// # Safety
/// - `handle_ptr` 必须是一个有效指针。
/// - 在调用此函数后，`handle_ptr` 将变为无效指针，不应再次使用。
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
        // 没有日志库，只能直接打印了
        eprintln!("opencc_destroy 内部发生 Panic！");
    }
}

/// 根据加载的配置转换字符串。
///
/// # 参数
/// - `handle_ptr`: 指向有效 `OpenCCHandle` 实例的指针。
/// - `text`: 一个指向需要转换的字符串的指针。
///
/// # 返回
/// - 成功时，返回一个指向新的、转换后的 UTF-8 字符串的指针。
/// - 如果句柄无效、输入文本为 `NULL` 或发生内部错误，则返回 `NULL`。
///
/// # 注意
/// 返回的字符串在堆上分配，你需要在使用完毕后调用 `opencc_free_string`
/// 来释放它，否则将导致内存泄漏。
///
/// # Safety
/// - `handle_ptr` 必须指向一个有效的、尚未被销毁的 `OpenCCHandle`。
/// - `text` 必须指向一个有效的、以空字符结尾的 C 字符串。
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
        let r_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        let converted_string = handle.instance.convert(r_str);

        match CString::new(converted_string) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }));

    result.unwrap_or_else(|_| {
        // 没有日志库，只能直接打印了
        eprintln!("opencc_convert 内部发生 Panic！");
        std::ptr::null_mut()
    })
}

/// 释放返回的字符串内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn opencc_free_string(s_ptr: *mut c_char) {
    if s_ptr.is_null() {
        return;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        unsafe { drop(CString::from_raw(s_ptr)) };
    }));

    if result.is_err() {
        // 没有日志库，只能直接打印了
        eprintln!("opencc_free_string 内部发生 Panic！");
    }
}
