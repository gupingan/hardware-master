use hardware_master::detector::disk::detect_disk;

#[test]
fn test_disk_detection() {
    match detect_disk() {
        Ok(info) => {
            println!("Disk Info:");
            println!("  Model: {}", info.model);
            println!("  Total Capacity: {} MB", info.total_capacity);
            println!("  Type: {:?}", info.disk_type);

            // 验证至少有磁盘数据
            assert!(info.total_capacity > 0.0,
                    "Expected disk capacity > 0, but got: {}", info.total_capacity);
            assert!(!info.model.is_empty() || info.model != "未知",
                    "Expected disk model, but got: {}", info.model);
        }
        Err(e) => {
            panic!("Disk detection failed: {:?}", e);
        }
    }
}
