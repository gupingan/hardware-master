#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use egui::IconData;
use hardware_master::{
    constants::{WINDOW_HEIGHT, WINDOW_MIN_HEIGHT, WINDOW_MIN_WIDTH, WINDOW_WIDTH},
    ui::HardwareMasterApp,
    NAME, VERSION,
};
use image;
use log::LevelFilter;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志记录器
    env_logger::builder()
        .filter(Some(&NAME.replace("-", "_")), LevelFilter::Debug)
        .init();

    // 应用图标创建
    let icon_bytes = include_bytes!("assets/icons/icon.webp");
    let icon_img = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::WebP)
        .expect("读取图标文件失败");
    let rgba_data = icon_img.into_rgba8();
    let (w, h) = (rgba_data.width(), rgba_data.height());
    let raw_data: Vec<u8> = rgba_data.into_raw();
    let icon = IconData {
        rgba: raw_data,
        width: w,
        height: h,
    };

    // 创建窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT])
            .with_max_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_maximize_button(false),
        ..Default::default()
    };

    // 运行应用程序
    eframe::run_native(
        &format!(
            "硬大师 v{} - 硬件概要信息获取 (数据仅供参考, 仅用于学习使用)",
            VERSION
        ),
        options,
        Box::new(|cc| Ok(Box::new(HardwareMasterApp::new(cc)))),
    )
}
