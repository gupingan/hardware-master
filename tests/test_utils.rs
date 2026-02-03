use hardware_master::utils::{
    math::{cm_to_inches, div},
    string::{u16_bytes_to_string, u16_slice_to_string, u8_slice_to_string},
    wmi_date::parse_wmi_date,
};

#[test]
fn test_math_functions() {
    // 测试厘米到英寸转换
    assert_eq!(cm_to_inches(25.4), 10.0);
    assert_eq!(cm_to_inches(50.8), 20.0);

    // 测试除法
    assert_eq!(div(10.0, 2.0), 5.0);
    assert_eq!(div(10.0, 3.0), 3.3333333333333335);
}

#[test]
fn test_string_functions() {
    // 测试 u16 字节数组转字符串
    let bytes = vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x00, 0x00];
    let result = u16_bytes_to_string(&bytes);
    assert_eq!(result, "Hello");

    // 测试 u16 切片转字符串
    let slice = vec![0x48, 0x65, 0x6C, 0x68, 0x61, 0x72, 0x73, 0x74, 0x75, 0x00];
    let result = u16_slice_to_string(&slice);
    assert_eq!(result, "Helharstu");

    // 测试 u8 切片转字符串
    let slice = vec![72, 101, 108, 111, 108, 32, 98, 111, 100, 0];
    let result = u8_slice_to_string(&slice);
    assert_eq!(result, "Helol bod");
}

#[test]
fn test_wmi_date_parsing() {
    // 测试 WMI 日期解析
    let date_str = "20240101120000.000000+000";
    let result = parse_wmi_date(date_str);
    assert!(!result.is_empty());
}
