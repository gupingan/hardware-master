/// 宏：生成硬件检测方法
///
/// 这个宏用于减少重复的检测方法代码
///
/// # 参数
/// * `$name` - 检测方法名称
/// * `$field` - HardwareDetector 中的字段名
/// * `$module` - 检测函数所在的模块
/// * `$detect_fn` - 模块中的检测函数名
/// * `$error_variant` - DetectionError 的错误变体
#[macro_export]
macro_rules! impl_detect_method {
    ($name:ident, $field:ident, $module:ident, $detect_fn:ident, $error_variant:ident) => {
        fn $name(&mut self) -> Result<(), DetectionError> {
            self.$field = $module::$detect_fn()
                .map_err(|e| DetectionError::$error_variant(e.to_string()))?;
            Ok(())
        }
    };
}

/// 宏：格式化字节大小
#[macro_export]
macro_rules! format_mb_size {
    ($size:expr) => {
        if ($size as f64) < crate::constants::MEMORY_THRESHOLD_MB {
            format!("{:.} MB", $size)
        } else {
            format!(
                "{:.0} GB",
                f64::round($size as f64 / crate::constants::MEMORY_THRESHOLD_MB)
            )
        }
    };
}