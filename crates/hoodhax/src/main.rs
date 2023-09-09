use hoodmem::scanner::ScanFilter;

fn main() -> hoodmem::Result<()> {
    let process = hoodmem::Process::attach_by_name("*AAA - Notepad")?;
    let mut u8_scanner = hoodmem::scanner::Scanner::new(process);
    u8_scanner.new_scan();
    println!("Scanning for unknown values...");
    u8_scanner.scan(ScanFilter::Unknown::<u8>)?;

    println!("Scanning for 'A' (0x41)");
    u8_scanner.scan(ScanFilter::Exact(0x41 as u8))?;

    println!("Scanning for unchanged values");
    u8_scanner.scan(ScanFilter::Unchanged::<u8>)?;

    // Print out the results for now
    u8_scanner.results.values().into_iter().for_each(|result| {
        result.print::<u8>();
    });
    Ok(())
}
