use crate::detector::DetectionError;
use crate::utils;
use std::os::windows::io::RawHandle;
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::GUID_DEVCLASS_BATTERY;
use windows::Win32::Foundation::{CloseHandle, GetLastError, GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Power::{
    BatteryDeviceName, BatteryInformation, BatteryManufactureName, BATTERY_INFORMATION,
    BATTERY_QUERY_INFORMATION, BATTERY_QUERY_INFORMATION_LEVEL, BATTERY_TAG_INVALID,
    IOCTL_BATTERY_QUERY_INFORMATION, IOCTL_BATTERY_QUERY_TAG,
};
use windows::Win32::System::IO::DeviceIoControl;

/// 电池信息
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    /// 电池列表
    pub batteries: Vec<Battery>,
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self { batteries: vec![] }
    }
}

/// 电池
#[derive(Debug, Clone)]
pub struct Battery {
    /// 电池名称
    pub name: String,
    /// 电池健康度（百分比）
    pub health: f64,
    /// 电池化学成分
    pub chemistry: BatteryChemistry,
    /// 制造商
    pub vendor: String,
    /// 设计容量 (mWh)
    pub designed_capacity: u32,
    /// 当前满充容量 (mWh)
    pub full_charged_capacity: u32,
    /// 循环次数
    pub cycle_count: u32,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
            health: 0.0,
            chemistry: BatteryChemistry::Unknown,
            vendor: "未知".to_string(),
            designed_capacity: 0,
            full_charged_capacity: 0,
            cycle_count: 0,
        }
    }
}

/// 电池化学成分
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatteryChemistry {
    Other = 1,
    Unknown,
    LeadAcid,
    NickelCadmium,
    NickelMetalHydride,
    LithiumIon,
    ZincAir,
    LithiumPolymer,
}

impl From<&String> for BatteryChemistry {
    /// 将化学成分字符串转换为 BatteryChemistry 枚举
    fn from(value: &String) -> Self {
        match value as &str {
            "LION" | "Li-I" | "Li-Ion" => BatteryChemistry::LithiumIon,
            "LiPo" | "Li-P" => BatteryChemistry::LithiumPolymer,
            "PbAc" | "Pb" => BatteryChemistry::LeadAcid,
            "NiCd" | "Ni-Cd" => BatteryChemistry::NickelCadmium,
            "NiMH" | "Ni-Mh" => BatteryChemistry::NickelMetalHydride,
            "Zn-Air" | "Zn" => BatteryChemistry::ZincAir,
            _ => BatteryChemistry::Other,
        }
    }
}

impl ToString for BatteryChemistry {
    /// 转换为可视化字符串，比如 "NickelCadmium" -> "镍镉电池"
    fn to_string(&self) -> String {
        match self {
            BatteryChemistry::Other => "其他".to_string(),
            BatteryChemistry::Unknown => "未知".to_string(),
            BatteryChemistry::LeadAcid => "铅酸电池".to_string(),
            BatteryChemistry::NickelCadmium => "镍镉电池".to_string(),
            BatteryChemistry::NickelMetalHydride => "镍氢电池".to_string(),
            BatteryChemistry::LithiumIon => "锂离子电池".to_string(),
            BatteryChemistry::ZincAir => "锌空气电池".to_string(),
            BatteryChemistry::LithiumPolymer => "锂聚合物电池".to_string(),
        }
    }
}

