pub mod types;
pub mod utils;

use std::iter;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::{Context, Result};
use hoodmem::scanner::ScanFilter;
use hoodmem::Process;
use types::*;

use self::utils::GenericScanFilter;

pub struct Core {
    process: Option<Arc<dyn Process>>,
    scanner: Option<hoodmem::scanner::Scanner>,
    attach_status: AttachStatus,
    scan_status: ScanStatus
}

impl Default for Core {
    fn default() -> Self {
        Self {
            process: Default::default(),
            scanner: Default::default(),
            attach_status: Default::default(),
            scan_status: Default::default()
        }
    }
}

impl Core {
    /// Attempt to attach to the target process.
    pub fn attach(&mut self, target: &AttachTarget) -> Result<()> {
        match &self.attach_status {
            AttachStatus::Detached => {
                let attach_status = match target {
                    AttachTarget::Process(pid) => {
                        self.process = Some(hoodmem::attach_external(*pid)?);
                        self.attach_status = AttachStatus::Attached(target.clone());
                        Ok(())
                    }
                    AttachTarget::Window(window_name) => {
                        self.process = Some(hoodmem::attach_external_by_name(window_name)?);
                        self.attach_status = AttachStatus::Attached(target.clone());
                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!(
                        "Attach not yet implemented for target: {:?}",
                        target
                    )),
                };
                if let Some(process) = &self.process {
                    self.scanner = Some(hoodmem::scanner::Scanner::new(process.clone()));
                };
                Ok(())
            }
            AttachStatus::Attached(target) => {
                Err(anyhow::anyhow!("Already attached to {:?}", target))
            }
            AttachStatus::Unknown => Err(anyhow::anyhow!(
                "MemNinja Core attach status is currently unknown"
            )),
        }
    }

    /// Detach from the current process
    pub fn detach(&mut self) {
        self.process = None;
        self.scanner = None;
        self.attach_status = AttachStatus::Detached;
    }
}

pub struct CoreController {
    core: Arc<Mutex<Core>>,
    core_thread: Option<JoinHandle<()>>,
    running: bool,
    core_tx: Option<crossbeam_channel::Sender<CoreCommand>>,
}

impl Default for CoreController {
    fn default() -> Self {
        Self {
            core: Default::default(),
            core_thread: None,
            running: false,
            core_tx: None,
        }
    }
}

impl CoreController {
    /// Start MemNinja Core
    pub fn start(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded::<CoreCommand>();
        self.core_tx = Some(tx);
        let core = self.core.clone();
        self.core_thread = Some(std::thread::spawn(move || loop {
            let command = rx.recv();
            if let Ok(mut core) = core.lock() {
                if let Ok(command) = command {
                    let result = command.execute(&mut core);
                    if let Err(err) = result {
                        eprintln!("Failed to execute command {:?}. Error: {:?}", command, err);
                    }
                }
            } else {
                eprintln!(
                    "Failed to accquire MemNinja Core lock. Dropping command: {:?}",
                    command
                );
            }
        }));
        self.running = true;
        Ok(())
    }

    /// Stop MemNinja Core
    pub fn stop(&mut self) -> Result<()> {
        match self.send_command(CoreCommand::Stop) {
            Ok(_) => {
                self.running = false;
                if let Some(core_thread) = self.core_thread.take() {
                    match core_thread.join() {
                        std::result::Result::Ok(_) => (),
                        Err(err) => {
                            return Err(anyhow::anyhow!(
                                "Error joining MemNinja core thread: {:?}",
                                err
                            ))
                        }
                    };
                };
            }
            Err(err) => return Err(err),
        }
        Ok(())
    }

    /// Sends a command to MemNinja Core
    pub fn send_command(&self, command: CoreCommand) -> Result<()> {
        if let Some(tx) = self.core_tx.as_ref() {
            tx.send(command)?;
        }
        Ok(())
    }

    /// Gets the attach status of MemNinja Core
    pub fn get_attach_status(&self) -> AttachStatus {
        if let Ok(core) = self.core.lock() {
            core.attach_status.clone()
        } else {
            AttachStatus::Unknown
        }
    }

    /// Gets the scan status of MemNinja Core
    pub fn get_scan_status(&self) -> ScanStatus {
        if let Ok(core) = self.core.lock() {
            core.scan_status.clone()
        } else {
            ScanStatus::Unknown
        }
    }

    /// Gets the first n results
    pub fn get_first_results(&self, scan_type: MemType, n: usize) -> Vec<(u64, String)> {
        if let Ok(core) = self.core.lock() {
            if let Some(scanner) = core.scanner.as_ref() {
                match scan_type {
                    MemType::U8 => scanner.get_first_results::<u8>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::U16 => scanner.get_first_results::<u16>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::U32 => scanner.get_first_results::<u32>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::U64 => scanner.get_first_results::<u32>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::I8 => scanner.get_first_results::<i8>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::I16 => scanner.get_first_results::<i16>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::I32 => scanner.get_first_results::<i32>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::I64 => scanner.get_first_results::<i64>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::F32 => scanner.get_first_results::<f32>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::F64 => scanner.get_first_results::<f64>(n).iter().map(|(addr, v)| (*addr, format!("{:#?}", v))).collect(),
                    MemType::Unknown => vec![],
                }
            }
            else {
                vec![]
            }
        }
        else {
            vec![]
        }
    }

    /// Checks whether MemNinja core is currently attached to something
    pub fn check_attached(&self) -> bool {
        if let Ok(core) = self.core.lock() {
            match core.attach_status {
                AttachStatus::Detached => false,
                AttachStatus::Attached(_) => true,
                AttachStatus::Unknown => false,
            }
        } else {
            false
        }
    }
}

/// A command to send to MemNinja Core
#[derive(Debug)]
pub enum CoreCommand {
    /// Attach to a process
    Attach(AttachTarget),
    /// Detach from ther current process
    Detach,
    /// The unknown command. Does nothing
    Unknown,
    /// Shut down the MemNinja core thread
    Stop,
    /// Initializes a new scan
    NewScan,
    /// Performs a scan with the given `GenericScanFilter`
    Scan(GenericScanFilter),
}

impl CoreCommand {
    pub fn execute(&self, core: &mut Core) -> anyhow::Result<()> {
        match self {
            CoreCommand::Attach(target) => {
                core.attach(target)?;
            }
            CoreCommand::Detach => {
                core.detach();
            },
            CoreCommand::Stop => {
                // TODO: I guess something probably could be done here.
            }
            CoreCommand::Unknown => {
                eprintln!("Attempted to run an unknown command");
            },
            CoreCommand::NewScan => {
                if let Some(scanner) = &mut core.scanner {
                    scanner.new_scan();
                }
            },
            CoreCommand::Scan(filter) => {
                core.scan_status = ScanStatus::Scanning;
                if let Some(scanner) = &mut core.scanner {
                    let result = filter.scan(scanner);
                    let num_results = scanner.count_results().unwrap_or(0);
                    core.scan_status = match result {
                        Ok(_) => {
                            ScanStatus::Done(num_results as u64)
                        },
                        Err(err) => ScanStatus::Failed(err.to_string()),
                    };
                }
            }
        };
        Ok(())
    }
}
