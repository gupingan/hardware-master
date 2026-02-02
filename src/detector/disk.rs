use crate::detector::DetectionError;
use windows::Win32::Foundation::GetLastError;
use std::ffi::CStr;
use std::mem;
use std::os::windows::io::RawHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Ioctl::{
    PropertyStandardQuery, StorageDeviceProperty, StorageDeviceSeekPenaltyProperty,
    DEVICE_SEEK_PENALTY_DESCRIPTOR, DISK_GEOMETRY, IOCTL_DISK_GET_DRIVE_GEOMETRY,
    IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY,
};
use windows::Win32::System::IO::DeviceIoControl;

/// 整个硬盘的信息
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// 磁盘名称，如：(标准磁盘驱动器) PCIe-8 SSD 1TB
    pub model: String,
    /// 总容量 (MB)
    pub total_capacity: u64,
    /// 磁盘类型
    pub disk_type: DiskType,
}

impl Default for DiskInfo {
    fn default() -> Self {
        Self {
            model: String::from("未知"),
            total_capacity: 0,
            disk_type: DiskType::Unknown,
        }
    }
}

/// 硬盘类型
#[derive(Debug, Clone)]
pub enum DiskType {
    SSD,
    HDD,
    Unknown,
}

impl DiskType {
    /// 转换为可视化字符串，比如 "SSD" -> "固态硬盘"
    pub fn to_string(&self) -> String {
        match self {
            DiskType::SSD => String::from("固态硬盘"),
            DiskType::HDD => String::from("机械硬盘"),
            DiskType::Unknown => String::from("未知类型"),
        }
    }
}

/// 检测磁盘信息（返回主要物理硬盘的信息）
pub fn detect_disk() -> Result<DiskInfo, DetectionError> {
    unsafe {
        // 获取磁盘 0 的信息（主要的物理硬盘）
        let (model, total_capacity, disk_type) = get_disk_info(0)?;
        Ok(DiskInfo {
            model,
            total_capacity: total_capacity / (1024 * 1024),
            disk_type,
        })
    }
}

/// 获取磁盘信息
///
/// * `disk_number` - 磁盘编号 (0, 1, 2, ...)
/// ```
pub unsafe fn get_disk_info(
    disk_number: u32,
) -> Result<(String, u64, DiskType), DetectionError> {
    let disk_path = format!(r"\\.\PhysicalDrive{}", disk_number);

    let handle = CreateFileW(
        PCWSTR::from_raw(
            disk_path
                .encode_utf16()
                .chain([0])
                .collect::<Vec<u16>>()
                .as_ptr(),
        ),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        None,
    )
    .map_err(|e| DetectionError::DiskError(format!("无法打开物理磁盘 {}: {}", disk_path, e)))?;

    let _guard = scopeguard::guard(handle, |h| {
        let _ = CloseHandle(h);
    });

    let disk_type = get_disk_type(handle.0)?;
    let total_capacity = get_disk_capacity(handle.0)?;
    let model = get_disk_model(handle.0)?;

    Ok((model, total_capacity, disk_type))
}

/// 使用 IOCTL 获取磁盘类型
///
/// * `handle` - 磁盘设备句柄
/// ```
pub unsafe fn get_disk_type(
    handle: RawHandle,
) -> Result<crate::detector::disk::DiskType, DetectionError> {
    use crate::detector::disk::DiskType;

    let spq = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceSeekPenaltyProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };

    let mut seek_penalty: DEVICE_SEEK_PENALTY_DESCRIPTOR = mem::zeroed();
    let mut dw_size: u32 = 0;

    let seek_result = DeviceIoControl(
        HANDLE(handle),
        IOCTL_STORAGE_QUERY_PROPERTY,
        Some(&spq as *const _ as *const _),
        mem::size_of_val(&spq) as u32,
        Some(&mut seek_penalty as *mut _ as *mut _),
        mem::size_of_val(&seek_penalty) as u32,
        Some(&mut dw_size),
        None,
    );

    if seek_result.is_ok() && dw_size == mem::size_of_val(&seek_penalty) as u32 {
        Ok(if seek_penalty.IncursSeekPenalty {
            DiskType::HDD
        } else {
            DiskType::SSD
        })
    } else {
        Err(DetectionError::DiskError(format!(
            "获取磁盘类型失败: {:?}",
            GetLastError()
        )))
    }
}

/// 获取磁盘容量
///
/// * `handle` - 磁盘设备句柄
unsafe fn get_disk_capacity(handle: RawHandle) -> Result<u64, DetectionError> {
    let mut geometry: DISK_GEOMETRY = mem::zeroed();
    let mut bytes_returned = 0u32;

    let result = DeviceIoControl(
        HANDLE(handle),
        IOCTL_DISK_GET_DRIVE_GEOMETRY,
        None,
        0,
        Some(&mut geometry as *mut _ as _),
        mem::size_of_val(&geometry) as u32,
        Some(&mut bytes_returned),
        None,
    );

    if result.is_err() {
        return Err(DetectionError::DiskError(format!(
            "获取磁盘几何信息失败: {:?}",
            GetLastError()
        )));
    }

    let total_sectors = geometry.Cylinders as u64
        * geometry.TracksPerCylinder as u64
        * geometry.SectorsPerTrack as u64;
    let bytes_per_sector = geometry.BytesPerSector;
    Ok(total_sectors * bytes_per_sector as u64)
}

/// 获取磁盘型号
///
/// * `handle` - 磁盘设备句柄
unsafe fn get_disk_model(handle: RawHandle) -> Result<String, DetectionError> {
    let spq = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };

    let mut buffer: Vec<u8> = vec![0; 512];
    let mut bytes_needed = 0u32;

    let result = DeviceIoControl(
        HANDLE(handle),
        IOCTL_STORAGE_QUERY_PROPERTY,
        Some(&spq as *const _ as *const _),
        mem::size_of_val(&spq) as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        buffer.len() as u32,
        Some(&mut bytes_needed),
        None,
    );

    if result.is_err() {
        return Err(DetectionError::DiskError(
            "DeviceIoControl 读取失败".to_string(),
        ));
    }

    let descriptor = &*(buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR);

    if descriptor.ProductIdOffset > 0 {
        let offset = descriptor.ProductIdOffset as usize;

        let name_ptr = buffer.as_ptr().add(offset) as *const i8;

        if let Ok(c_str) = CStr::from_ptr(name_ptr).to_str() {
            return Ok(c_str.trim().to_string());
        }
    }

    Ok("未知磁盘".to_string())
}
