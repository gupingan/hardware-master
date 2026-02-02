use hardware_master::detector::memory::detect_memory;
#[test]
fn test_memory_detection() {
    match detect_memory() {
        Ok(info) => {
            println!("Memory Info:");
            println!("  Name: {}", info.name);
            println!("  Total Memory: {} MB", info.total_memory);
            println!("  Slots: {}", info.slots.len());
            
            for (i, slot) in info.slots.iter().enumerate() {
                println!("  Slot {}:", i);
                println!("    Name: {}", slot.name);
                println!("    Capacity: {} MB", slot.capacity);
                println!("    Manufacturer: {}", slot.manufacturer);
                println!("    Type: {}", slot.memory_type);
                println!("    Frequency: {} MHz", slot.frequency);
            }
            
            // 验证至少有内存数据
            assert!(info.total_memory > 0.0 || !info.slots.is_empty(),
                    "Expected some memory data, but got total_memory={} and {} slots",
                    info.total_memory, info.slots.len());
        }
        Err(e) => {
            panic!("Memory detection failed: {:?}", e);
        }
    }
}