use crate::detector::DetectionError;
use crate::utils::wmi;

/// 网络信息
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub adapters: Vec<String>,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self { adapters: vec![] }
    }
}

/// 网络适配器
#[derive(Debug, Clone)]
pub struct NetworkAdapter {
    pub name: String,
}

impl Default for NetworkAdapter {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
        }
    }
}

/// 检测网络信息
pub fn detect_network() -> Result<NetworkInfo, DetectionError> {
    let mut info = NetworkInfo::default();

    unsafe {
        let config = wmi::WmiConfig::default();
        let client =
            wmi::WmiClient::connect(&config).map_err(|e| DetectionError::NetworkError(e))?;

        let mut enumerator = client
            .query("SELECT * FROM Win32_NetworkAdapter")
            .map_err(|e| DetectionError::NetworkError(e))?;

        while let Some(obj) = enumerator.next() {
            // 获取物理适配器标志
            let physical_adapter = if let Ok(var) = wmi::get_property(&obj, "PhysicalAdapter") {
                wmi::variant_to_bool(&var).unwrap_or(false)
            } else {
                false
            };

            // 获取 PnP 设备 ID
            let pnp_device_id = if let Ok(var) = wmi::get_property(&obj, "PNPDeviceID") {
                wmi::variant_to_string(&var)
            } else {
                None
            };

            // 获取适配器类型 ID
            let adapter_type_id = if let Ok(var) = wmi::get_property(&obj, "AdapterTypeID") {
                wmi::variant_to_u16(&var)
            } else {
                None
            };

            // 获取适配器名称
            let name = if let Ok(var) = wmi::get_property(&obj, "Name") {
                wmi::variant_to_string(&var).unwrap_or_else(|| "未知".to_string())
            } else {
                "未知".to_string()
            };

            // 只处理物理 PCI 适配器
            let is_physical = physical_adapter
                && pnp_device_id.as_deref().map(|p| p.starts_with("PCI")) == Some(true);

            if !is_physical {
                continue;
            }

            match adapter_type_id {
                Some(0) | Some(9) => info.adapters.push(name),
                _ => {}
            }
        }
    }

    Ok(info)
}
