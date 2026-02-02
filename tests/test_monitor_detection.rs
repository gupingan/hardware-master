use hardware_master::detector::monitor::detect_display;

#[test]
fn test_monitor_detection() {
    match detect_display() {
        Ok(info) => {
            println!("Monitor Info:");
            println!("  Name: {}", info.name);
            println!("  Manufacturer: {}", info.manufacturer);
            println!("  Size: {:.1} inches", info.size_desc);
            println!("  Manufacture Date: {}", info.manufacture_date);

            // 验证至少有显示器数据
            assert!(
                info.size_desc > 0.0 || !info.name.is_empty(),
                "Expected some monitor data, but got size_desc={} and name='{}'",
                info.size_desc,
                info.name
            );

            // 验证制造商不为空
            assert!(
                !info.manufacturer.is_empty(),
                "Expected manufacturer to not be empty"
            );
        }
        Err(e) => {
            panic!("Monitor detection failed: {:?}", e);
        }
    }
}
