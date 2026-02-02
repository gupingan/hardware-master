use crate::detector::DetectionError;
use crate::format_mb_size;
use crate::utils::wmi;
use crate::constants::BYTES_PER_MB;
use std::collections::HashMap;
use windows::Win32::System::Wmi::IWbemClassObject;

/// 内存信息
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// 内存条信息名称
    pub name: String,
    /// 总内存大小 (MB)
    pub total_memory: f64,
    /// 内存插槽信息
    pub slots: Vec<MemorySlot>,
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            name: "未获取到内存条信息".to_string(),
            total_memory: 0.0,
            slots: Vec::new(),
        }
    }
}

/// 内存插槽信息
#[derive(Debug, Clone)]
pub struct MemorySlot {
    /// 内存条名称
    pub name: String,
    /// 容量 (MB)
    pub capacity: f64,
    /// 制造商
    pub manufacturer: String,
    /// 类型 (DDR3, DDR4, DDR5 等)
    pub memory_type: String,
    /// 频率 (MHz)
    pub frequency: u64,
}

impl Default for MemorySlot {
    fn default() -> Self {
        Self {
            name: "未知".to_string(),
            capacity: 0.0,
            manufacturer: "未知".to_string(),
            memory_type: "未知".to_string(),
            frequency: 0,
        }
    }
}

/// 检测内存信息
pub fn detect_memory() -> Result<MemoryInfo, DetectionError> {
    let mut info = MemoryInfo::default();

    unsafe {
        let config = wmi::WmiConfig::default();
        let client = wmi::WmiClient::connect(&config)
            .map_err(|e| DetectionError::MemoryError(e))?;

        let mut enumerator = client
            .query("SELECT * FROM Win32_PhysicalMemory")
            .map_err(|e| DetectionError::MemoryError(e))?;

        while let Some(obj) = enumerator.next() {
            if let Some(slot) = parse_memory_object(&obj) {
                info.total_memory += slot.capacity;
                info.slots.push(slot);
            }
        }
    }

    info.name = generate_total_name(&info.slots);

    Ok(info)
}

/// 解析 WMI 内存对象
unsafe fn parse_memory_object(obj: &IWbemClassObject) -> Option<MemorySlot> {
    let mut slot = MemorySlot::default();

    // 获取 Capacity 属性
    if let Ok(var) = wmi::get_property(obj, "Capacity") {
        if let Some(capacity) = wmi::variant_to_u64(&var) {
            slot.capacity = capacity as f64 / BYTES_PER_MB; // 转换为 MB
        }
    }

    // 获取 Manufacturer 属性
    if let Ok(var) = wmi::get_property(obj, "Manufacturer") {
        if let Some(manufacturer) = wmi::variant_to_string(&var) {
            slot.manufacturer = if manufacturer.trim().is_empty() {
                "未知".to_string()
            } else {
                manufacturer
            };
        }
    }

    // 获取 SMBIOSMemoryType 属性
    if let Ok(var) = wmi::get_property(obj, "SMBIOSMemoryType") {
        if let Some(mem_type) = wmi::variant_to_u32(&var) {
            slot.memory_type = parse_memory_type(mem_type);
        }
    }

    // 获取 Speed 和 ConfiguredClockSpeed 属性
    let speed = if let Ok(var) = wmi::get_property(obj, "ConfiguredClockSpeed") {
        wmi::variant_to_u32(&var)
    } else {
        None
    };

    slot.frequency = match speed {
        Some(s) if s > 0 => s as u64,
        _ => {
            // 如果 ConfiguredClockSpeed 不可用或为 0，尝试使用 Speed
            if let Ok(var) = wmi::get_property(obj, "Speed") {
                wmi::variant_to_u32(&var).unwrap_or(0) as u64
            } else {
                0
            }
        }
    };

    let capacity = format_mb_size!(slot.capacity);

    // 生成名称
    slot.name = format!(
        "{} {} {} {}",
        slot.manufacturer, slot.memory_type, slot.frequency, capacity
    );

    if slot.capacity > 0.0 {
        Some(slot)
    } else {
        None
    }
}

/// 解析内存类型
fn parse_memory_type(mem_type: u32) -> String {
    match mem_type {
        1 => "Other".to_string(),
        2 => "DRAM".to_string(),
        3 => "Synchronous DRAM".to_string(),
        4 => "Cache DRAM".to_string(),
        5 => "EDO".to_string(),
        6 => "EDRAM".to_string(),
        7 => "VRAM".to_string(),
        8 => "SRAM".to_string(),
        9 => "RAM".to_string(),
        10 => "ROM".to_string(),
        11 => "Flash".to_string(),
        12 => "EEPROM".to_string(),
        13 => "FEPROM".to_string(),
        14 => "EPROM".to_string(),
        15 => "CDRAM".to_string(),
        16 => "3DRAM".to_string(),
        17 => "SDRAM".to_string(),
        18 => "SGRAM".to_string(),
        19 => "RDRAM".to_string(),
        20 => "DDR".to_string(),
        21 => "DDR2".to_string(),
        22 => "DDR2 FB-DIMM".to_string(),
        24 => "DDR3".to_string(),
        25 => "FBD2".to_string(),
        26 => "DDR4".to_string(),
        27 => "LPDDR".to_string(),
        28 => "LPDDR2".to_string(),
        29 => "LPDDR3".to_string(),
        30 => "LPDDR4".to_string(),
        31 => "DDR5".to_string(),
        32 => "LPDDR5".to_string(),
        _ => "未知".to_string(),
    }
}

/// 根据内存插槽生成总名称
pub fn generate_total_name(slots: &[MemorySlot]) -> String {
    let mut name_counts = HashMap::new();
    let mut total_capacity = 0.0;

    // 单次遍历同时计数和求和容量
    for slot in slots {
        *name_counts.entry(&slot.name).or_insert(0) += 1;
        total_capacity += slot.capacity;
    }

    // 构建名称部分：使用迭代器收集并连接
    let name_parts: Vec<String> = name_counts
        .into_iter()
        .map(|(name, count)| format!("{} x {}", name, count))
        .collect();

    let names_str = name_parts.join(" ");

    format!("{} ({})", format_mb_size!(total_capacity), names_str)
}
