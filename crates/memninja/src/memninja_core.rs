use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};

use anyhow::{Result};
use hoodmem::Process;

#[derive(Debug, Clone)]
enum AttachType {
    Process(u32),
    Window(String),
    Other(String)
}

enum AttachStatus {
    Detached,
    Attached(AttachType),
}

impl Default for AttachStatus {
    fn default() -> Self {
        Self::Detached
    }
}

struct Core {
    process: Option<Box<dyn Process>>,
    scanner: Option<hoodmem::scanner::Scanner>,
    attach_status: AttachStatus,
}

impl Default for Core {
    fn default() -> Self {
        Self { process: Default::default(), scanner: Default::default(), attach_status: Default::default() }
    }
}

impl Core {
    /// Attempt to attach to the target process.
    pub fn attach(&mut self, target: &AttachType) -> Result<()> {
        match &self.attach_status {
            AttachStatus::Detached => {
                match target {
                    AttachType::Process(pid) => {
                        self.process = Some(hoodmem::attach_external(*pid)?);
                        Ok(())
                    },
                    AttachType::Window(window_name) => {
                        self.process = Some(hoodmem::attach_external_by_name(window_name)?);
                        Ok(())
                    },
                    _ => Err(anyhow::anyhow!("Attach not yet implemented for target: {:?}", target))
                }
            },
            AttachStatus::Attached(target) => {
                Err(anyhow::anyhow!("Already attached to {:?}", target))
            },
        }
    }

    /// Detach from the current process
    pub fn detach(&mut self) {
        self.process = None;
    }
}

pub struct CoreManager {
    core: Arc<Mutex<Core>>,
    core_thread: Option<JoinHandle<()>>,
    running: bool
}

impl Default for CoreManager {
    fn default() -> Self {
        Self { core: Default::default(), core_thread: None, running: false }
    }
}

impl CoreManager {
    /// Start MemNinja core
    pub fn start(&mut self) {
        let (tx, rx) = crossbeam_channel::unbounded::<CoreCommand>();
        let core = self.core.clone();
        self.core_thread = Some(std::thread::spawn(move|| {
            loop {
                let command = rx.recv();
                if let Ok(mut core) = core.lock() {
                    match command.as_ref() {
                        std::result::Result::Ok(cmd) => {
                            match cmd {
                                CoreCommand::Attach(target) => {
                                    core.attach(target);
                                },
                                CoreCommand::Detach => {
                                    core.detach();
                                },
                            };
                        },
                        Err(err) => {
                            eprintln!("MemNinja Core failed to receive a message: {:?}", err)
                        },
                    }
                }
                else {
                    eprintln!("Failed to accquire MemNinja Core lock. Dropping command: {:?}", command);
                }
            }
        }));
        self.running = true
    }

    // Stop MemNinja core
    pub fn stop(&mut self) -> Result<()> {
        self.running = false;
        if let Some(core_thread) = self.core_thread.take() {
            match core_thread.join() {
                std::result::Result::Ok(_) => (),
                Err(err) => return Err(anyhow::anyhow!("Error stopping MemNinja core: {:?}", err)),
            };
        };
        Ok(())
    }
}

/// A command to send to MemNinja core
#[derive(Debug)]
enum CoreCommand {
    Attach(AttachType),
    Detach
}