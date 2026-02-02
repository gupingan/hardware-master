//! 字体加载模块
//!
//! 提供跨平台的中文字体加载功能

use egui::FontDefinitions;

/// 获取系统可用的中文字体路径列表
///
/// 根据不同的操作系统返回相应平台的中文字体路径
pub fn get_chinese_font_paths() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        vec![
            "C:\\Windows\\Fonts\\msyh.ttc",    // 微软雅黑
            "C:\\Windows\\Fonts\\msyhbd.ttc",  // 微软雅黑粗体
            "C:\\Windows\\Fonts\\simsun.ttc",  // 宋体
            "C:\\Windows\\Fonts\\simhei.ttf",  // 黑体
            "C:\\Windows\\Fonts\\simkai.ttf",  // 楷体
            "C:\\Windows\\Fonts\\simfang.ttf", // 仿宋
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            "/System/Library/Fonts/PingFang.ttc",      // 苹方
            "/System/Library/Fonts/STHeiti Light.ttc", // 华文黑体
            "/System/Library/Fonts/STSong.ttc",        // 华文宋体
            "/System/Library/Fonts/STKaiti.ttc",       // 华文楷体
            "/Library/Fonts/Arial Unicode.ttf",        // Arial Unicode
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", // 文泉驿微米黑
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",   // 文泉驿正黑
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", // Noto Sans CJK
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // 其他平台：依赖 egui 默认字体
        vec![]
    }
}

/// 设置中文字体
///
/// 尝试加载系统中的中文字体，如果加载失败则使用默认字体
///
/// # 参数
/// * `ctx` - egui 上下文
///
/// # 返回值
/// 返回是否成功加载了中文字体
pub fn setup_chinese_fonts(ctx: &egui::Context) -> bool {
    let mut fonts = FontDefinitions::default();
    let font_paths = get_chinese_font_paths();

    let mut font_loaded = false;

    for font_path in font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts
                .font_data
                .insert(font_path.to_string(), egui::FontData::from_owned(font_data));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, font_path.to_string());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(font_path.to_string());
            font_loaded = true;
            log::info!("成功加载中文字体: {}", font_path);
            break;
        }
    }

    if !font_loaded {
        log::warn!("无法加载中文字体，中文可能显示为方框");
    }

    ctx.set_fonts(fonts);
    font_loaded
}
