//! Windows 设备操作模块

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::RawHandle;
use windows::core::{GUID, PCWSTR};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiGetClassDevsW, SetupDiGetDeviceInstanceIdW,
    SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW, DIGCF_DEVICEINTERFACE,
    DIGCF_PRESENT, HDEVINFO, SETUP_DI_GET_CLASS_DEVS_FLAGS, SP_DEVICE_INTERFACE_DATA,
    SP_DEVICE_INTERFACE_DETAIL_DATA_W,
};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiEnumDeviceInterfaces, SETUP_DI_REGISTRY_PROPERTY, SP_DEVINFO_DATA,
};
use windows::Win32::Foundation::{
    GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, HANDLE,
};
use windows::Win32::System::IO::DeviceIoControl;

use super::string::pwstr_to_string;
use crate::detector::DetectionError;

/// 执行 DeviceIoControl 并返回字节 Vec
///
/// * `handle` - 设备句柄
/// * `ioctl_code` - IOCTL 控制码
/// * `in_buf` - 输入缓冲区
/// * `out_buf_len` - 输出缓冲区长度
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::device_io_control;
///
/// unsafe {
///     let result = device_io_control(
///         handle,
///         IOCTL_BATTERY_QUERY_INFORMATION,
///         &input_bytes,
///         mem::size_of::<BATTERY_INFORMATION>(),
///     )?;
/// }
/// ```
pub unsafe fn device_io_control(
    handle: RawHandle,
    ioctl_code: u32,
    in_buf: &[u8],
    out_buf_len: usize,
) -> Result<Vec<u8>, DetectionError> {
    let mut bytes_returned = 0u32;
    let mut out_buf = vec![0u8; out_buf_len];

    let success = DeviceIoControl(
        HANDLE(handle),
        ioctl_code,
        Some(in_buf.as_ptr() as _),
        in_buf.len() as u32,
        Some(out_buf.as_mut_ptr() as _),
        out_buf_len as u32,
        Some(&mut bytes_returned),
        None,
    );

    if success.is_ok() {
        out_buf.truncate(bytes_returned as usize);
        Ok(out_buf)
    } else {
        Err(DetectionError::WindowsApiError(format!(
            "DeviceIoControl 失败 (code: 0x{:08X}): {:?}",
            ioctl_code,
            GetLastError()
        )))
    }
}

/// 设备实例 ID 获取
///
/// 通用的设备实例 ID 获取函数，可被多个模块复用。
///
/// * `device_info_set` - 设备信息集句柄
/// * `device_info_data` - 设备信息数据
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::get_device_instance_id;
///
/// let instance_id = unsafe {
///     get_device_instance_id(device_info_set, &device_info_data)?
/// };
/// ```
pub unsafe fn get_device_instance_id(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
) -> Result<String, DetectionError> {
    let mut instance_id_buffer: [u16; 256] = [0; 256];
    let mut required_size = 0u32;

    SetupDiGetDeviceInstanceIdW(
        device_info_set,
        device_info_data,
        Some(&mut instance_id_buffer),
        Some(&mut required_size),
    )
    .map_err(|e| {
        DetectionError::WindowsApiError(format!("SetupDiGetDeviceInstanceIdW 失败: {:?}", e))
    })?;

    Ok(pwstr_to_string(windows::core::PWSTR(
        instance_id_buffer.as_mut_ptr(),
    )))
}

/// 设备实例 ID 信息结构体
#[derive(Debug, Clone)]
pub struct DeviceInstanceIdInfo {
    /// 原始设备实例 ID (例如: `PCI\VEN_10EC&DEV_8168&SUBSYS_12341462&REV_06\4&12a3b456&0&00E5`)
    pub id: String,
    /// 总线类型 (例如: PCI)
    pub bus_type: String,
    /// 厂商标识符 (例如: 10EC)
    pub vendor_id: String,
    /// 设备标识符 (例如: 8168)
    pub device_id: String,
    /// 子系统厂商标识符 (例如: 1234)
    pub subsystem_vendor_id: String,
    /// 子系统设备标识符 (例如: 8168)
    pub subsystem_device_id: String,
    /// 修订版本号 (例如: 06)
    pub revision_id: String,
    /// 总线编号 (实例部分)
    pub bus_number: String,
    /// 实例 ID
    pub instance_id: String,
    /// 特征码/唯一序列号 (实例部分)
    pub feature_code: String,
    /// 设备编号 (实例部分)
    pub device_number: String,
    /// 功能编号 (实例部分)
    pub function_number: String,
}

