use hardware_master::detector::system::detect_system;

#[test]
fn test_system_detection() {
    match detect_system() {
        Ok(info) => {
            println!("System Info:");
            println!("  OS Name: {}", info.os_name);
            println!("  Manufacturer: {}", info.system_manufacturer);
            println!("  Model: {}", info.computer_model);
            println!("  Type: {:?}", info.computer_type);

            // 验证至少有系统信息
            assert!(!info.os_name.is_empty() || info.os_name != "未知",
                    "Expected OS name, but got: {}", info.os_name);
        }
        Err(e) => {
            panic!("System detection failed: {:?}", e);
        }
    }
}
