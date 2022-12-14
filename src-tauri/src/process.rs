use crate::region::Region;
use crate::scan::{Scan, Scannable};
use log::warn;
use serde::Serialize;
use std::mem::{self, MaybeUninit};
use std::os::windows::io::{HandleOrNull, NullHandleError, OwnedHandle};
use std::os::windows::prelude::AsRawHandle;
use std::{fmt, io};
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE};
use winapi::um::winnt;
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// How many process identifiers will be enumerated at most.
const MAX_PIDS: usize = 1024;

/// How many ASCII characters to read for a process name at most.
const MAX_PROC_NAME_LEN: usize = 64;

/// A handle to an opened process.
#[derive(Debug)]
pub struct Process {
    pid: u32,
    handle: OwnedHandle,
}

unsafe impl Send for Process {}

/// Enumerate the process identifiers of all programs currently running.
pub fn enum_proc() -> io::Result<Vec<u32>> {
    let mut size = 0;
    let mut pids = Vec::<DWORD>::with_capacity(MAX_PIDS);
    // SAFETY: the pointer is valid and the size matches the capacity.
    if unsafe {
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * mem::size_of::<DWORD>()) as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    let count = size as usize / mem::size_of::<DWORD>();
    // SAFETY: the call succeeded and count equals the right amount of items.
    unsafe { pids.set_len(count) };
    Ok(pids)
}

impl Process {
    /// Open a process handle given its process identifier.
    pub fn open(pid: u32) -> Result<Self, NullHandleError> {
        // SAFETY: the call doesn't have dangerous side-effects
        unsafe {
            HandleOrNull::from_raw_handle(winapi::um::processthreadsapi::OpenProcess(
                winnt::PROCESS_QUERY_INFORMATION
                    | winnt::PROCESS_VM_READ
                    | winnt::PROCESS_VM_WRITE
                    | winnt::PROCESS_VM_OPERATION,
                FALSE,
                pid,
            ))
        }
        .try_into()
        .map(|handle| Self { pid, handle })
    }

    /// Return the process identifier.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Return the base name of the first module loaded by this process.
    pub fn name(&self) -> io::Result<String> {
        let mut module = MaybeUninit::<HMODULE>::uninit();
        let mut size = 0;
        // SAFETY: the pointer is valid and the size is correct.
        if unsafe {
            winapi::um::psapi::EnumProcessModules(
                self.handle.as_raw_handle(),
                module.as_mut_ptr(),
                mem::size_of::<HMODULE>() as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: the call succeeded, so module is initialized.
        let module = unsafe { module.assume_init() };

        let mut buffer = Vec::<u8>::with_capacity(MAX_PROC_NAME_LEN);
        // SAFETY: the handle, module and buffer are all valid.
        let length = unsafe {
            winapi::um::psapi::GetModuleBaseNameA(
                self.handle.as_raw_handle(),
                module,
                buffer.as_mut_ptr().cast(),
                buffer.capacity() as u32,
            )
        };
        if length == 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: the call succeeded and length represents bytes.
        unsafe { buffer.set_len(length as usize) };
        Ok(String::from_utf8(buffer).unwrap())
    }

    pub fn memory_regions(&self) -> Vec<MEMORY_BASIC_INFORMATION> {
        let mut base = 0;
        let mut regions = Vec::new();
        let mut info = MaybeUninit::uninit();

        loop {
            // SAFETY: the info structure points to valid memory.
            let written = unsafe {
                winapi::um::memoryapi::VirtualQueryEx(
                    self.handle.as_raw_handle(),
                    base as *const _,
                    info.as_mut_ptr(),
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if written == 0 {
                break regions;
            }
            // SAFETY: a non-zero amount was written to the structure
            let info = unsafe { info.assume_init() };
            base = info.BaseAddress as usize + info.RegionSize;
            regions.push(info);
        }
    }

    pub fn read_memory(&self, addr: usize, n: usize) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::with_capacity(n);
        let mut read = 0;

        // SAFETY: the buffer points to valid memory, and the buffer size is correctly set.
        if unsafe {
            winapi::um::memoryapi::ReadProcessMemory(
                self.handle.as_raw_handle(),
                addr as *const _,
                buffer.as_mut_ptr().cast(),
                buffer.capacity(),
                &mut read,
            )
        } == FALSE
        {
            Err(io::Error::last_os_error())
        } else {
            // SAFETY: the call succeeded and `read` contains the amount of bytes written.
            unsafe { buffer.set_len(read as usize) };
            Ok(buffer)
        }
    }

    pub fn write_memory<const SIZE: usize, T: Scannable<SIZE>>(
        &self,
        addr: usize,
        value: &T,
    ) -> io::Result<usize> {
        let mut written = 0;

        // SAFETY: the input value buffer points to valid memory.
        if unsafe {
            winapi::um::memoryapi::WriteProcessMemory(
                self.handle.as_raw_handle(),
                addr as *mut _,
                (value as *const T).cast(),
                SIZE,
                &mut written,
            )
        } == FALSE
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(written)
        }
    }

    pub fn scan_regions<const SIZE: usize, T: Scannable<SIZE>>(
        &self,
        regions: &[MEMORY_BASIC_INFORMATION],
        scan: Scan<SIZE, T>,
    ) -> Vec<Region<SIZE, T>> {
        regions
            .iter()
            .flat_map(
                |region| match self.read_memory(region.BaseAddress as _, region.RegionSize) {
                    Ok(memory) => Some(scan.run(region.clone(), memory)),
                    Err(err) => {
                        warn!(
                            "Failed to read {} bytes at {:?}: {}",
                            region.RegionSize, region.BaseAddress, err,
                        );
                        None
                    }
                },
            )
            .filter(|region| region.locations.len() > 0)
            .collect()
    }

    pub fn rescan_regions<const SIZE: usize, T: Scannable<SIZE>>(
        &self,
        regions: &[Region<SIZE, T>],
        scan: Scan<SIZE, T>,
    ) -> Vec<Region<SIZE, T>> {
        regions
            .iter()
            .flat_map(|region| {
                match self.read_memory(region.info.BaseAddress as _, region.info.RegionSize) {
                    Ok(memory) => Some(scan.rerun(region, memory)),
                    Err(err) => {
                        warn!(
                            "Failed to read {} bytes at {:?}: {}",
                            region.info.RegionSize, region.info.BaseAddress, err,
                        );
                        None
                    }
                }
            })
            .filter(|region| region.locations.len() > 0)
            .collect()
    }
}

#[derive(Serialize)]
pub struct ProcessView {
    pub pid: u32,
    pub name: String,
}

impl fmt::Display for ProcessView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (pid={})", self.name, self.pid)
    }
}

impl TryFrom<&Process> for ProcessView {
    type Error = io::Error;

    fn try_from(value: &Process) -> Result<Self, Self::Error> {
        value.name().map(|name| ProcessView {
            pid: value.pid,
            name,
        })
    }
}