/// 解析设备实例 ID，返回设备实例 ID 信息结构体
///
/// * `dev_ins_id` - 设备实例 ID 字符串
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::parse_device_instance_id;
///
/// let info = parse_device_instance_id("PCI\\VEN_10EC&DEV_8168&SUBSYS_12341462&REV_06\\4&12a3b456&0&00E5");
/// assert_eq!(info.vendor_id, "10EC");
/// assert_eq!(info.device_id, "8168");
/// ```
pub fn parse_device_instance_id(dev_ins_id: &str) -> DeviceInstanceIdInfo {
    let mut info = DeviceInstanceIdInfo {
        id: dev_ins_id.to_string(),
        bus_type: "未知".to_string(),
        vendor_id: "未知".to_string(),
        device_id: "未知".to_string(),
        subsystem_vendor_id: "未知".to_string(),
        subsystem_device_id: "未知".to_string(),
        revision_id: "未知".to_string(),
        bus_number: "未知".to_string(),
        instance_id: "未知".to_string(),
        feature_code: "未知".to_string(),
        device_number: "未知".to_string(),
        function_number: "未知".to_string(),
    };

    // BUS_TYPE\HW_PARAMS\INSTANCE_PARAMS
    let parts: Vec<&str> = dev_ins_id.split('\\').collect();

    if parts.is_empty() {
        return info;
    }

    // 总线类型
    info.bus_type = parts[0].to_string();

    // 硬件参数
    if parts.len() > 1 {
        let hw_params = parts[1];
        for pair in hw_params.split('&') {
            if let Some((key, value)) = pair.split_once('_') {
                match key {
                    "VEN" => info.vendor_id = value.to_string(),
                    "DEV" => info.device_id = value.to_string(),
                    "SUBSYS" => {
                        if value.len() >= 4 {
                            info.subsystem_device_id = value[..4].to_string();
                            info.subsystem_vendor_id = value[4..].to_string();
                        }
                    }
                    "REV" => info.revision_id = value.to_string(),
                    _ => {}
                }
            }
        }
    }

    // 实例 ID
    if parts.len() > 2 {
        let instance_params = parts[2];
        info.instance_id = instance_params.to_string();

        let instance_parts: Vec<&str> = instance_params.split('&').collect();

        if instance_parts.len() >= 1 {
            info.bus_number = instance_parts[0].to_string();
        }
        if instance_parts.len() >= 2 {
            info.feature_code = instance_parts[1].to_string();
        }
        if instance_parts.len() >= 3 {
            info.device_number = instance_parts[2].to_string();
        }
        if instance_parts.len() >= 4 {
            info.function_number = instance_parts[3].to_string();
        }
    }

    info
}

