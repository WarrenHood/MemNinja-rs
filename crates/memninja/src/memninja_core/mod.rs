pub mod types;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::{Context, Result};
use hoodmem::Process;
use types::*;

struct Core {
    process: Option<Arc<dyn Process>>,
    scanner: Option<hoodmem::scanner::Scanner>,
    attach_status: AttachStatus,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            process: Default::default(),
            scanner: Default::default(),
            attach_status: Default::default(),
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
                let result = match command.as_ref() {
                    std::result::Result::Ok(cmd) => match cmd {
                        CoreCommand::Attach(target) => core.attach(target),
                        CoreCommand::Detach => {
                            core.detach();
                            Ok(())
                        }
                        CoreCommand::Stop => {
                            println!("MemNinja Core stopped");
                            break;
                        }
                        _ => Err(anyhow::anyhow!("Unimplemented core command: {:?}", cmd)),
                    },
                    Err(err) => Err(anyhow::anyhow!("{:?}", err)),
                };
                if let Err(err) = result {
                    eprintln!(
                        "Failed to execute command {:?}. Error: {:?}",
                        command.unwrap_or(CoreCommand::Unknown),
                        err
                    );
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

    // Stop MemNinja Core
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
}
