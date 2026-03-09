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
    InvalidPath(String),
    InvalidProcessName(String),
}

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionError::ProcessNotFound(msg) => write!(f, "Process not found: {}", msg),
            InjectionError::OpenProcessFailed(msg) => write!(f, "Failed to open process: {}", msg),
            InjectionError::MemoryAllocationFailed(msg) => write!(f, "Failed to allocate memory: {}", msg),
            InjectionError::WriteMemoryFailed(msg) => write!(f, "Failed to write memory: {}", msg),
            InjectionError::CreateRemoteThreadFailed(msg) => write!(f, "Failed to create remote thread: {}", msg),
            InjectionError::InvalidPath(msg) => write!(f, "Invalid DLL path: {}", msg),
            InjectionError::InvalidProcessName(msg) => write!(f, "Invalid process name: {}", msg),
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
        
        // Create a snapshot of all running processes
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
                // Get information about the first process in the snapshot
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        // Find null terminator in process name and convert from UTF-16
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

                        // Move to the next process in the snapshot
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
        // Create a snapshot of all running processes
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
                // Get information about the first process in the snapshot
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        // Find null terminator and convert process name from UTF-16
                        let null_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);
                        let exe_name = exe_name.trim_end_matches('\0');
                        
                        // Check if this is the process we're looking for
                        if exe_name.eq_ignore_ascii_case(process_name) {
                            let _ = CloseHandle(snapshot);
                            return Ok(entry.th32ProcessID);
                        }

                        // Move to the next process in the snapshot
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
        if process_name.is_empty() {
            return Err(InjectionError::InvalidProcessName("Process name cannot be empty".to_string()));
        }
        
        if process_name.len() > 260 {
            return Err(InjectionError::InvalidProcessName("Process name too long (max 260 characters)".to_string()));
        }
        
        if !dll_path.exists() {
            return Err(InjectionError::InvalidPath("DLL file does not exist".to_string()));
        }
        
        if !dll_path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("dll")) {
            return Err(InjectionError::InvalidPath("File must be a .dll".to_string()));
        }
        
        let dll_path_str = dll_path.to_string_lossy().to_string();
        let dll_path_wide: Vec<u16> = dll_path_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let process_id = self.get_process_id(process_name)?;

        // Open the target process with all access rights
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
        // Allocate memory in the target process for the DLL path
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
            if let Err(e) = unsafe { CloseHandle(process_handle) } {
                eprintln!("Warning: Failed to close process handle: {:?}", e);
            }
            return Err(InjectionError::MemoryAllocationFailed(format!(
                "Error: {}", unsafe { GetLastError().0 }
            )));
        }

        // Write the DLL path to the allocated memory in the target process
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
                    if let Err(e) = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE) {
                        eprintln!("Warning: Failed to free memory: {:?}", e);
                    }
                    if let Err(e) = CloseHandle(process_handle) {
                        eprintln!("Warning: Failed to close process handle: {:?}", e);
                    }
                }
                return Err(InjectionError::WriteMemoryFailed(format!(
                    "Error: {}", unsafe { GetLastError().0 }
                )));
            }
        };

        // Get the address of LoadLibraryW function in kernel32.dll
        let load_library_addr: *const c_void = unsafe {
            match GetProcAddress(GetModuleHandleA(windows::core::s!("kernel32.dll")).unwrap(), windows::core::s!("LoadLibraryW")) {
                Some(addr) => addr as *const c_void,
                None => return Err(InjectionError::CreateRemoteThreadFailed(
                    "Failed to find LoadLibraryW".to_string()
                ))
            }
        };
        
        // Create a remote thread in the target process to load the DLL
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
                    if let Err(e) = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE) {
                        eprintln!("Warning: Failed to free memory: {:?}", e);
                    }
                    if let Err(e) = CloseHandle(process_handle) {
                        eprintln!("Warning: Failed to close process handle: {:?}", e);
                    }
                }
                return Err(InjectionError::CreateRemoteThreadFailed(format!(
                    "Error: {}", unsafe { GetLastError().0 }
                )));
            }
        };

        // Wait for the remote thread to complete and clean up
        unsafe {
            WaitForSingleObject(thread_handle, INFINITE);
            if let Err(e) = CloseHandle(thread_handle) {
                eprintln!("Warning: Failed to close thread handle: {:?}", e);
            }
            if let Err(e) = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE) {
                eprintln!("Warning: Failed to free memory: {:?}", e);
            }
            if let Err(e) = CloseHandle(process_handle) {
                eprintln!("Warning: Failed to close process handle: {:?}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_enumeration() {
        let injector = Injector::new();
        let processes = injector.get_all_processes();
        
        if processes.is_empty() {
            return;
        }
        
        for process in &processes {
            assert!(!process.name.is_empty(), "Process name should not be empty");
            if process.pid > 0 {
                return;
            }
        }
        panic!("No processes with valid PID found");
    }
    
    #[test]
    fn test_get_process_id_invalid() {
        let injector = Injector::new();
        let result = injector.get_process_id("NonExistentProcessName123456");
        
        assert!(result.is_err(), "Should return error for non-existent process");
        match result {
            Err(InjectionError::ProcessNotFound(_)) => (),
            _ => panic!("Should return ProcessNotFound error"),
        }
    }
    
    #[test]
    fn test_invalid_dll_path() {
        let injector = Injector::new();
        let result = injector.inject("notepad.exe", Path::new("nonexistent.dll"));
        
        assert!(result.is_err(), "Should return error for non-existent DLL");
        match result {
            Err(InjectionError::InvalidPath(_)) => (),
            _ => panic!("Should return InvalidPath error"),
        }
    }
    
    #[test]
    fn test_invalid_process_name_empty() {
        let injector = Injector::new();
        let result = injector.inject("", Path::new("test.dll"));
        
        assert!(result.is_err(), "Should return error for empty process name");
        match result {
            Err(InjectionError::InvalidProcessName(_)) => (),
            _ => panic!("Should return InvalidProcessName error"),
        }
    }
    
    #[test]
    fn test_invalid_process_name_too_long() {
        let injector = Injector::new();
        let long_name = "a".repeat(300);
        let result = injector.inject(&long_name, Path::new("test.dll"));
        
        assert!(result.is_err(), "Should return error for too long process name");
        match result {
            Err(InjectionError::InvalidProcessName(_)) => (),
            _ => panic!("Should return InvalidProcessName error"),
        }
    }
}
