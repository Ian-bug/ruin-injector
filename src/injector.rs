use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use std::path::Path;
use std::ffi::c_void;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}

#[derive(Debug)]
pub enum InjectionError {
    ProcessNotFound(String),
    OpenProcessFailed(String),
    MemoryAllocationFailed(String),
    WriteMemoryFailed(String),
    CreateRemoteThreadFailed(String),
}

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionError::ProcessNotFound(msg) => write!(f, "Process not found: {}", msg),
            InjectionError::OpenProcessFailed(msg) => write!(f, "Failed to open process: {}", msg),
            InjectionError::MemoryAllocationFailed(msg) => write!(f, "Failed to allocate memory: {}", msg),
            InjectionError::WriteMemoryFailed(msg) => write!(f, "Failed to write memory: {}", msg),
            InjectionError::CreateRemoteThreadFailed(msg) => write!(f, "Failed to create remote thread: {}", msg),
        }
    }
}

impl std::error::Error for InjectionError {}

pub struct Injector;

impl Injector {
    pub fn new() -> Self {
        Injector
    }

    pub fn get_all_processes(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new();
        
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
        };

        if let Ok(snapshot) = snapshot {
            if snapshot.is_invalid() {
                return processes;
            }

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            unsafe {
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        let null_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos])
                            .trim_end_matches('\0')
                            .to_string();
                        
                        if !exe_name.is_empty() {
                            processes.push(ProcessInfo {
                                name: exe_name,
                                pid: entry.th32ProcessID,
                            });
                        }

                        if Process32NextW(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }
                let _ = CloseHandle(snapshot);
            }
        }

        processes
    }

    fn get_process_id(&self, process_name: &str) -> Result<u32, InjectionError> {
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
        };

        if let Ok(snapshot) = snapshot {
            if snapshot.is_invalid() {
                return Err(InjectionError::ProcessNotFound(
                    "Failed to create snapshot".to_string()
                ));
            }

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            unsafe {
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        let null_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);
                        let exe_name = exe_name.trim_end_matches('\0');
                        
                        if exe_name.eq_ignore_ascii_case(process_name) {
                            let _ = CloseHandle(snapshot);
                            return Ok(entry.th32ProcessID);
                        }

                        if Process32NextW(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }
                let _ = CloseHandle(snapshot);
            }
        } else {
            return Err(InjectionError::ProcessNotFound(
                "Failed to create snapshot".to_string()
            ));
        }

        Err(InjectionError::ProcessNotFound(format!(
            "Process '{}' not found", process_name
        )))
    }

    pub fn inject(&self, process_name: &str, dll_path: &Path) -> Result<(), InjectionError> {
        let dll_path_str = dll_path.to_string_lossy().to_string();
        let dll_path_wide: Vec<u16> = dll_path_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let process_id = self.get_process_id(process_name)?;

        let process_handle = unsafe {
            OpenProcess(
                PROCESS_ALL_ACCESS,
                FALSE,
                process_id
            )
        };

        let process_handle = match process_handle {
            Ok(handle) => handle,
            Err(_) => {
                return Err(InjectionError::OpenProcessFailed(format!(
                    "Error: {}, Process ID: {}",
                    unsafe { GetLastError().0 }, process_id
                )));
            }
        };

        let alloc_size = dll_path_wide.len() * std::mem::size_of::<u16>();
        let remote_buffer = unsafe {
            VirtualAllocEx(
                process_handle,
                None,
                alloc_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE
            )
        };

        if remote_buffer.is_null() {
            unsafe { CloseHandle(process_handle) };
            return Err(InjectionError::MemoryAllocationFailed(format!(
                "Error: {}", unsafe { GetLastError().0 }
            )));
        }

        let write_result = unsafe {
            WriteProcessMemory(
                process_handle,
                remote_buffer,
                dll_path_wide.as_ptr() as *const _,
                alloc_size,
                None
            )
        };

        let _ = match write_result {
            Ok(result) => result,
            Err(_) => {
                unsafe {
                    VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                    CloseHandle(process_handle);
                }
                return Err(InjectionError::WriteMemoryFailed(format!(
                    "Error: {}", unsafe { GetLastError().0 }
                )));
            }
        };

        let load_library_addr: *const c_void = unsafe {
            match GetProcAddress(GetModuleHandleA(windows::core::s!("kernel32.dll")).unwrap(), windows::core::s!("LoadLibraryW")) {
                Some(addr) => addr as *const c_void,
                None => return Err(InjectionError::CreateRemoteThreadFailed(
                    "Failed to find LoadLibraryW".to_string()
                ))
            }
        };
        
        let thread_handle = unsafe {
            CreateRemoteThread(
                process_handle,
                None,
                0,
                Some(std::mem::transmute::<*const c_void, extern "system" fn(*mut c_void) -> u32>(load_library_addr)),
                Some(remote_buffer),
                0,
                None
            )
        };

        let thread_handle = match thread_handle {
            Ok(handle) => handle,
            Err(_) => {
                unsafe {
                    VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                    CloseHandle(process_handle);
                }
                return Err(InjectionError::CreateRemoteThreadFailed(format!(
                    "Error: {}", unsafe { GetLastError().0 }
                )));
            }
        };

        unsafe {
            WaitForSingleObject(thread_handle, INFINITE);
            CloseHandle(thread_handle);
            VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
            CloseHandle(process_handle);
        }

        Ok(())
    }
}
