use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor};
use std::sync::LazyLock;

/// 描述查询结果的结构体
#[derive(Debug, Clone)]
pub struct DeviceDescription {
    pub vendor_name: String,
    pub device_name: Option<String>,
    pub subsystem_name: Option<String>,
}

// 内部存储结构
#[derive(Debug, Clone)]
struct DeviceEntry {
    name: String,
    // Key: 对于 PCI 是 "sub_vendor sub_device"，对于 USB 是 Interface ID
    subsystems: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct VendorEntry {
    name: String,
    devices: HashMap<String, DeviceEntry>,
}

/// 设备 ID 数据库
pub struct DeviceIdDatabase {
    pci_db: HashMap<String, VendorEntry>,
    usb_db: HashMap<String, VendorEntry>,
}

impl DeviceIdDatabase {
    /// 创建并加载数据库
    pub fn new(pci_content: &[u8], usb_content: &[u8]) -> std::io::Result<Self> {
        let pci_db = Self::load_ids_content(pci_content);
        let usb_db = Self::load_ids_content(usb_content);
        Ok(Self { pci_db, usb_db })
    }

    /// 解析 .ids 字符串内容
    fn load_ids_content(content: &[u8]) -> HashMap<String, VendorEntry> {
        let mut map = HashMap::new();

        let cursor = Cursor::new(content);
        let reader = BufReader::new(cursor);

        let mut current_vendor_id: Option<String> = None;
        let mut current_device_id: Option<String> = None;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let line_str = line.trim();
            if line_str.is_empty() || line_str.starts_with('#') {
                continue;
            }

            let level = line.len() - line.trim_start_matches('\t').len();

            match level {
                0 => {
                    if let Some((vid, vname)) = line_str.split_once("  ") {
                        current_vendor_id = Some(vid.trim().to_string());
                        current_device_id = None;
                        map.entry(vid.trim().to_string())
                            .or_insert_with(|| VendorEntry {
                                name: vname.trim().to_string(),
                                devices: HashMap::new(),
                            });
                    }
                }
                1 => {
                    if let (Some(ref vid), Some((did, dname))) =
                        (&current_vendor_id, line_str.split_once("  "))
                    {
                        current_device_id = Some(did.trim().to_string());
                        if let Some(vendor) = map.get_mut(vid) {
                            vendor
                                .devices
                                .entry(did.trim().to_string())
                                .or_insert_with(|| DeviceEntry {
                                    name: dname.trim().to_string(),
                                    subsystems: HashMap::new(),
                                });
                        }
                    }
                }
                2 => {
                    if let (Some(ref vid), Some(ref did)) = (&current_vendor_id, &current_device_id)
                    {
                        if let Some(vendor) = map.get_mut(vid) {
                            if let Some(device) = vendor.devices.get_mut(did) {
                                if let Some((id_part, sub_name)) = line_str.split_once("  ") {
                                    device.subsystems.insert(
                                        id_part.trim().to_string(),
                                        sub_name.trim().to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        map
    }

    /// 核心查询函数
    ///
    /// * `bus_type`: "PCI" 或 "USB"
    /// * `vendor_id`: 厂商 ID (例如 "10EC")
    /// * `device_id`: 设备 ID (可选，例如 "8168")
    /// * `subsystem_vendor_id`: 子系统厂商 ID (可选，仅 PCI 有效)
    /// * `subsystem_device_id`: 子系统设备 ID (可选，仅 PCI 有效)
    pub fn lookup(
        &self,
        bus_type: &str,
        vendor_id: &str,
        device_id: Option<&str>,
        subsystem_vendor_id: Option<&str>,
        subsystem_device_id: Option<&str>,
    ) -> Option<DeviceDescription> {
        let db = match bus_type.to_uppercase().as_str() {
            "PCI" => &self.pci_db,
            "USB" => &self.usb_db,
            _ => return None,
        };

        // 格式化 ID：去除 0x 前缀，不足4位补0
        let fmt_vid = Self::format_id(vendor_id);

        let vendor_entry = db.get(&fmt_vid)?;
        let mut device_name_opt = None;
        let mut subsystem_name_opt = None;

        // 查询设备
        if let Some(did) = device_id {
            let fmt_did = Self::format_id(did);
            if let Some(device_entry) = vendor_entry.devices.get(&fmt_did) {
                device_name_opt = Some(device_entry.name.clone());

                // 查询子系统
                if bus_type.to_uppercase() == "PCI" {
                    if let (Some(svid), Some(sdid)) = (subsystem_vendor_id, subsystem_device_id) {
                        let fmt_svid = Self::format_id(svid);
                        let fmt_sdid = Self::format_id(sdid);
                        let sub_key = format!("{} {}", fmt_svid, fmt_sdid);
                        if let Some(sub_name) = device_entry.subsystems.get(&sub_key) {
                            subsystem_name_opt = Some(sub_name.to_string());
                        }
                    }
                } else if bus_type.to_uppercase() == "USB" {
                    // USB 通常只有一个 Interface ID，复用 subsystem_vendor_id 参数或者忽略 subsystem
                    // subsystem_vendor_id 当作 interface ID 传入
                    if let Some(intf_id) = subsystem_vendor_id {
                        let fmt_intfid = Self::format_id(intf_id);
                        if let Some(intf_name) = device_entry.subsystems.get(&fmt_intfid) {
                            subsystem_name_opt = Some(intf_name.to_string());
                        }
                    }
                }
            }
        }

        Some(DeviceDescription {
            vendor_name: vendor_entry.name.to_string(),
            device_name: device_name_opt,
            subsystem_name: subsystem_name_opt,
        })
    }

    /// 格式化 ID (去除 0x，转小写，补全4位)
    fn format_id(id: &str) -> String {
        let id = id.trim().trim_start_matches("0x").trim_start_matches("0X");
        format!("{:0>4}", id.to_lowercase())
    }
}

impl Default for DeviceIdDatabase {
    fn default() -> Self {
        Self {
            pci_db: HashMap::new(),
            usb_db: HashMap::new(),
        }
    }
}

const PCI_IDS_BYTES: &[u8] = include_bytes!("pci.ids");
const USB_IDS_BYTES: &[u8] = include_bytes!("usb.ids");

pub static DB: LazyLock<DeviceIdDatabase> =
    LazyLock::new(|| DeviceIdDatabase::new(PCI_IDS_BYTES, USB_IDS_BYTES).unwrap_or_default());
