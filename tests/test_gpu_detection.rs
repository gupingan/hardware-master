use hardware_master::detector::gpu::detect_gpu;

#[test]
fn test_gpu_detection() {
    match detect_gpu() {
        Ok(info) => {
            println!("GPU Info:");
            println!("  Total GPUs: {}", info.gpus.len());

            for (i, gpu) in info.gpus.iter().enumerate() {
                println!("  GPU {}:", i);
                println!("    Description: {}", gpu.description);
                println!("    Manufacturer: {}", gpu.manufacturer);
                println!("    Chip Vendor: {}", gpu.chip_vendor);
                println!("    Type: {:?}", gpu.gpu_type);
                println!("    VRAM: {:.0} MB", gpu.vram_size);
                println!("    Device ID: {}", gpu.device_id);
                println!("    Vendor ID: {}", gpu.vendor_id);
            }

            // 验证至少有一个 GPU
            assert!(!info.gpus.is_empty(),
                    "Expected at least one GPU, but got none");
        }
        Err(e) => {
            panic!("GPU detection failed: {:?}", e);
        }
    }
}
