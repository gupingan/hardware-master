//! 硬件检测核心模块
//!
//! 提供各种硬件设备的检测功能

pub mod audio;
pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod monitor;
pub mod motherboard;
pub mod network;
pub mod system;

pub use audio::AudioInfo;
pub use battery::BatteryInfo;
pub use cpu::CpuInfo;
pub use disk::DiskInfo;
pub use gpu::GpuInfo;
pub use memory::MemoryInfo;
pub use monitor::MonitorInfo;
pub use motherboard::MotherboardInfo;
pub use network::NetworkInfo;
pub use system::{ComputerType, SystemInfo};

use crate::impl_detect_method;
use thiserror::Error;

/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(f32, &str) + Send>;

/// 检测任务总数
pub const TOTAL_DETECTION_TASKS: usize = 10;

/// 进度完成标记
pub const PROGRESS_COMPLETE: f32 = 1.0;

/// 防抖时间（秒）
pub const DEBOUNCE_DURATION_SECS: u64 = 1;

/// 硬件检测器
pub struct HardwareDetector {
    /// 系统信息
    pub system_info: SystemInfo,
    /// CPU 信息
    pub cpu_info: CpuInfo,
    /// 内存信息
    pub memory_info: MemoryInfo,
    /// 磁盘信息
    pub disk_info: DiskInfo,
    /// 显卡信息
    pub gpu_info: GpuInfo,
    /// 主板信息
    pub motherboard_info: MotherboardInfo,
    /// 网络信息
    pub network_info: NetworkInfo,
    /// 声卡信息
    pub audio_info: AudioInfo,
    /// 显示器信息
    pub monitor_info: MonitorInfo,
    /// 电池信息
    pub battery_info: BatteryInfo,
    /// 进度回调函数
    progress_callback: Option<ProgressCallback>,
}

impl std::fmt::Debug for HardwareDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HardwareDetector")
            .field("system_info", &self.system_info)
            .field("cpu_info", &self.cpu_info)
            .field("memory_info", &self.memory_info)
            .field("disk_info", &self.disk_info)
            .field("gpu_info", &self.gpu_info)
            .field("motherboard_info", &self.motherboard_info)
            .field("network_info", &self.network_info)
            .field("audio_info", &self.audio_info)
            .field("monitor_info", &self.monitor_info)
            .field("battery_info", &self.battery_info)
            .finish()
    }
}

impl Default for HardwareDetector {
    fn default() -> Self {
        Self {
            system_info: SystemInfo::default(),
            cpu_info: CpuInfo::default(),
            memory_info: MemoryInfo::default(),
            disk_info: DiskInfo::default(),
            gpu_info: GpuInfo::default(),
            motherboard_info: MotherboardInfo::default(),
            network_info: NetworkInfo::default(),
            audio_info: AudioInfo::default(),
            monitor_info: MonitorInfo::default(),
            battery_info: BatteryInfo::default(),
            progress_callback: None,
        }
    }
}

impl HardwareDetector {
    /// 创建新的硬件检测器
    pub fn new() -> Self {
        Self {
            system_info: SystemInfo::default(),
            cpu_info: CpuInfo::default(),
            memory_info: MemoryInfo::default(),
            disk_info: DiskInfo::default(),
            gpu_info: GpuInfo::default(),
            motherboard_info: MotherboardInfo::default(),
            network_info: NetworkInfo::default(),
            audio_info: AudioInfo::default(),
            monitor_info: MonitorInfo::default(),
            battery_info: BatteryInfo::default(),
            progress_callback: None,
        }
    }

    /// 设置进度回调函数
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    /// 更新进度
    fn update_progress(&self, progress: f32, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(progress, message);
        }
    }

    /// 检测所有硬件信息
    pub fn detect_all(&mut self) -> Result<(), DetectionError> {
        let tasks: [(&str, fn(&mut Self) -> Result<(), DetectionError>); TOTAL_DETECTION_TASKS] = [
            ("系统信息", Self::detect_system_info),
            ("CPU信息", Self::detect_cpu_info),
            ("显卡信息", Self::detect_gpu_info),
            ("内存信息", Self::detect_memory_info),
            ("磁盘信息", Self::detect_disk_info),
            ("主板信息", Self::detect_motherboard_info),
            ("网络信息", Self::detect_network_info),
            ("声卡信息", Self::detect_audio_info),
            ("显示器信息", Self::detect_monitor_info),
            ("电池信息", Self::detect_battery_info),
        ];

        let total = tasks.len();
        let mut prev_name = "开始检测";

        for (index, task) in tasks.iter().enumerate() {
            let (name, func) = task;
            let progress = index as f32 / total as f32;
            let message = format!("({}/{}) {}√ {}...", index, total, prev_name, name);
            self.update_progress(progress, &message);
            (func)(self)?;
            prev_name = name;
        }

        let final_message = format!("({}/{}) 本次检测完成√", total, total);
        self.update_progress(PROGRESS_COMPLETE, &final_message);

        Ok(())
    }

    // 使用宏生成的检测方法
    impl_detect_method!(
        detect_system_info,
        system_info,
        system,
        detect_system,
        SystemError
    );
    impl_detect_method!(detect_cpu_info, cpu_info, cpu, detect_cpu, CpuError);
    impl_detect_method!(detect_gpu_info, gpu_info, gpu, detect_gpu, GpuError);
    impl_detect_method!(
        detect_memory_info,
        memory_info,
        memory,
        detect_memory,
        MemoryError
    );
    impl_detect_method!(detect_disk_info, disk_info, disk, detect_disk, DiskError);
    impl_detect_method!(
        detect_motherboard_info,
        motherboard_info,
        motherboard,
        detect_motherboard,
        MotherboardError
    );
    impl_detect_method!(
        detect_network_info,
        network_info,
        network,
        detect_network,
        NetworkError
    );
    impl_detect_method!(
        detect_audio_info,
        audio_info,
        audio,
        detect_audio,
        AudioError
    );
    impl_detect_method!(
        detect_monitor_info,
        monitor_info,
        monitor,
        detect_display,
        MonitorError
    );
    impl_detect_method!(
        detect_battery_info,
        battery_info,
        battery,
        detect_battery,
        BatteryError
    );
}

/// 硬件检测错误类型
#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("Windows API 检测失败: {0}")]
    WindowsApiError(String),
    #[error("系统信息检测失败: {0}")]
    SystemError(String),
    #[error("CPU 信息检测失败: {0}")]
    CpuError(String),
    #[error("显卡信息检测失败: {0}")]
    GpuError(String),
    #[error("内存信息检测失败: {0}")]
    MemoryError(String),
    #[error("磁盘信息检测失败: {0}")]
    DiskError(String),
    #[error("主板信息检测失败: {0}")]
    MotherboardError(String),
    #[error("网络信息检测失败: {0}")]
    NetworkError(String),
    #[error("声卡信息检测失败: {0}")]
    AudioError(String),
    #[error("显示器信息检测失败: {0}")]
    MonitorError(String),
    #[error("电池信息检测失败: {0}")]
    BatteryError(String),
}
