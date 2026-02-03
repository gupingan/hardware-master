use hardware_master::ui::theme::AppTheme;

#[test]
fn test_theme_next() {
    // 测试主题切换循环
    assert_eq!(AppTheme::System.next(), AppTheme::Light);
    assert_eq!(AppTheme::Light.next(), AppTheme::Dark);
    assert_eq!(AppTheme::Dark.next(), AppTheme::System);
}

#[test]
fn test_theme_from_str() {
    // 测试从字符串解析主题
    assert_eq!(AppTheme::from_str("系统"), AppTheme::System);
    assert_eq!(AppTheme::from_str("亮色"), AppTheme::Light);
    assert_eq!(AppTheme::from_str("暗色"), AppTheme::Dark);
    assert_eq!(AppTheme::from_str("未知"), AppTheme::System); // 未知值默认为系统主题
}

#[test]
fn test_theme_name() {
    // 测试主题名称
    assert_eq!(AppTheme::System.name(), "系统");
    assert_eq!(AppTheme::Light.name(), "亮色");
    assert_eq!(AppTheme::Dark.name(), "暗色");
}

#[test]
fn test_theme_display() {
    // 测试主题显示
    let theme = AppTheme::System;
    assert_eq!(format!("{}", theme), "系统");
    assert_eq!(format!("{}", AppTheme::Light), "亮色");
    assert_eq!(format!("{}", AppTheme::Dark), "暗色");
}
