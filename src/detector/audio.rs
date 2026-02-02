use crate::detector::DetectionError;
use crate::utils;
use std::mem;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
    HDEVINFO, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME, SPDRP_HARDWAREID, SPDRP_MFG, SP_DEVINFO_DATA,
};
use windows::Win32::Foundation::{GetLastError, ERROR_NO_MORE_ITEMS};
use windows::Win32::Media::KernelStreaming::KSCATEGORY_AUDIO;

/// 声卡信息
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// 音频设备列表
    pub devices: Vec<AudioDevice>,
}

impl Default for AudioInfo {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
}

/// 音频设备信息
///
/// 表示单个音频设备的基本信息。
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// 设备名称
    pub name: String,
    /// 制造商
    pub manufacturer: String,
    /// 设备 ID（硬件 ID）
    pub device_id: String,
}

impl Default for AudioDevice {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
            manufacturer: "未知".to_string(),
            device_id: "未知".to_string(),
        }
    }
}

/// 检测声卡信息
pub fn detect_audio() -> Result<AudioInfo, DetectionError> {
    unsafe {
        let mut devices: Vec<AudioDevice> = Vec::new();
        if let Ok(device_info_set) = utils::device::get_device_info_set(
            &KSCATEGORY_AUDIO,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        ) {
            let _guard = scopeguard::guard(device_info_set, |h| {
                let _ = SetupDiDestroyDeviceInfoList(h);
            });

            let mut device_index = 0;

            loop {
                match enumerate_device(device_info_set, device_index) {
                    Ok(Some(device)) => {
                        devices.push(device);
                        device_index += 1;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        log::warn!("枚举设备索引 {} 时出错: {:?}", device_index, e);
                        device_index += 1;
                        continue;
                    }
                }
            }
        }
        Ok(AudioInfo { devices })
    }
}

/// 枚举指定索引的设备
///
/// 获取指定索引的设备信息，并返回设备元组。
/// 如果设备不是 HDAUDIO 设备，返回 `Ok(None)`。
///
/// * `device_info_set` - 设备信息集句柄
/// * `device_index` - 设备索引
unsafe fn enumerate_device(
    device_info_set: HDEVINFO,
    device_index: u32,
) -> Result<Option<AudioDevice>, DetectionError> {
    let mut device_info_data = SP_DEVINFO_DATA {
        cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };

    SetupDiEnumDeviceInfo(device_info_set, device_index, &mut device_info_data).map_err(|e| {
        let err = GetLastError();
        if err == ERROR_NO_MORE_ITEMS {
            return DetectionError::AudioError("没有更多设备".to_string());
        }
        DetectionError::AudioError(format!("SetupDiEnumDeviceInfo 失败: {:?}", e))
    })?;

    let device_instance_id =
        utils::device::get_device_instance_id(device_info_set, &device_info_data)?;

    // 过滤非 HDAUDIO 设备
    let is_hd_audio_device = device_instance_id.starts_with("HDAUDIO\\");
    if !is_hd_audio_device {
        return Ok(None);
    }

    // 优先使用设备描述，其次使用友好名称
    let name = utils::device::get_device_property(
        device_info_set,
        &device_info_data,
        SPDRP_DEVICEDESC,
    )
    .or_else(|| {
        utils::device::get_device_property(
            device_info_set,
            &device_info_data,
            SPDRP_FRIENDLYNAME,
        )
    })
    .unwrap_or_else(|| "未知设备".to_string());

    let manufacturer = utils::device::get_device_property(
        device_info_set,
        &device_info_data,
        SPDRP_MFG
    )
    .unwrap_or_else(|| "未知制造商".to_string());

    let device_id = utils::device::get_device_property(
        device_info_set,
        &device_info_data,
        SPDRP_HARDWAREID,
    )
    .unwrap_or_default();

    Ok(Some(AudioDevice {
        name,
        manufacturer,
        device_id,
    }))
}
