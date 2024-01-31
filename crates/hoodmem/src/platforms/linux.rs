use std::io::IoSliceMut;

use crate::{MemoryRead, MemoryRegion, ScannableMemoryRegions};
use anyhow::{anyhow, Result};
use nix::{
    sys::uio::{process_vm_readv, RemoteIoVec},
    unistd::Pid,
};
use proc_maps::get_process_maps;

#[derive(Debug, Clone, Copy)]
pub struct LinuxProcess {
    pid: Pid,
}

impl LinuxProcess {
    pub fn attach(pid: u32) -> Self {
        Self {
            pid: Pid::from_raw(pid as i32),
        }
    }

    pub fn attach_by_proc_name(name: &str) -> Self {
        unimplemented!()
    }
}

impl MemoryRead for LinuxProcess {
    fn read_memory_bytes(&self, address: u64, bytes_to_read: usize) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(bytes_to_read);
        unsafe {
            buffer.set_len(bytes_to_read);
        }
        let mut local_iov = [IoSliceMut::new(&mut buffer)];
        let remote_iov = [RemoteIoVec {
            base: address as usize,
            len: bytes_to_read,
        }];
        let bytes_read = process_vm_readv(self.pid, &mut local_iov, &remote_iov)?;
        if bytes_read != bytes_to_read {
            return Err(anyhow!(
                "Failed to read {} bytes from process (pid={}). Only {} bytes read",
                bytes_to_read,
                self.pid,
                bytes_read
            ));
        }

        Ok(buffer)
    }
}

impl ScannableMemoryRegions for LinuxProcess {
    fn get_writable_regions(&self) -> Vec<MemoryRegion> {
        let mut regions = Vec::new();
        if let Ok(maps) = get_process_maps(self.pid.into()) {
            for map in maps {
                if map.is_write() && map.is_read() {
                    regions.push(MemoryRegion {
                        base_address: map.start() as u64,
                        size: map.size() as u64,
                    })
                }
            }
        }

        regions
    }
}
