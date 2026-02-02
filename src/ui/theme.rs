//! 主题模块
//!
//! 提供主题切换功能，支持系统、亮色、暗色三种主题

use eframe::egui;

/// 应用主题
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    /// 系统主题（跟随系统设置）
    System,
    /// 亮色主题
    Light,
    /// 暗色主题
    Dark,
}

impl AppTheme {
    /// 获取主题名称
    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::System => crate::constants::THEME_SYSTEM,
            AppTheme::Light => crate::constants::THEME_LIGHT,
            AppTheme::Dark => crate::constants::THEME_DARK,
        }
    }

    /// 获取下一个主题
    pub fn next(&self) -> Self {
        match self {
            AppTheme::System => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
            AppTheme::Dark => AppTheme::System,
        }
    }

    /// 应用主题到 egui 上下文
    pub fn apply(&self, ctx: &egui::Context) {
        match self {
            AppTheme::System => {
                // 系统主题：不强制设置，让 egui 自动检测
                let mut style = (*ctx.style()).clone();
                style.visuals = egui::Visuals::default(); // 使用默认（系统）主题
                ctx.set_style(style);
            }
            AppTheme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            AppTheme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    /// 从字符串解析主题
    pub fn from_str(s: &str) -> Self {
        match s {
            crate::constants::THEME_SYSTEM => AppTheme::System,
            crate::constants::THEME_LIGHT => AppTheme::Light,
            crate::constants::THEME_DARK => AppTheme::Dark,
            _ => AppTheme::System,
        }
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::System
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
