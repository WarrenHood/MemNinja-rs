use hoodmem::{self, Process};

fn main() -> hoodmem::Result<()> {
    let process: Process = hoodmem::process_attach(59548)?;
    let writable_regions = process.get_writable_regions();
    for region in writable_regions {
        println!(
            "Detected memory region: 0x{:x} -> 0x{:x}",
            region.base_address,
            region.base_address + region.size
        );
        let first_few_bytes = process.read_memory_bytes(region.base_address, 100);
        if let Ok(first_few_bytes) = first_few_bytes {
            println!("First 100 bytes: {:?}", &first_few_bytes);
            println!("First 100 bytes as lossy UTF-8 string: {}", String::from_utf8_lossy(&first_few_bytes));
        }
    }
    Ok(())
}
