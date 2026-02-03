use hardware_master::detector::memory::{generate_total_name, MemorySlot};

const BYTES_B: f64 = 1024.0 * 1024.0;

#[test]
fn test_same_memory_slots() {
    let slots = vec![
        MemorySlot {
            name: "英睿达 DDR4 3200MHz 8GB".to_string(),
            capacity: 8192.0 * BYTES_B,
            ..Default::default()
        },
        MemorySlot {
            name: "英睿达 DDR4 3200MHz 8GB".to_string(),
            capacity: 8192.0 * BYTES_B,
            ..Default::default()
        },
    ];
    assert_eq!("16 GB (英睿达 DDR4 3200MHz 8GB x 2)", generate_total_name(&slots));
}

#[test]
fn test_mixed_memory_slots() {
    let slots = vec![
        MemorySlot {
            name: "金士顿 DDR4 2666MHz 4GB".to_string(),
            capacity: 4096.0 * BYTES_B,
            ..Default::default()
        },
        MemorySlot {
            name: "三星 DDR4 3200MHz 16GB".to_string(),
            capacity: 16384.0 * BYTES_B,
            ..Default::default()
        },
        MemorySlot {
            name: "金士顿 DDR4 2666MHz 4GB".to_string(),
            capacity: 4096.0 * BYTES_B,
            ..Default::default()
        },
    ];
    assert_eq!("24 GB (三星 DDR4 3200MHz 16GB x 1  金士顿 DDR4 2666MHz 4GB x 2)", generate_total_name(&slots));
}

#[test]
fn test_single_memory_slot() {
    let slots = vec![MemorySlot {
        name: "海盗船 DDR5 4800MHz 32GB".to_string(),
        capacity: 32768.0 * BYTES_B,
        ..Default::default()
    }];
    assert_eq!("32 GB (海盗船 DDR5 4800MHz 32GB x 1)", generate_total_name(&slots));
}

#[test]
fn test_empty_slots() {
    let slots: Vec<MemorySlot> = vec![];
    assert_eq!("0 B ()", generate_total_name(&slots));
}
