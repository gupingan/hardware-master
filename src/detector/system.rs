use crate::detector::DetectionError;
use crate::utils::wmi;

/// 电脑类型
#[derive(Debug, Clone)]
pub enum ComputerType {
    /// 笔记本
    Laptop,
    /// 台式机
    Desktop,
    /// 未知
    Unknown,
}

/// 系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// 电脑型号
    pub computer_model: String,
    /// 操作系统名称
    pub os_name: String,
    /// 系统制造商
    pub system_manufacturer: String,
    /// 电脑类型
    pub computer_type: ComputerType,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            computer_model: "未知".to_string(),
            os_name: "未知".to_string(),
            system_manufacturer: "未知".to_string(),
            computer_type: ComputerType::Unknown,
        }
    }
}

/// 检测系统信息
pub fn detect_system() -> Result<SystemInfo, DetectionError> {
    let mut info = SystemInfo::default();

    unsafe {
        let config = wmi::WmiConfig::default();
        let client = wmi::WmiClient::connect(&config)
            .map_err(|e| DetectionError::SystemError(e))?;

        // 查询操作系统信息
        let mut os_enumerator = client
            .query("SELECT * FROM Win32_OperatingSystem")
            .map_err(|e| DetectionError::SystemError(e))?;

        if let Some(obj) = os_enumerator.next() {
            if let Ok(var) = wmi::get_property(&obj, "Caption") {
                info.os_name = wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
        } else {
            return Err(DetectionError::SystemError("未找到操作系统信息".to_string()));
        }

        // 查询计算机系统信息
        let mut cs_enumerator = client
            .query("SELECT * FROM Win32_ComputerSystem")
            .map_err(|e| DetectionError::SystemError(e))?;

        if let Some(obj) = cs_enumerator.next() {
            // 获取制造商
            if let Ok(var) = wmi::get_property(&obj, "Manufacturer") {
                info.system_manufacturer =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }

            // 获取型号
            if let Ok(var) = wmi::get_property(&obj, "Model") {
                info.computer_model =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }

            // 获取系统类型
            let pc_system_type = if let Ok(var) = wmi::get_property(&obj, "PCSystemType") {
                wmi::variant_to_u16(&var)
            } else {
                None
            };

            // 根据系统类型判断电脑类型
            info.computer_type = if let Some(pc_type) = pc_system_type {
                match pc_type {
                    1 | 2 => ComputerType::Laptop,  // Portable / Mobile
                    3 | 4 => ComputerType::Desktop, // Desktop
                    _ => ComputerType::Unknown,
                }
            } else {
                // 如果没有系统类型，尝试从型号判断
                let model_lower = info.computer_model.to_lowercase();
                if model_lower.contains("notebook")
                    || model_lower.contains("laptop")
                    || model_lower.contains("book")
                    || model_lower.contains("mobile")
                {
                    ComputerType::Laptop
                } else if model_lower.contains("desktop") || model_lower.contains("tower") {
                    ComputerType::Desktop
                } else {
                    ComputerType::Unknown
                }
            };
        } else {
            return Err(DetectionError::SystemError("未找到计算机系统信息".to_string()));
        }
    }

    Ok(info)
}
