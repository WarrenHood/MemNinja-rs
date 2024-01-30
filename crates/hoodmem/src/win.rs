use windows::core::PCSTR;
pub use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY, PAGE_PROTECTION_FLAGS, PAGE_READWRITE, PAGE_WRITECOPY,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, GetWindowThreadProcessId};

#[derive(Debug, Clone, Copy)]
pub struct WinProcess {
    handle: HANDLE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub size: u64,
}

impl WinProcess {
    pub fn get_writable_regions(&self) -> Vec<MemoryRegion> {
        let page_protection: PAGE_PROTECTION_FLAGS =
            PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
        let system_info = get_system_info();
        let mut address: u64 =
            unsafe { std::mem::transmute(system_info.lpMinimumApplicationAddress) };
        let max_address: u64 =
            unsafe { std::mem::transmute(system_info.lpMaximumApplicationAddress) };
        let mut regions: Vec<MemoryRegion> = Vec::new();

        let mut memory_basic_info: MEMORY_BASIC_INFORMATION = Default::default();
        while address < max_address {
            let query_result: usize = unsafe {
                VirtualQueryEx(
                    self.handle,
                    Some(std::mem::transmute(address)),
                    &mut memory_basic_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if query_result <= 0 {
                break;
            }

            if (memory_basic_info.State & MEM_COMMIT).0 > 0
                && (memory_basic_info.Protect & page_protection).0 > 0
            {
                regions.push(MemoryRegion {
                    base_address: memory_basic_info.BaseAddress as u64,
                    size: memory_basic_info.RegionSize as u64,
                });
            }
            address = memory_basic_info.BaseAddress as u64 + memory_basic_info.RegionSize as u64;
        }
        regions
    }

    pub fn read_memory_bytes(&self, address: u64, bytes_to_read: usize) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(bytes_to_read);
        unsafe {
            buffer.set_len(bytes_to_read);
            ReadProcessMemory(
                self.handle,
                std::mem::transmute(address),
                std::mem::transmute(buffer.as_mut_ptr()),
                bytes_to_read,
                None,
            )?;
        }
        Ok(buffer)
    }

    pub fn read_memory<T: Copy>(&self, address: u64) -> Result<T> {
        let result: Vec<u8> = self.read_memory_bytes(address, std::mem::size_of::<T>())?;
        unsafe {
            Ok(*std::mem::transmute::<*const Vec<u8>, *const T>(
                &result as *const Vec<u8>,
            ))
        }
    }

    pub fn attach(pid: u32) -> Result<Self> {
        unsafe {
            Ok(Self {
                handle: OpenProcess(PROCESS_ALL_ACCESS, false, pid)?,
            })
        }
    }

    pub fn attach_by_name(window_name: &str) -> Result<Self> {
        if window_name.trim().len() == 0 {
            return Err(anyhow::format_err!("Window name cannot be empty"));
        }
        let window_name = CString::new(window_name)?;
        let mut pid: u32 = 0;
        unsafe {
            let game_window: HWND = FindWindowA(
                PCSTR::null(),
                std::mem::transmute::<*const c_char, PCSTR>(window_name.as_ptr()),
            );
            GetWindowThreadProcessId(game_window, Some(&mut pid));
        };
        if pid != 0 {
            Self::attach(pid)
        } else {
            Err(anyhow::format_err!("Process not found"))
        }
    }
}

pub fn get_system_info() -> SYSTEM_INFO {
    let mut system_info: SYSTEM_INFO = Default::default();
    unsafe {
        GetSystemInfo(&mut system_info);
    }
    system_info
}