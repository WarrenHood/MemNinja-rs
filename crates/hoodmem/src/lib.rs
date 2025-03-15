mod platforms;
pub mod scanner;
pub mod util;

pub use anyhow::Result;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
pub use std::ffi::{c_void, CString};
use std::sync::Arc;

#[cfg(target_os = "windows")]
pub use crate::platforms::windows::*;

#[cfg(target_os = "linux")]
pub use crate::platforms::linux::*;

pub trait Process: Send + Sync {
    fn read_memory_bytes(&self, address: usize, bytes_to_read: usize) -> Result<Vec<u8>>;

    fn get_writable_regions(&self) -> Vec<MemoryRegion>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub base_address: usize,
    pub size: usize,
}

/// Attach to an external process on the native system
pub fn attach_external(pid: u32) -> Result<Arc<dyn Process>> {
    #[cfg(target_os = "linux")]
    LinuxProcess::attach_external(pid)
}

pub fn attach_remote(addr: &str) -> Result<Arc<dyn Process>> {
    Ok(Arc::new(platforms::remote::RemoteProcess::connect(addr)?))
}

/// Attach to an external process on the native system
pub fn attach_external_by_name(name: &str) -> Result<Arc<dyn Process>> {
    unimplemented!()
}

pub fn attach_external_and_run_server(pid: u32, _addr: &str) {
    loop {
        let server = platforms::remote::RemoteProcessServer::listen(pid);
        if let Ok(server) = server {
            server.run();
        } else {
            eprintln!("Failed to create server daemon for process {pid}");
        }
    }
}
