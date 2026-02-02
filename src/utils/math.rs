//! 数学计算工具模块

/// 将厘米转换为英寸
///
/// 示例
/// ```
/// use hardware_master::utils::math::cm_to_inches;
/// assert_eq!(cm_to_inches(2.54), 1.0);
/// ```
pub fn cm_to_inches(cm: f64) -> f64 {
    cm / 2.54
}

/// 根据宽度和高度计算显示器的对角线长度（英寸）
///
/// 使用勾股定理计算对角线长度，然后通过[cm_to_inches]将厘米转换为英寸。
///
/// 示例
/// ```
/// use hardware_master::utils::math::diagonal_inches_from_cm;
/// // 27 英寸显示器约为 59.77cm x 33.62cm
/// let diagonal = diagonal_inches_from_cm(59.77, 33.62);
/// assert!((diagonal - 27.0).abs() < 0.1);
/// ```
pub fn diagonal_inches_from_cm(w_cm: f64, h_cm: f64) -> f64 {
    let diag_cm = (w_cm * w_cm + h_cm * h_cm).sqrt();
    cm_to_inches(diag_cm)
}

/// 浮点除法，自动检测分母是否为 0
///
/// 示例
/// ```
/// use hardware_master::utils::math::div;
/// assert_eq!(div(1.0, 2.0), 0.5);
/// assert_eq!(div(1.0, 0.0), std::f64::INFINITY);
/// ```
pub fn div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        std::f64::INFINITY
    } else {
        a / b
    }
}
