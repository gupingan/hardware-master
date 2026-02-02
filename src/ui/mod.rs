//! GUI 模块
//!
//! 使用 egui 提供用户界面

pub mod app;
pub mod font;
pub mod theme;

pub use app::HardwareMasterApp;
pub use font::setup_chinese_fonts;
pub use theme::AppTheme;
