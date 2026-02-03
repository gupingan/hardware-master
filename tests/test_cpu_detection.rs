use hardware_master::detector::cpu::detect_cpu;

#[test]
fn test_cpu_detection() {
    match detect_cpu() {
        Ok(info) => {
            println!("CPU Info:");
            println!("  Name: {}", info.name);
            println!("  Architecture: {}", info.architecture);
            println!("  Cores: {}", info.cores);
            println!("  Vendor: {}", info.vendor);
            println!("  CPU ID: {}", info.cpu_id);
            println!("  Max Frequency: {} MHz", info.max_frequency);

            // 验证至少有 CPU 名称
            assert!(!info.name.is_empty() || info.name != "未知",
                    "Expected CPU name, but got: {}", info.name);
        }
        Err(e) => {
            panic!("CPU detection failed: {:?}", e);
        }
    }
}
