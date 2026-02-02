//! WMI 日期解析工具模块

/// 解析 WMI 日期格式
///
/// WMI 日期格式: yyyymmddHHMMSS.mmmmmm+UUU
/// 该函数将其转换为更易读的格式: yyyy-mm-dd
///
/// 示例
/// ```
/// use hardware_master::utils::wmi_date::parse_wmi_date;
/// assert_eq!(parse_wmi_date("20230101120000.000000+000"), "2023-01-01");
/// ```
pub fn parse_wmi_date(date: &str) -> String {
    if date.len() >= 8 {
        let year = &date[0..4];
        let month = &date[4..6];
        let day = &date[6..8];
        format!("{}-{}-{}", year, month, day)
    } else {
        date.to_string()
    }
}
