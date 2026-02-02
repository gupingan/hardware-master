use crate::detector::DetectionError;
use crate::utils::math::diagonal_inches_from_cm;
use crate::utils::string::u16_slice_to_string;
use crate::utils::wmi;
use windows::Win32::System::Wmi::IWbemClassObject;

/// 显示器信息
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// 显示器名称
    pub name: String,
    /// 制造商
    pub manufacturer: String,
    /// 屏幕尺寸
    pub size_desc: f64,
    /// 生成日期，某年第几周
    pub manufacture_date: String,
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
            manufacturer: "未知".to_string(),
            size_desc: 0.0,
            manufacture_date: "0年第0周".to_string(),
        }
    }
}

/// 检测显示器信息
pub fn detect_display() -> Result<MonitorInfo, DetectionError> {
    let mut info = MonitorInfo::default();

    unsafe {
        let config = wmi::WmiConfig {
            namespace: "ROOT\\wmi".to_string(),
        };
        let client =
            wmi::WmiClient::connect(&config).map_err(|e| DetectionError::MonitorError(e))?;

        // 查询 WmiMonitorBasicDisplayParams 获取显示器尺寸
        let mut display_params_map = std::collections::HashMap::new();
        let mut display_params_enumerator = client
            .query("SELECT * FROM WmiMonitorBasicDisplayParams")
            .map_err(|e| DetectionError::MonitorError(e))?;

        while let Some(obj) = display_params_enumerator.next() {
            if let Some(instance_name) = get_instance_name(&obj) {
                let (w_cm, h_cm) = get_display_params(&obj);
                display_params_map.insert(instance_name, (w_cm, h_cm));
            }
        }

        // 查询 WmiMonitorID 获取显示器基本信息
        let mut monitor_id_enumerator = client
            .query("SELECT * FROM WmiMonitorID")
            .map_err(|e| DetectionError::MonitorError(e))?;

        while let Some(obj) = monitor_id_enumerator.next() {
            if let Some(monitor_info) = parse_monitor_object(&obj, &display_params_map) {
                info = monitor_info;
                break; // 只获取第一个显示器
            }
        }
    }

    Ok(info)
}

/// 获取实例名称
unsafe fn get_instance_name(obj: &IWbemClassObject) -> Option<String> {
    if let Ok(var) = wmi::get_property(obj, "InstanceName") {
        wmi::variant_to_string(&var)
    } else {
        None
    }
}

/// 获取显示器尺寸参数
unsafe fn get_display_params(obj: &IWbemClassObject) -> (u8, u8) {
    let w_cm = if let Ok(var) = wmi::get_property(obj, "MaxHorizontalImageSize") {
        wmi::variant_to_u8(&var).unwrap_or(0)
    } else {
        0
    };

    let h_cm = if let Ok(var) = wmi::get_property(obj, "MaxVerticalImageSize") {
        wmi::variant_to_u8(&var).unwrap_or(0)
    } else {
        0
    };

    (w_cm, h_cm)
}

/// 解析显示器对象
unsafe fn parse_monitor_object(
    obj: &IWbemClassObject,
    display_params_map: &std::collections::HashMap<String, (u8, u8)>,
) -> Option<MonitorInfo> {
    let mut info = MonitorInfo::default();

    // 获取制造商名称
    let manufacturer = if let Ok(var) = wmi::get_property(obj, "ManufacturerName") {
        wmi::variant_to_u16_slice(&var)
            .map(|v| u16_slice_to_string(&v))
            .unwrap_or_else(|| "未知厂商".to_string())
    } else {
        "未知厂商".to_string()
    };
    info.manufacturer = manufacturer.clone();

    // 获取产品代码
    let product_code = if let Ok(var) = wmi::get_property(obj, "ProductCodeID") {
        wmi::variant_to_u16_slice(&var)
            .map(|v| u16_slice_to_string(&v))
            .unwrap_or_else(|| "未知".to_string())
    } else {
        "未知".to_string()
    };

    // 获取实例名称用于查找尺寸信息
    let instance_name = get_instance_name(obj)?;

    // 查找对应的尺寸信息
    if let Some((w_cm, h_cm)) = display_params_map.get(&instance_name) {
        if *w_cm == 0 || *h_cm == 0 {
            return None;
        }

        let w = *w_cm as f64;
        let h = *h_cm as f64;
        let diag = diagonal_inches_from_cm(w, h);
        info.size_desc = diag;
    }

    // 获取制造日期
    let week = if let Ok(var) = wmi::get_property(obj, "WeekOfManufacture") {
        wmi::variant_to_u8(&var).unwrap_or(0)
    } else {
        0
    };

    let year = if let Ok(var) = wmi::get_property(obj, "YearOfManufacture") {
        wmi::variant_to_u16(&var).unwrap_or(0)
    } else {
        0
    };

    let manufacture_date = if week != 0 {
        format!("{}年第{}周", year, week)
    } else {
        format!("{}年", year)
    };
    info.manufacture_date = manufacture_date.to_string();

    // 生成名称
    info.name = format!(
        "{} {} ({:.1}英寸, {}产)",
        manufacturer, product_code, info.size_desc, manufacture_date
    );

    Some(info)
}
