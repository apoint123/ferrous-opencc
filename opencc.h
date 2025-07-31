#ifndef FERROUS_OPENCC_FFI_H
#define FERROUS_OPENCC_FFI_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// FFI 函数的通用返回码。
enum class OpenCCResult : int32_t {
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
};

/// OpenCC 的不透明句柄。
struct OpenCCHandle;

extern "C" {

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
OpenCCResult opencc_create(const char *config_name,
                           OpenCCHandle **out_handle);

/// 销毁 OpenCC 实例，并释放所有资源。
///
/// # Safety
/// - `handle_ptr` 必须是一个有效指针。
/// - 在调用此函数后，`handle_ptr` 将变为无效指针，不应再次使用。
void opencc_destroy(OpenCCHandle *handle_ptr);

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
char *opencc_convert(const OpenCCHandle *handle_ptr, const char *text);

/// 释放返回的字符串内存。
///
/// # Safety
/// - `s_ptr` 必须是通过 `opencc_convert` 返回的有效指针，或者是 `NULL`。
/// - `s_ptr` 只能被释放一次，重复释放会导致未定义行为。
/// - 在调用此函数后，`s_ptr` 将变为无效指针，不应再次使用。
/// - 传入不是由 `opencc_convert` 分配的指针会导致未定义行为。
void opencc_free_string(char *s_ptr);

}  // extern "C"

#endif  // FERROUS_OPENCC_FFI_H
