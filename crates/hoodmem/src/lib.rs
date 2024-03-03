mod platforms;
pub mod scanner;
pub mod util;

pub use anyhow::Result;
pub use std::ffi::{c_void, CString};

#[cfg(target_os = "windows")]
pub use crate::platforms::windows::*;

#[cfg(target_os = "linux")]
pub use crate::platforms::linux::*;

pub trait MemoryRead {
    fn read_memory_bytes(&self, address: u64, bytes_to_read: usize) -> Result<Vec<u8>>;
}

pub trait GenericMemoryRead<T: Copy> {
    fn read_memory(&self, address: u64) -> Result<T>;
}

impl<T: MemoryRead, U: Copy> GenericMemoryRead<U> for T {
    fn read_memory(&self, address: u64) -> Result<U> {
        let result: Vec<u8> = self.read_memory_bytes(address, std::mem::size_of::<U>())?;
        unsafe {
            Ok(*std::mem::transmute::<*const Vec<u8>, *const U>(
                &result as *const Vec<u8>,
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub size: u64,
}

pub trait ScannableMemoryRegions {
    fn get_writable_regions(&self) -> Vec<MemoryRegion>;
}

pub fn attach_external(pid: u32) -> Result<Box<dyn Process>> {
    #[cfg(target_os = "windows")]
    return Ok(Box::new(WinProcess::attach(pid)?));

    #[cfg(target_os = "linux")]
    return Ok(Box::new(LinuxProcess::attach(pid)))
}

pub fn attach_external_by_name(name: &str) -> Result<Box<dyn Process>> {
    #[cfg(target_os = "windows")]
    return Ok(Box::new(WinProcess::attach_by_name(name)?));
    #[cfg(target_os = "linux")]
    return Ok(Box::new(LinuxProcess::attach_by_proc_name(name)))
}

pub trait Process: MemoryRead + ScannableMemoryRegions + 'static + Send + Sync {}
impl<T: MemoryRead + ScannableMemoryRegions + 'static + Send + Sync> Process for T {}
