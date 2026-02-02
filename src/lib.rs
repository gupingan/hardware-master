//! Hardware Master - 高性能硬件检测工具
//!
//! 这是一个类似鲁大师的硬件检测工具，使用 Rust 和 egui 开发。
//! 提供全面的硬件信息检测和友好的 GUI 界面。

pub mod constants;
pub mod detector;
pub mod iddb;
pub mod ui;
pub mod utils;

pub use detector::HardwareDetector;

/// 硬件检测工具的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
