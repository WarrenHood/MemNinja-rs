mod platforms;
pub mod scanner;
pub mod util;

pub use anyhow::Result;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryRegion {
    pub base_address: usize,
    pub size: usize,
}

/// Attach to an external process on the native system
pub fn attach_external(pid: u32) -> Result<Arc<dyn Process>> {
    #[cfg(target_os = "linux")]
    LinuxProcess::attach_external(pid)
}
