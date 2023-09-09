fn main() -> hoodmem::Result<()> {
    let process = hoodmem::Process::attach_by_name("*AAA - Notepad")?;
    let mut u8_scanner = hoodmem::scanner::KnownIVScanner::<u8>::new(process);
    u8_scanner.new_scan();
    println!("Scanning for 'A' (0x41)");
    u8_scanner.scan(0x41)?;
    println!("Found {} results", u8_scanner.results.len());
        u8_scanner
            .results
            .keys()
            .into_iter()
            .take(100)
            .map(|addr| {
                format!(
                    "0x{:016x} : {:016}",
                    addr,
                    u8_scanner.results.get(addr).unwrap().0
                )
            })
            .for_each(|s| {
                println!("{}", s);
            });
    Ok(())
}
