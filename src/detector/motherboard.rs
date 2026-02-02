use crate::detector::DetectionError;
use crate::utils::wmi;
use crate::utils::wmi_date::parse_wmi_date;

/// 主板信息
#[derive(Debug, Clone)]
pub struct MotherboardInfo {
    /// 制造商
    pub manufacturer: String,
    /// 产品名称
    pub product_name: String,
    /// 序列号
    pub serial_number: String,
    /// 芯片组
    pub chipset: String,
    /// BIOS 制造商
    pub bios_vendor: String,
    /// BIOS 版本
    pub bios_version: String,
    /// BIOS 日期
    pub bios_date: String,
    /// BIOS 序列号
    pub bios_serial: String,
}

impl Default for MotherboardInfo {
    fn default() -> Self {
        Self {
            manufacturer: "未知".to_string(),
            product_name: "未知".to_string(),
            serial_number: "未知".to_string(),
            chipset: "未知".to_string(),
            bios_vendor: "未知".to_string(),
            bios_version: "未知".to_string(),
            bios_date: "未知".to_string(),
            bios_serial: "未知".to_string(),
        }
    }
}

/// 检测主板信息
pub fn detect_motherboard() -> Result<MotherboardInfo, DetectionError> {
    let mut info = MotherboardInfo::default();

    unsafe {
        let config = wmi::WmiConfig::default();
        let client =
            wmi::WmiClient::connect(&config).map_err(|e| DetectionError::MotherboardError(e))?;

        // 获取主板信息
        let mut baseboard_enumerator = client
            .query("SELECT * FROM Win32_BaseBoard")
            .map_err(|e| DetectionError::MotherboardError(e))?;

        if let Some(obj) = baseboard_enumerator.next() {
            if let Ok(var) = wmi::get_property(&obj, "Manufacturer") {
                info.manufacturer =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
            if let Ok(var) = wmi::get_property(&obj, "Product") {
                info.product_name =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
            if let Ok(var) = wmi::get_property(&obj, "SerialNumber") {
                info.serial_number =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
        }

        // 获取 BIOS 信息
        let mut bios_enumerator = client
            .query("SELECT * FROM Win32_BIOS")
            .map_err(|e| DetectionError::MotherboardError(e))?;

        if let Some(obj) = bios_enumerator.next() {
            if let Ok(var) = wmi::get_property(&obj, "Manufacturer") {
                info.bios_vendor =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
            if let Ok(var) = wmi::get_property(&obj, "SMBIOSBIOSVersion") {
                info.bios_version =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
            if let Ok(var) = wmi::get_property(&obj, "ReleaseDate") {
                let date_str = wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
                info.bios_date = parse_wmi_date(&date_str);
            }
            if let Ok(var) = wmi::get_property(&obj, "SerialNumber") {
                info.bios_serial =
                    wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string());
            }
        }

        // 获取芯片组信息（查找LPC控制器等芯片组核心设备）
        let mut pnp_enumerator = client
            .query("SELECT * FROM Win32_PnPEntity")
            .map_err(|e| DetectionError::MotherboardError(e))?;

        while let Some(obj) = pnp_enumerator.next() {
            if let Ok(var) = wmi::get_property(&obj, "Name") {
                if let Some(name) = wmi::variant_to_string(&var) {
                    // 查找LPC控制器（BIOS对应的芯片组设备）
                    if name.contains("LPC Controller")
                        || name.contains("LPC")
                        || name.contains("ISA Bridge")
                        || name.contains("SMBus Controller")
                    {
                        info.chipset = name;
                        break;
                    }
                }
            }
        }
    }

    Ok(info)
}
