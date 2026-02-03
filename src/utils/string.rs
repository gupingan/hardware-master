//! 字符串转换工具模块

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PWSTR;

/// 移除切片末尾所有的零值元素。
///
/// 用于处理来自底层 API（如 Windows API）返回的定长数组。
/// 例如将 `[u16; 256]` 转换为动态的 `Vec<u16>`，并截断末尾所有的空终止符 `\0`。
///
/// 注意：仅移除末尾的 0，中间的 0 会被保留。
///
/// # 类型参数
///
/// * `T` - 切片元素的类型。必须实现 `Copy` 和 `PartialEq`，且必须有默认值为 0（通常是整数类型）。
///
/// # 参数
///
/// * `v` - 输入的切片引用。
///
/// # 返回值
///
/// 返回一个新的 `Vec<T>`，包含原切片中移除了末尾所有零值后的内容。
///
/// # 示例
///
/// ```ignore
/// // 示例 1: 标准 Windows 字符串
/// let data: Vec<u16> = vec![72, 101, 108, 108, 111, 0, 0, 0]; // "Hello\0\0\0"
/// let trimmed = _trim_zeros(&data);
/// assert_eq!(trimmed, vec![72, 101, 108, 108, 111]); // 仅保留 Hello
///
/// // 示例 2: 保留中间的 0
/// let data: Vec<u32> = vec![1, 0, 2, 3, 0, 0];
/// let trimmed = _trim_zeros(&data);
/// assert_eq!(trimmed, vec![1, 0, 2, 3]); // 末尾两个0被移除，中间的0保留
///
/// // 示例 3: 全是 0
/// let data: Vec<u8> = vec![0, 0, 0];
/// let trimmed = _trim_zeros(&data);
/// assert!(trimmed.is_empty());
/// ```
fn _trim_zeros<T>(v: &[T]) -> Vec<T>
where
    T: Copy + PartialEq + Default,
{
    // 从后向前查找最后一个非零元素的索引
    if let Some(last_non_zero_index) = v.iter().rposition(|&c| c != T::default()) {
        v[..=last_non_zero_index].to_vec()
    } else {
        Vec::new()
    }
}

/// 将 UTF-16 编码的 u16 切片转换为字符串
///
/// 该函数处理 Windows API 返回的 UTF-16 字符串数据，
/// 会自动去除末尾的 null 终止符（0）。
///
/// 示例
/// ```
/// use hardware_master::utils::string::u16_slice_to_string;
/// let data = vec![72, 101, 108, 108, 111, 0];
/// assert_eq!(u16_slice_to_string(&data), "Hello");
/// ```
pub fn u16_slice_to_string(v: &[u16]) -> String {
    let trimmed = _trim_zeros(v);
    String::from_utf16_lossy(&trimmed)
}

/// 将 UTF-8 编码的 u8 切片转换为字符串
///
/// 该函数处理 UTF-8 编码的字节数据，
/// 会自动去除末尾的 null 终止符（0）。
///
/// 示例
/// ```
/// use hardware_master::utils::string::u8_slice_to_string;
/// let data = vec![72, 101, 108, 108, 111, 0];
/// assert_eq!(u8_slice_to_string(&data), "Hello");
/// ```
pub fn u8_slice_to_string(v: &[u8]) -> String {
    let trimmed = _trim_zeros(v);
    String::from_utf8_lossy(&trimmed).into_owned()
}

/// 小端序数据转换字符串，Windows 下理论上安全
///
/// 该函数用于处理 Windows API 返回的 UTF-16 字符串数据，
/// 会自动去除末尾的 null 终止符（0）。
///
/// * `bytes` - 包含 UTF-16 数据的字节切片
///
/// 示例
/// ```
/// use hardware_master::utils::string::u16_bytes_to_string;
/// // UTF-16 编码的 "Hello" + null
/// let data: Vec<u8> = vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x00, 0x00];
/// assert_eq!(u16_bytes_to_string(&data), "Hello");
/// ```
pub fn u16_bytes_to_string(bytes: &[u8]) -> String {
    if bytes.len() % 2 != 0 {
        return String::new();
    }

    let chars: &[u16] =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / 2) };

    let len = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
    String::from_utf16_lossy(&chars[..len])
}

/// 将字符串转换为 Windows 宽字符串 (Vec<u16>)
///
/// 该函数将 UTF-8 字符串编码为 UTF-16，并在末尾添加 null 终止符。
/// 这是在 Windows API 中传递字符串参数的标准格式。
///
/// 示例
/// ```
/// use hardware_master::utils::string::wide_str;
/// let wide = wide_str("Hello");
/// assert_eq!(wide, vec![72, 101, 108, 108, 111, 0]);
/// ```
pub fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 从 PWSTR 指针读取字符串
///
/// * `pwstr` - 指向以 null 结尾的 UTF-16 字符串的指针
///
/// 示例
/// ```ignore
/// use hardware_master::utils::string::pwstr_to_string;
/// use windows::core::PWSTR;
///
/// // 创建一个 UTF-16 字符串
/// let utf16_data: Vec<u16> = vec![72, 101, 108, 108, 111, 0]; // "Hello" + null
/// let pwstr = PWSTR(utf16_data.as_ptr());
///
/// let result = unsafe {
///     pwstr_to_string(pwstr)
/// };
/// assert_eq!(result, "Hello");
/// ```
pub unsafe fn pwstr_to_string(pwstr: PWSTR) -> String {
    if pwstr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *pwstr.0.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(pwstr.0, len);
    OsString::from_wide(slice).to_string_lossy().into_owned()
}

/// 格式化
pub fn format_size(size: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if size <= 0.0 {
        return "0 B".to_string();
    }
    let exp = size.log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = size as f64 / 1024f64.powi(exp as i32);
    format!("{:.0} {}", value, UNITS[exp])
}
