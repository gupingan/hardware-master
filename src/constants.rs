//! 常量定义模块
//!
//! 存放项目中使用的各种常量，避免 Magic Numbers

/// 字节转换常量
pub const BYTES_PER_KB: f64 = 1024.0;
pub const BYTES_PER_MB: f64 = BYTES_PER_KB * 1024.0;
pub const BYTES_PER_GB: f64 = BYTES_PER_MB * 1024.0;

/// 窗口尺寸常量
pub const WINDOW_WIDTH: f32 = 560.0;
pub const WINDOW_HEIGHT: f32 = 360.0;
pub const WINDOW_MIN_WIDTH: f32 = 400.0;
pub const WINDOW_MIN_HEIGHT: f32 = 300.0;

/// 显存大小阈值 (MB)
pub const VRAM_THRESHOLD_MB: f64 = 1000.0;

/// 内存大小阈值 (MB)
pub const MEMORY_THRESHOLD_MB: f64 = 1024.0;

/// 进度相关常量
pub const PROGRESS_START: f32 = 0.0;
pub const PROGRESS_END: f32 = 1.0;

/// 文本格式化常量
pub const TEXT_FORMAT_PRECISION: usize = 0;

/// 主题常量
pub const THEME_SYSTEM: &str = "系统";
pub const THEME_LIGHT: &str = "亮色";
pub const THEME_DARK: &str = "暗色";
