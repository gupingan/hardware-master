use crate::detector::DetectionError;
use crate::utils;
use std::mem;
use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
use windows::Win32::System::SystemInformation::{
    GetLogicalProcessorInformation, GetNativeSystemInfo, RelationProcessorCore,
    PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM, PROCESSOR_ARCHITECTURE_ARM64,
    PROCESSOR_ARCHITECTURE_IA64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
    SYSTEM_LOGICAL_PROCESSOR_INFORMATION,
};

/// CPU 信息
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// 处理器名称
    pub name: String,
    /// 处理器架构
    pub architecture: String,
    /// 核心数
    pub cores: String,
    /// 制造商
    pub vendor: String,
    /// CPU ID
    pub cpu_id: String,
    /// 最大频率 (MHz)
    pub max_frequency: u16,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
            architecture: "未知".to_string(),
            cores: "未知".to_string(),
            max_frequency: 0,
            vendor: "未知".to_string(),
            cpu_id: "未知".to_string(),
        }
    }
}

/// CPU 注册表路径
const CPU_KEY_PATH: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

/// 检测 CPU 信息
pub fn detect_cpu() -> Result<CpuInfo, DetectionError> {
    unsafe {
        let name = get_processor_name().unwrap_or_else(|| "未知".to_string());
        let architecture = get_processor_architecture();
        let cores = get_cores_info();
        let vendor = get_vendor_identifier().unwrap_or_else(|| "未知".to_string());
        let cpu_id = get_cpu_identifier().unwrap_or_else(|| "未知".to_string());
        let max_frequency = get_max_frequency().unwrap_or(0);

        return Ok(CpuInfo {
            name,
            architecture,
            cores,
            vendor,
            cpu_id,
            max_frequency,
        });
    }
}

/// 获取处理器名称
pub unsafe fn get_processor_name() -> Option<String> {
    utils::registry::read_registry_string(HKEY_LOCAL_MACHINE, CPU_KEY_PATH, "ProcessorNameString")
}

/// 获取供应商标识符
pub unsafe fn get_vendor_identifier() -> Option<String> {
    utils::registry::read_registry_string(HKEY_LOCAL_MACHINE, CPU_KEY_PATH, "VendorIdentifier")
        .or_else(|| {
            utils::registry::read_registry_string(HKEY_LOCAL_MACHINE, CPU_KEY_PATH, "Identifier")
        })
}

/// 获取 CPU 标识符
pub unsafe fn get_cpu_identifier() -> Option<String> {
    utils::registry::read_registry_string(HKEY_LOCAL_MACHINE, CPU_KEY_PATH, "Identifier")
}

/// 获取最大频率
pub unsafe fn get_max_frequency() -> Option<u16> {
    utils::registry::read_registry_dword(HKEY_LOCAL_MACHINE, CPU_KEY_PATH, "~MHz")
        .map(|mhz| mhz as u16)
}

/// 获取处理器架构
pub unsafe fn get_processor_architecture() -> String {
    let mut sys_info: SYSTEM_INFO = mem::zeroed();
    GetNativeSystemInfo(&mut sys_info);

    match sys_info.Anonymous.Anonymous.wProcessorArchitecture {
        PROCESSOR_ARCHITECTURE_INTEL => "x86".to_string(),
        PROCESSOR_ARCHITECTURE_AMD64 => "x64".to_string(),
        PROCESSOR_ARCHITECTURE_ARM => "ARM".to_string(),
        PROCESSOR_ARCHITECTURE_ARM64 => "ARM64".to_string(),
        PROCESSOR_ARCHITECTURE_IA64 => "Itanium".to_string(),
        _ => "未知".to_string(),
    }
}

/// 获取物理核心和逻辑核心数。
pub unsafe fn get_cores_info() -> String {
    let mut buffer_size = 0u32;
    let _ = GetLogicalProcessorInformation(None, &mut buffer_size);

    if buffer_size == 0 {
        return "未知".to_string();
    }

    let num_entries = buffer_size as usize / mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>();
    let mut buffer: Vec<SYSTEM_LOGICAL_PROCESSOR_INFORMATION> = vec![mem::zeroed(); num_entries];

    if GetLogicalProcessorInformation(Some(buffer.as_mut_ptr()), &mut buffer_size).is_err() {
        return "未知".to_string();
    }

    let mut physical_cores = 0;
    let mut logical_cores = 0;

    for entry in &buffer {
        if entry.Relationship == RelationProcessorCore {
            physical_cores += 1;
            logical_cores += entry.ProcessorMask.count_ones();
        }
    }

    format!("物理核：{} / 逻辑核：{}", physical_cores, logical_cores)
}
