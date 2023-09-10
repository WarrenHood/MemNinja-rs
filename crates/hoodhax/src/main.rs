use std::{str::FromStr, io::Write};

use hoodmem::scanner::ScanFilter;

#[derive(Debug, Clone, Copy)]
enum ScanType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

fn do_scan<T>(scanner: &mut hoodmem::scanner::Scanner, command: &[&str]) -> anyhow::Result<()>
where
    T: Copy
        + std::fmt::Debug
        + Send
        + Sync
        + PartialOrd
        + PartialEq
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + FromStr,
{
    match command.len() {
        0 => {
            eprintln!("Expected one of: exact, changed, unchanged, increased, decreased, increasedby, decreasedby, unknown");
        }
        1 => match command[0].trim() {
            "changed" => {
                scanner.scan(ScanFilter::Unchanged::<T>)?;
            }
            "unchanged" => {
                scanner.scan(ScanFilter::Unchanged::<T>)?;
            }
            "increased" => {
                scanner.scan(ScanFilter::Increased::<T>)?;
            }
            "decreased" => {
                scanner.scan(ScanFilter::Decreased::<T>)?;
            }
            "unknown" => {
                scanner.scan(ScanFilter::Unknown::<T>)?;
            }
            _ => {
                eprintln!("Expected one of: exact, changed, unchanged, increased, decreased, increasedby, decreasedby, unknown");
            }
        },
        2 => {
            match command[0].trim() {
                "exact" => {
                    if let Ok(value) = T::from_str(command[1].trim()) {
                        scanner.scan(ScanFilter::Exact(value))?;
                    } else {
                        eprintln!("Unable to parse value {}", command[1].trim());
                    }
                }
                "increasedby" => {
                    if let Ok(value) = T::from_str(command[1].trim()) {
                        scanner.scan(ScanFilter::IncreasedBy(value))?;
                    } else {
                        eprintln!("Unable to parse value {}", command[1].trim());
                    }
                }
                "decreasedby" => {
                    if let Ok(value) = T::from_str(command[1].trim()) {
                        scanner.scan(ScanFilter::DecreasedBy(value))?;
                    } else {
                        eprintln!("Unable to parse value {}", command[1].trim());
                    }
                }
                _ => {
                    eprintln!("Unknown command {}", command[0].trim())
                }
            }
        }
        _ => {
            eprintln!("Unexpected command length {}", command.len());
        }
    };
    Ok(())
}

fn do_scan_with_results<T>(scanner: &mut hoodmem::scanner::Scanner, command: &[&str])
where
    T: Copy
        + std::fmt::Debug
        + Send
        + Sync
        + PartialOrd
        + PartialEq
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + FromStr
        ,
{
    let scan_result = do_scan::<T>(scanner, command);
    if let Err(scan_err) = scan_result {
        eprintln!("Scan failed due to {}", scan_err);
    } else {
        println!("Scan was successful");
        let results = scanner.get_results::<T>();
        println!("{} Results found (at most first 100 shown)", results.len());
        results
            .into_iter()
            .take(100)
            .for_each(|(addr, value)| println!("0x{:016x}: {:?}", addr, value));
    }
}

fn main() -> hoodmem::Result<()> {
    let mut process: Option<hoodmem::Process> = None;
    let mut scanner: Option<hoodmem::scanner::Scanner> = None;
    let mut scan_type: ScanType = ScanType::U32;

    loop {
        print!("hoodhax> ");
        let _ = std::io::stdout().flush();
        let stdin = std::io::stdin();
        let mut command = String::new();
        if stdin.read_line(&mut command).is_ok() {
            let command: Vec<&str> = command.split(' ').collect();
            if command.len() > 0 {
                match command[0].trim() {
                    "attach" => {
                        if command.len() == 2 {
                            if let Ok(pid) = command[1].trim().parse::<u32>() {
                                if let Ok(attach_result) = hoodmem::Process::attach(pid) {
                                    process = Some(attach_result);
                                    scanner =
                                        Some(hoodmem::scanner::Scanner::new(process.unwrap()));
                                    println!("Successfully attached to process with PID {}", pid);
                                } else {
                                    eprintln!("Failed to attach to process with PID {}", pid);
                                }
                            } else {
                                eprintln!("Unable to parse PID {}", command[1].trim());
                            }
                        } else {
                            eprintln!("Expected a PID to attach to");
                        }
                    }
                    "newscan" => {
                        if let Some(scanner) = scanner.as_mut() {
                            scanner.new_scan();
                        } else {
                            eprintln!("Scanner not yet initialized. Please attach to a process first with `attach <pid>`");
                        }
                    }
                    "scantype" => {
                        if command.len() == 2 {
                            match command[1].trim() {
                                "u8" => scan_type = ScanType::U8,
                                "u16" => scan_type = ScanType::U16,
                                "u32" => scan_type = ScanType::U32,
                                "u64" => scan_type = ScanType::U64,
                                "i8" => scan_type = ScanType::I8,
                                "i16" => scan_type = ScanType::I16,
                                "i32" => scan_type = ScanType::I32,
                                "i64" => scan_type = ScanType::I64,
                                "f32" => scan_type = ScanType::F32,
                                "f64" => scan_type = ScanType::F64,
                                _ => {
                                    eprintln!("Unknown scan type '{}'", command[1].trim());
                                }
                            }
                        } else {
                            eprintln!("Expected a scan type ({{u,i}}{{8,16,32,64}} or f{{32,64}})");
                        }
                    }
                    "scan" => {
                        if let Some(scanner) = scanner.as_mut() {
                            match scan_type {
                                ScanType::U8 => do_scan_with_results::<u8>(scanner, &command[1..]),
                                ScanType::U16 => {
                                    do_scan_with_results::<u16>(scanner, &command[1..])
                                }
                                ScanType::U32 => {
                                    do_scan_with_results::<u32>(scanner, &command[1..])
                                }
                                ScanType::U64 => {
                                    do_scan_with_results::<u64>(scanner, &command[1..])
                                }
                                ScanType::I8 => do_scan_with_results::<i8>(scanner, &command[1..]),
                                ScanType::I16 => {
                                    do_scan_with_results::<i16>(scanner, &command[1..])
                                }
                                ScanType::I32 => {
                                    do_scan_with_results::<i32>(scanner, &command[1..])
                                }
                                ScanType::I64 => {
                                    do_scan_with_results::<i64>(scanner, &command[1..])
                                }
                                ScanType::F32 => {
                                    do_scan_with_results::<f32>(scanner, &command[1..])
                                }
                                ScanType::F64 => {
                                    do_scan_with_results::<f64>(scanner, &command[1..])
                                }
                            };
                        } else {
                            eprintln!("Scanner not yet initialized. Please attach to a process first with `attach <pid>`");
                        }
                    }
                    "getresults" => {
                        if let Some(scanner) = scanner.as_ref() {
                            scanner.results.values().for_each(|r| r.print::<u8>());
                        }
                    }
                    "quit" => break,
                    _ => {
                        println!("Unknown command '{}'", command[0].trim());
                    }
                }
            }
        }
    }

    Ok(())
}
