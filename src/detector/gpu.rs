use crate::detector::DetectionError;
use crate::iddb;
use crate::utils;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    DXGI_ERROR_NOT_FOUND,
};

/// 显卡信息
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// 显卡列表
    pub gpus: Vec<Gpu>,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self { gpus: Vec::new() }
    }
}

/// 单个显卡信息
#[derive(Debug, Clone)]
pub struct Gpu {
    /// 显卡描述
    pub description: String,
    /// 制造商
    pub manufacturer: String,
    /// 芯片商
    pub chip_vendor: String,
    /// 显卡类型
    pub gpu_type: GpuType,
    /// 显存大小 (B)
    pub vram_size: f64,
    /// 显卡 ID
    pub device_id: String,
    /// 厂商 ID
    pub vendor_id: String,
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            description: "未知".to_string(),
            manufacturer: "未知".to_string(),
            chip_vendor: "未知".to_string(),
            gpu_type: GpuType::Other,
            vram_size: 0.0,
            device_id: "未知".to_string(),
            vendor_id: "未知".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuType {
    /// Other or Unknown.
    Other,
    /// Integrated GPU with shared CPU/GPU memory.
    IntegratedGpu,
    /// Discrete GPU with separate CPU/GPU memory.
    DiscreteGpu,
    /// Virtual / Hosted.
    VirtualGpu,
    /// Cpu / Software Rendering.
    Cpu,
}

impl ToString for GpuType {
    fn to_string(&self) -> String {
        match self {
            GpuType::IntegratedGpu => "集成显卡".to_string(),
            GpuType::DiscreteGpu => "独立显卡".to_string(),
            GpuType::VirtualGpu => "虚拟显卡".to_string(),
            _ => "其它显卡".to_string(),
        }
    }
}

/// 检测显卡信息
pub fn detect_gpu() -> Result<GpuInfo, DetectionError> {
    let mut info = GpuInfo::default();

    unsafe {
        if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
            let mut adapter_index = 0;
            loop {
                let adapter: IDXGIAdapter1 = match factory.EnumAdapters1(adapter_index) {
                    Ok(a) => a,
                    Err(ref e) if e.code() == DXGI_ERROR_NOT_FOUND => {
                        break;
                    }
                    Err(e) => {
                        log::warn!("枚举 IDXGI 适配器时发生错误: {:?}", e);
                        break;
                    }
                };

                let desc = match adapter.GetDesc1() {
                    Ok(d) => d,
                    Err(e) => {
                        log::warn!("获取适配器描述时发生错误: {:?}", e);
                        continue;
                    }
                };

                let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;
                if is_software {
                    adapter_index += 1;
                    continue;
                }

                let description = utils::u16_slice_to_string(&desc.Description);
                let vendor_id = format!("{:04x}", desc.VendorId);
                let device_id = format!("{:04x}", desc.DeviceId);
                let subsys_vendor_id = format!("{:04x}", desc.SubSysId)[4..].to_string();
                let manufacturer = match subsys_vendor_id.as_str() {
                    "" => "未知".to_string(),
                    _ => get_vendor_by_id("PCI", &subsys_vendor_id),
                };
                let chip_vendor = get_vendor_by_id("PCI", &vendor_id);
                let gpu_type = get_gpu_type(&description, &vendor_id);
                let vram_size = desc.DedicatedVideoMemory as f64;

                let gpu = Gpu {
                    description,
                    manufacturer,
                    chip_vendor,
                    gpu_type,
                    vram_size,
                    device_id,
                    vendor_id,
                };

                info.gpus.push(gpu);

                adapter_index += 1;
            }
        }
    }

    Ok(info)
}

/// 获取厂商名称
fn get_vendor_by_id(bus_type: &str, vendor_id: &str) -> String {
    match iddb::DB.lookup(bus_type, vendor_id, None, None, None) {
        Some(info) => info.vendor_name,
        None => {
            log::warn!(
                "未查询到 bus_type={:?}; vendor_id={:?} 的信息",
                bus_type,
                vendor_id
            );
            return "未知".to_string();
        }
    }
}

/// 获取显卡类型
fn get_gpu_type(description: &str, vendor_id: &str) -> GpuType {
    let desc_lower = description.to_lowercase();
    let vid_lower = vendor_id.to_lowercase();

    let is_cpu_renderer = desc_lower.contains("microsoft basic display adapter")
        || desc_lower.contains("microsoft remote display adapter")
        || desc_lower.contains("basic render driver")
        || desc_lower.contains("llvmpipe")
        || desc_lower.contains("swiftshader")
        || desc_lower.contains("mesa offscreen");

    let is_virtual = desc_lower.contains("virtio")
        || desc_lower.contains("qxl")
        || desc_lower.contains("vmware")
        || desc_lower.contains("virtual")
        || desc_lower.contains("virtualbox")
        || desc_lower.contains("vga")
        || vid_lower == "1af4" // virtio
        || vid_lower == "80ee"; // virtualbox

    if is_cpu_renderer {
        return GpuType::Cpu;
    }
    if is_virtual {
        return GpuType::VirtualGpu;
    }

    let is_intel_gpu = desc_lower.contains("intel")
        || desc_lower.contains("iris")
        || desc_lower.contains("uhd")
        || desc_lower.contains("hd graphics");
    if is_intel_gpu {
        return GpuType::IntegratedGpu;
    }

    let is_amd_apu = desc_lower.contains("amd renoir")
        || desc_lower.contains("radeon hd 4200")
        || desc_lower.contains("radeon hd 4250")
        || desc_lower.contains("radeon hd 4270")
        || desc_lower.contains("radeon hd 4225")
        || desc_lower.contains("radeon hd 3100")
        || desc_lower.contains("radeon hd 3200")
        || desc_lower.contains("radeon hd 3000")
        || desc_lower.contains("radeon hd 3300")
        || desc_lower.contains("radeon r4 graphics")
        || desc_lower.contains("radeon r5 graphics")
        || desc_lower.contains("radeon r6 graphics")
        || desc_lower.contains("radeon r7 graphics");

    if is_amd_apu {
        return GpuType::IntegratedGpu;
    }

    let is_nvidia = desc_lower.contains("nvidia") || vid_lower == "10de";

    if is_nvidia {
        return GpuType::DiscreteGpu;
    }

    GpuType::Other
}