/// 获取设备注册表属性
///
/// * `device_info_set` - 设备信息集句柄
/// * `device_info_data` - 设备信息数据
/// * `property` - 要查询的属性（如 SPDRP_FRIENDLYNAME、SPDRP_DEVICEDESC 等）
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::get_device_property;
/// use windows::Win32::Devices::DeviceAndDriverInstallation::SPDRP_FRIENDLYNAME;
///
/// let friendly_name = unsafe {
///     get_device_property(device_info_set, &device_info_data, SPDRP_FRIENDLYNAME)
/// };
/// ```
pub unsafe fn get_device_property(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Option<String> {
    let mut buffer_type: u32 = 0;
    let mut required_size = 0u32;

    // 第一次调用获取所需缓冲区大小
    let _ = SetupDiGetDeviceRegistryPropertyW(
        device_info_set,
        device_info_data,
        property,
        Some(&mut buffer_type),
        None,
        Some(&mut required_size),
    );

    if required_size == 0 {
        return None;
    }

    // 分配缓冲区（required_size 是字节数）
    let mut buffer: Vec<u8> = vec![0; required_size as usize];

    // 第二次调用获取实际数据，并更新 required_size 为实际写入的字节数
    let result = SetupDiGetDeviceRegistryPropertyW(
        device_info_set,
        device_info_data,
        property,
        Some(&mut buffer_type),
        Some(&mut buffer),
        Some(&mut required_size),
    );

    if result.is_ok() {
        // 将字节数据转换为 UTF-16
        let u16_len = required_size as usize / 2;
        let u16_slice = std::slice::from_raw_parts(buffer.as_ptr() as *const u16, u16_len);

        // 移除末尾的空字符
        if let Some(pos) = u16_slice.iter().position(|&c| c == 0) {
            let trimmed = &u16_slice[..pos];
            Some(OsString::from_wide(trimmed).to_string_lossy().into_owned())
        } else {
            Some(
                OsString::from_wide(u16_slice)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    } else {
        None
    }
}

/// 获取指定设备类的设备信息集
///
/// 通用的设备信息集获取函数，可被多个模块复用。
///
/// * `class_guid` - 设备类的 GUID
/// * `flags` - 获取设备信息的标志
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::get_device_info_set;
/// use windows::Win32::Devices::DeviceAndDriverInstallation::GUID_DEVCLASS_BATTERY;
///
/// let device_info_set = unsafe {
///     get_device_info_set(&GUID_DEVCLASS_BATTERY)?
/// };
/// ```
pub unsafe fn get_device_info_set(
    class_guid: *const GUID,
    flags: SETUP_DI_GET_CLASS_DEVS_FLAGS,
) -> Result<HDEVINFO, DetectionError> {
    let device_info_set = SetupDiGetClassDevsW(Some(class_guid), PCWSTR::null(), None, flags)
        .map_err(|e| {
            DetectionError::WindowsApiError(format!("SetupDiGetClassDevsW 失败: {:?}", e))
        })?;

    if device_info_set.is_invalid() {
        let err = GetLastError();
        return Err(DetectionError::WindowsApiError(format!(
            "SetupDiGetClassDevsW 返回无效句柄: {:?}",
            err
        )));
    }

    Ok(device_info_set)
}

/// 获取设备接口的路径
///
/// * `device_info_set` - 设备信息集句柄
/// * `device_interface_data` - 设备接口数据
/// * `class_guid` - 设备类的 GUID
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::get_device_interface_path;
///
/// let device_path = unsafe {
///     get_device_interface_path(
///         device_info_set,
///         &mut did,
///         &GUID_DEVCLASS_BATTERY,
///     )?
/// };
/// ```
pub unsafe fn get_device_interface_path(
    device_info_set: HDEVINFO,
    device_interface_data: &SP_DEVICE_INTERFACE_DATA,
) -> Result<String, DetectionError> {
    // 获取设备路径所需缓冲大小
    let mut needed_size = 0u32;
    let _ = SetupDiGetDeviceInterfaceDetailW(
        device_info_set,
        device_interface_data,
        None,
        0,
        Some(&mut needed_size),
        None,
    );
    let err = GetLastError();
    if err != ERROR_INSUFFICIENT_BUFFER {
        return Err(DetectionError::WindowsApiError(format!(
            "SetupDiGetDeviceInterfaceDetailW (获取缓冲大小) 错误: {:?}",
            err
        )));
    }

    let mut detail_data_buf = vec![0u8; needed_size as usize];
    let pdidd = &mut *(detail_data_buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W);
    pdidd.cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

    // 获取设备路径
    if SetupDiGetDeviceInterfaceDetailW(
        device_info_set,
        device_interface_data,
        Some(pdidd),
        needed_size,
        None,
        None,
    )
    .is_err()
    {
        return Err(DetectionError::WindowsApiError(
            "SetupDiGetDeviceInterfaceDetailW 失败".to_string(),
        ));
    }

    // 获取设备路径字符串
    let device_path_offset = mem::size_of::<u32>();
    let device_path_ptr = detail_data_buf.as_ptr().add(device_path_offset) as *const u16;

    // 计算最大可能的路径长度
    let max_path_len = (detail_data_buf.len() - device_path_offset) / mem::size_of::<u16>();
    let device_path_slice = std::slice::from_raw_parts(device_path_ptr, max_path_len);

    // 找到第一个 null 终止符
    let device_path_len = device_path_slice
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(max_path_len);
    let device_path = String::from_utf16_lossy(&device_path_slice[..device_path_len]);

    Ok(device_path)
}

/// 枚举设备接口并返回设备路径列表
///
/// * `class_guid` - 设备类的 GUID
///
/// 示例
/// ```ignore
/// use hardware_master::utils::device::enumerate_device_paths;
/// use windows::Win32::Devices::DeviceAndDriverInstallation::GUID_DEVCLASS_BATTERY;
///
/// let paths = unsafe {
///     enumerate_device_paths(&GUID_DEVCLASS_BATTERY)?
/// };
/// ```
pub unsafe fn enumerate_device_paths(
    class_guid: *const GUID,
) -> Result<Vec<String>, DetectionError> {
    let mut paths = Vec::new();

    let device_info_set =
        get_device_info_set(class_guid, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)?;

    let mut did = SP_DEVICE_INTERFACE_DATA {
        cbSize: mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
        ..Default::default()
    };

    // 限制最多枚举 100 个设备
    for index in 0..100 {
        if SetupDiEnumDeviceInterfaces(device_info_set, None, class_guid, index, &mut did).is_err()
        {
            let err = GetLastError();
            if err == ERROR_NO_MORE_ITEMS {
                break;
            }
            log::warn!("SetupDiEnumDeviceInterfaces 错误: {:?}", err);
            continue;
        }

        match get_device_interface_path(device_info_set, &did) {
            Ok(path) => paths.push(path),
            Err(e) => {
                log::warn!("获取设备路径失败: {:?}", e);
                continue;
            }
        }
    }

    // 清理设备信息集
    let _ = SetupDiDestroyDeviceInfoList(device_info_set);

    Ok(paths)
}