/// 检测电池信息
pub fn detect_battery() -> Result<BatteryInfo, DetectionError> {
    let mut info = BatteryInfo::default();

    unsafe {
        let device_paths = utils::device::enumerate_device_paths(&GUID_DEVCLASS_BATTERY)?;

        for device_path in device_paths {
            // CreateFile 打开设备
            let handle = match CreateFileW(
                PCWSTR::from_raw(
                    device_path
                        .encode_utf16()
                        .chain([0])
                        .collect::<Vec<u16>>()
                        .as_ptr(),
                ),
                GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            ) {
                Ok(h) => h,
                Err(e) => {
                    log::warn!("CreateFileW 失败: {:?}", e);
                    continue;
                }
            };

            let _guard = scopeguard::guard(handle, |h| {
                let _ = CloseHandle(h);
            });

            let handle_raw = handle.0;

            // 查询 BatteryTag
            let tag = match query_tag(handle_raw) {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("query_tag 失败: {:?}", e);
                    continue;
                }
            };

            // 查询电池基本信息
            let battery_info = match query_information(handle_raw, tag) {
                Ok(info) => info,
                Err(e) => {
                    log::warn!("query_information 失败: {:?}", e);
                    continue;
                }
            };

            // 解析化学成分
            let chem_str = utils::u8_slice_to_string(&battery_info.Chemistry);
            let chemistry = BatteryChemistry::from(&chem_str);

            // 计算健康度
            let health = utils::div(
                battery_info.FullChargedCapacity as f64,
                battery_info.DesignedCapacity as f64,
            ) * 100.0;

            // 查询电池名称
            let name = match query_string_info(handle_raw, tag, BatteryDeviceName) {
                Ok(n) if !n.is_empty() => n,
                _ => "未知电池".to_string(),
            };

            // 查询制造商
            let vendor = match query_string_info(handle_raw, tag, BatteryManufactureName) {
                Ok(v) if !v.is_empty() => v,
                _ => "未知".to_string(),
            };

            // 构建 Battery 结构
            let battery = Battery {
                name,
                health,
                chemistry,
                vendor,
                designed_capacity: battery_info.DesignedCapacity,
                full_charged_capacity: battery_info.FullChargedCapacity,
                cycle_count: battery_info.CycleCount,
            };

            info.batteries.push(battery);
        }
    }

    Ok(info)
}

/// 查询 BatteryTag
///
/// * `handle` - 电池设备句柄
///
/// 示例
/// ```ignore
/// use crate::core::battery::query_tag;
///
/// let tag = unsafe {
///     query_tag(handle)?
/// };
/// ```
unsafe fn query_tag(handle: RawHandle) -> Result<u32, DetectionError> {
    let mut tag = BATTERY_TAG_INVALID;
    let mut bytes_returned = 0u32;

    let result = DeviceIoControl(
        HANDLE(handle),
        IOCTL_BATTERY_QUERY_TAG,
        None,
        0,
        Some(&mut tag as *mut _ as _),
        std::mem::size_of_val(&tag) as u32,
        Some(&mut bytes_returned),
        None,
    );

    if result.is_ok() && tag != BATTERY_TAG_INVALID {
        return Ok(tag);
    }

    Err(DetectionError::BatteryError(format!(
        "query_tag 失败: {:?}",
        GetLastError()
    )))
}

/// 查询电池信息（化学成分、健康度、循环次数）
///
/// * `handle` - 电池设备句柄
/// * `tag` - 电池标签
///
/// 示例
/// ```ignore
/// use crate::core::battery::query_information;
///
/// let info = unsafe {
///     query_information(handle, tag)?
/// };
/// ```
unsafe fn query_information(
    handle: RawHandle,
    tag: u32,
) -> Result<BATTERY_INFORMATION, DetectionError> {
    let input = BATTERY_QUERY_INFORMATION {
        BatteryTag: tag,
        InformationLevel: BatteryInformation,
        AtRate: 0,
    };

    let in_bytes = std::slice::from_raw_parts(
        &input as *const _ as *const u8,
        std::mem::size_of_val(&input),
    );

    let out_bytes = utils::device::device_io_control(
        handle,
        IOCTL_BATTERY_QUERY_INFORMATION,
        in_bytes,
        std::mem::size_of::<BATTERY_INFORMATION>(),
    )?;

    if out_bytes.len() < std::mem::size_of::<BATTERY_INFORMATION>() {
        return Err(DetectionError::BatteryError("返回数据不足".to_string()));
    }

    let info: BATTERY_INFORMATION = std::ptr::read(out_bytes.as_ptr() as *const _);
    Ok(info)
}

/// 查询字符串类型的电池信息
///
/// * `handle` - 电池设备句柄
/// * `tag` - 电池标签
/// * `info_level` - 信息级别
unsafe fn query_string_info(
    handle: RawHandle,
    tag: u32,
    info_level: BATTERY_QUERY_INFORMATION_LEVEL,
) -> Result<String, DetectionError> {
    let input = BATTERY_QUERY_INFORMATION {
        BatteryTag: tag,
        InformationLevel: info_level,
        AtRate: 0,
    };

    let in_bytes = std::slice::from_raw_parts(
        &input as *const _ as *const u8,
        std::mem::size_of_val(&input),
    );

    const BUF_WCHAR: usize = 256;
    let out_bytes = utils::device::device_io_control(
        handle,
        IOCTL_BATTERY_QUERY_INFORMATION,
        in_bytes,
        BUF_WCHAR * std::mem::size_of::<u16>(),
    )?;

    Ok(utils::u16_bytes_to_string(&out_bytes))
}
