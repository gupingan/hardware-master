//! Windows 注册表操作模块

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ, REG_DWORD, REG_SZ,
};

use super::string::wide_str;

/// 读取注册表字符串值
///
/// 从指定注册表项读取字符串值。
///
/// * `hkey` - 根键
/// * `sub_key` - 子键路径
/// * `value_name` - 值名称
///
/// 示例
/// ```ignore
/// use hardware_master::utils::registry::read_registry_string;
/// use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
///
/// let value = unsafe {
///     read_registry_string(HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0", "ProcessorNameString")
/// };
/// ```
pub unsafe fn read_registry_string(
    hkey: HKEY,
    sub_key: &str,
    value_name: &str,
) -> Option<String> {
    let mut key_handle = HKEY::default();
    let mut path = wide_str(sub_key);

    if RegOpenKeyExW(
        hkey,
        windows::core::PWSTR(path.as_mut_ptr()),
        None,
        KEY_READ,
        &mut key_handle,
    )
    .is_err()
    {
        return None;
    }

    let _guard = scopeguard::guard(key_handle, |h| {
        let _ = RegCloseKey(h);
    });

    let mut value_name_wide = wide_str(value_name);
    let mut buffer_type = REG_SZ;
    let mut buffer_size = 0u32;

    // 第一次调用：获取大小
    let result = RegQueryValueExW(
        key_handle,
        windows::core::PWSTR(value_name_wide.as_mut_ptr()),
        None,
        Some(&mut buffer_type),
        None,
        Some(&mut buffer_size),
    );

    if !result.is_ok() || buffer_size == 0 {
        return None;
    }

    // 第二次调用：读取数据
    let mut buffer: Vec<u16> = vec![0; (buffer_size as usize) / 2];
    let result = RegQueryValueExW(
        key_handle,
        windows::core::PWSTR(value_name_wide.as_mut_ptr()),
        None,
        Some(&mut buffer_type),
        Some(buffer.as_mut_slice().as_mut_ptr() as _),
        Some(&mut buffer_size),
    );

    if result.is_ok() {
        // 移除末尾的 \0
        if let Some(pos) = buffer.iter().position(|&c| c == 0) {
            buffer.truncate(pos);
        }
        Some(OsString::from_wide(&buffer).to_string_lossy().into_owned())
    } else {
        None
    }
}

/// 读取注册表 DWORD (u32) 值
///
/// 从指定注册表项读取 DWORD 值。
///
/// * `hkey` - 根键
/// * `sub_key` - 子键路径
/// * `value_name` - 值名称
///
/// 示例
/// ```ignore
/// use hardware_master::utils::registry::read_registry_dword;
/// use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
///
/// let value = unsafe {
///     read_registry_dword(HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0", "~MHz")
/// };
/// ```
pub unsafe fn read_registry_dword(hkey: HKEY, sub_key: &str, value_name: &str) -> Option<u32> {
    let mut key_handle = HKEY::default();
    let mut path = wide_str(sub_key);

    if RegOpenKeyExW(
        hkey,
        windows::core::PWSTR(path.as_mut_ptr()),
        None,
        KEY_READ,
        &mut key_handle,
    )
    .is_err()
    {
        return None;
    }

    let _guard = scopeguard::guard(key_handle, |h| {
        let _ = RegCloseKey(h);
    });

    let mut value_name_wide = wide_str(value_name);
    let mut buffer_type = REG_DWORD;
    let mut buffer: u32 = 0;
    let mut buffer_size = std::mem::size_of::<u32>() as u32;

    let result = RegQueryValueExW(
        key_handle,
        windows::core::PWSTR(value_name_wide.as_mut_ptr()),
        None,
        Some(&mut buffer_type),
        Some(&mut buffer as *mut _ as _),
        Some(&mut buffer_size),
    );

    if result.is_ok() {
        Some(buffer)
    } else {
        None
    }
}
