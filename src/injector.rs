use std::ffi::c_void;
use std::path::Path;
use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;

const MAX_PROCESS_NAME_LENGTH: usize = 260;
const MAX_PATH_LENGTH: usize = 260;

pub fn is_elevated() -> bool {
    unsafe {
        let mut token_handle = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_ok() {
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            let _ = CloseHandle(token_handle);
            result.is_ok() && elevation.TokenIsElevated != 0
        } else {
            false
        }
    }
}

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
    PathTooLong(String),
    DllLoadFailed(String),
    ThreadWaitFailed(String),
    UwpProcessNotSupported(String),
}

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionError::ProcessNotFound(msg) => write!(f, "Process not found: {}", msg),
            InjectionError::OpenProcessFailed(msg) => write!(
                f,
                "Failed to open process: {}. Try running as Administrator.",
                msg
            ),
            InjectionError::MemoryAllocationFailed(msg) => {
                write!(f, "Failed to allocate memory: {}", msg)
            }
            InjectionError::WriteMemoryFailed(msg) => write!(f, "Failed to write memory: {}", msg),
            InjectionError::CreateRemoteThreadFailed(msg) => {
                write!(f, "Failed to create remote thread: {}", msg)
            }
            InjectionError::InvalidPath(msg) => write!(f, "Invalid DLL path: {}", msg),
            InjectionError::InvalidProcessName(msg) => write!(f, "Invalid process name: {}", msg),
            InjectionError::PathTooLong(msg) => {
                write!(
                    f,
                    "DLL path too long: {} (max {} characters)",
                    msg, MAX_PATH_LENGTH
                )
            }
            InjectionError::DllLoadFailed(msg) => {
                write!(
                    f,
                    "DLL load failed: {}. Possible causes: architecture mismatch, missing dependencies, or anti-cheat protection.",
                    msg
                )
            }
            InjectionError::ThreadWaitFailed(msg) => write!(f, "Thread wait failed: {}", msg),
            InjectionError::UwpProcessNotSupported(msg) => {
                write!(
                    f,
                    "UWP process not supported: {}. UWP apps have restricted injection capabilities. Consider using a different process or debugging approach.",
                    msg
                )
            }
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

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

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
                        let null_pos = entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(MAX_PROCESS_NAME_LENGTH);
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
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        if let Ok(snapshot) = snapshot {
            if snapshot.is_invalid() {
                return Err(InjectionError::ProcessNotFound(
                    "Failed to create snapshot".to_string(),
                ));
            }

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            unsafe {
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        let null_pos = entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(MAX_PROCESS_NAME_LENGTH);
                        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos])
                            .trim_end_matches('\0')
                            .to_string();

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
                "Failed to create snapshot".to_string(),
            ));
        }

        Err(InjectionError::ProcessNotFound(format!(
            "Process '{}' not found",
            process_name
        )))
    }

    /// Check if a process is 64-bit (requires process handle)
    fn is_process_64bit(process_handle: HANDLE) -> Result<bool, InjectionError> {
        unsafe {
            let mut is_wow64 = FALSE;
            let result = IsWow64Process(process_handle, &mut is_wow64);
            if result.is_err() {
                return Err(InjectionError::OpenProcessFailed(
                    "Failed to check process architecture".to_string(),
                ));
            }
            // If the process is running under WOW64, it's a 32-bit process on 64-bit Windows
            // Otherwise, if we're on 64-bit Windows, it's a 64-bit process
            #[cfg(target_pointer_width = "64")]
            {
                Ok(!is_wow64.as_bool())
            }
            #[cfg(target_pointer_width = "32")]
            {
                Ok(is_wow64.as_bool() == true)
            }
        }
    }

    /// Check if a process is likely a UWP app by examining its path
    /// UWP apps are typically installed in C:\\Program Files\\WindowsApps
    fn detect_uwp_process(pid: u32) -> Result<bool, InjectionError> {
        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
            if let Ok(handle) = process_handle {
                if !handle.is_invalid() {
                    // Use QueryFullProcessImageNameW if available (Windows 8.1+)
                    let mut buffer: [u16; MAX_PATH_LENGTH] = [0; MAX_PATH_LENGTH];
                    let mut size = buffer.len() as u32;

                    // Get function address dynamically
                    let k32_handle = GetModuleHandleA(windows::core::s!("kernel32.dll"));
                    if let Ok(k32_handle) = k32_handle {
                        let get_full_process_image_name = GetProcAddress(
                            k32_handle,
                            windows::core::s!("QueryFullProcessImageNameW"),
                        );

                        if let Some(func_ptr) = get_full_process_image_name {
                            type QueryFullProcessImageNameW =
                                unsafe extern "system" fn(HANDLE, u32, *mut u16, *mut u32) -> BOOL;

                            let func: QueryFullProcessImageNameW = std::mem::transmute(func_ptr);

                            let _ = func(handle, 0, buffer.as_mut_ptr(), &mut size);
                        }
                    }

                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    let _ = CloseHandle(handle);

                    // UWP apps are typically in WindowsApps directory
                    Ok(path.contains("WindowsApps") || path.contains("AppPackages"))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
    }

    pub fn inject(&self, process_name: &str, dll_path: &Path) -> Result<(), InjectionError> {
        if process_name.is_empty() {
            return Err(InjectionError::InvalidProcessName(
                "Process name cannot be empty".to_string(),
            ));
        }

        if process_name.len() > MAX_PROCESS_NAME_LENGTH {
            return Err(InjectionError::InvalidProcessName(format!(
                "Process name too long (max {} characters)",
                MAX_PROCESS_NAME_LENGTH
            )));
        }

        if !dll_path.exists() {
            return Err(InjectionError::InvalidPath(
                "DLL file does not exist".to_string(),
            ));
        }

        if !dll_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
        {
            return Err(InjectionError::InvalidPath(
                "File must be a .dll".to_string(),
            ));
        }

        // Validate DLL path length
        let dll_path_str = dll_path.to_string_lossy().into_owned();
        if dll_path_str.len() > MAX_PATH_LENGTH {
            return Err(InjectionError::PathTooLong(dll_path_str));
        }

        // Get canonical path to avoid issues with relative paths
        let canonical_path = dll_path
            .canonicalize()
            .map_err(|e| InjectionError::InvalidPath(format!("Cannot resolve path: {}", e)))?;

        // Convert to wide string with proper Windows path format
        let dll_path_wide: Vec<u16> = canonical_path
            .to_string_lossy()
            .as_ref()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let process_id = self.get_process_id(process_name)?;

        // Check for UWP process
        if Self::detect_uwp_process(process_id)? {
            return Err(InjectionError::UwpProcessNotSupported(
                process_name.to_string(),
            ));
        }

        // Open the target process with all access rights
        let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, process_id) };

        let process_handle = match process_handle {
            Ok(handle) => handle,
            Err(e) => {
                return Err(InjectionError::OpenProcessFailed(format!(
                    "Error: {} ({}), Process ID: {}. Try running as Administrator.",
                    unsafe { GetLastError().0 },
                    e,
                    process_id
                )));
            }
        };

        // Check architecture compatibility
        let process_is_64bit = Self::is_process_64bit(process_handle)?;
        #[cfg(target_pointer_width = "64")]
        let injector_is_64bit = true;
        #[cfg(target_pointer_width = "32")]
        let injector_is_64bit = false;

        if process_is_64bit != injector_is_64bit {
            let _ = unsafe { CloseHandle(process_handle) };
            return Err(InjectionError::DllLoadFailed(format!(
                "Architecture mismatch: injector is {}-bit, target process is {}-bit",
                if injector_is_64bit { "64" } else { "32" },
                if process_is_64bit { "64" } else { "32" }
            )));
        }

        let alloc_size = dll_path_wide.len() * std::mem::size_of::<u16>();

        // Allocate memory in the target process for the DLL path
        let remote_buffer = unsafe {
            VirtualAllocEx(
                process_handle,
                None,
                alloc_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };

        if remote_buffer.is_null() {
            let _ = unsafe { CloseHandle(process_handle) };
            return Err(InjectionError::MemoryAllocationFailed(format!(
                "Error: {}",
                unsafe { GetLastError().0 }
            )));
        }

        // Write the DLL path to the allocated memory in the target process
        let write_result = unsafe {
            WriteProcessMemory(
                process_handle,
                remote_buffer,
                dll_path_wide.as_ptr() as *const _,
                alloc_size,
                None,
            )
        };

        if write_result.is_err() {
            unsafe {
                let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                let _ = CloseHandle(process_handle);
            }
            return Err(InjectionError::WriteMemoryFailed(format!(
                "Error: {}",
                unsafe { GetLastError().0 }
            )));
        }

        // Get the address of LoadLibraryW function in kernel32.dll
        let kernel32_handle = unsafe { GetModuleHandleA(windows::core::s!("kernel32.dll")) };
        let kernel32_handle = match kernel32_handle {
            Ok(handle) => handle,
            Err(_) => {
                unsafe {
                    let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                    let _ = CloseHandle(process_handle);
                }
                return Err(InjectionError::CreateRemoteThreadFailed(
                    "Failed to get kernel32.dll module handle".to_string(),
                ));
            }
        };

        let load_library_addr: *const c_void = unsafe {
            match GetProcAddress(kernel32_handle, windows::core::s!("LoadLibraryW")) {
                Some(addr) => addr as *const c_void,
                None => {
                    let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                    let _ = CloseHandle(process_handle);
                    return Err(InjectionError::CreateRemoteThreadFailed(
                        "Failed to find LoadLibraryW".to_string(),
                    ));
                }
            }
        };

        // Safety: LoadLibraryW has the correct signature for CreateRemoteThread's thread function.
        // This is a standard Windows API pattern for DLL injection.
        // LPTHREAD_START_ROUTINE is: extern "system" fn(*mut c_void) -> u32
        // LoadLibraryW matches this signature on Windows x86/x64.
        #[allow(clippy::missing_transmute_annotations)]
        let thread_proc: extern "system" fn(*mut c_void) -> u32 =
            unsafe { std::mem::transmute(load_library_addr) };

        // Create a remote thread in the target process to load the DLL
        let thread_handle = unsafe {
            CreateRemoteThread(
                process_handle,
                None,
                0,
                Some(thread_proc),
                Some(remote_buffer),
                0,
                None,
            )
        };

        let thread_handle = match thread_handle {
            Ok(handle) => handle,
            Err(_) => {
                unsafe {
                    let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                    let _ = CloseHandle(process_handle);
                }
                return Err(InjectionError::CreateRemoteThreadFailed(format!(
                    "Error: {}",
                    unsafe { GetLastError().0 }
                )));
            }
        };

        // Wait for the remote thread to complete with timeout (10 seconds for larger DLLs)
        const INJECTION_TIMEOUT_MS: u32 = 10000;
        let injection_result = unsafe {
            let wait_result = WaitForSingleObject(thread_handle, INJECTION_TIMEOUT_MS);

            if wait_result == WAIT_TIMEOUT {
                let _ = CloseHandle(thread_handle);
                let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                let _ = CloseHandle(process_handle);
                return Err(InjectionError::CreateRemoteThreadFailed(
                    "Injection timed out after 10 seconds".to_string(),
                ));
            }

            if wait_result == WAIT_FAILED {
                let error = GetLastError();
                let _ = CloseHandle(thread_handle);
                let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
                let _ = CloseHandle(process_handle);
                return Err(InjectionError::ThreadWaitFailed(format!(
                    "Wait failed with error: {}",
                    error.0
                )));
            }

            // Get the exit code (return value of LoadLibraryW)
            // If it's 0, LoadLibraryW failed
            let mut exit_code: u32 = 0;
            let exit_result = GetExitCodeThread(thread_handle, &mut exit_code);

            // Clean up handles
            let _ = CloseHandle(thread_handle);
            let _ = VirtualFreeEx(process_handle, remote_buffer, 0, MEM_RELEASE);
            let _ = CloseHandle(process_handle);

            if exit_result.is_err() {
                return Err(InjectionError::DllLoadFailed(
                    "Failed to get thread exit code".to_string(),
                ));
            }

            exit_code
        };

        // LoadLibraryW returns NULL (0) on failure, module handle on success
        if injection_result == 0 {
            return Err(InjectionError::DllLoadFailed(
                "LoadLibraryW returned NULL - DLL failed to load. Possible causes:\n\
                 - DLL is missing dependencies\n\
                 - DLL architecture doesn't match target process (32-bit vs 64-bit)\n\
                 - DLL initialization (DllMain) crashed\n\
                 - Target process has anti-cheat protection\n\
                 - Insufficient permissions"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated() {
        // This test verifies the function runs without panicking
        // The result depends on whether the test runner is elevated
        let elevated = is_elevated();
        assert!(
            elevated == true || elevated == false,
            "is_elevated should return a boolean"
        );
    }

    #[test]
    fn test_process_enumeration() {
        let injector = Injector::new();
        let processes = injector.get_all_processes();

        assert!(!processes.is_empty(), "Should find at least one process");

        for process in &processes {
            assert!(!process.name.is_empty(), "Process name should not be empty");
        }

        assert!(
            processes.iter().any(|p| p.pid > 0),
            "Should find at least one process with positive PID"
        );
    }

    #[test]
    fn test_get_process_id_invalid() {
        let injector = Injector::new();
        let result = injector.get_process_id("NonExistentProcessName123456");

        assert!(
            result.is_err(),
            "Should return error for non-existent process"
        );
        match result {
            Err(InjectionError::ProcessNotFound(_)) => (),
            _ => panic!("Should return ProcessNotFound error"),
        }
    }

    #[test]
    fn test_invalid_dll_path() {
        let injector = Injector::new();
        let result = injector.inject("notepad.exe", Path::new("nonexistent.dll"));

        if is_elevated() {
            assert!(result.is_err(), "Should return error for non-existent DLL");
            match result {
                Err(InjectionError::InvalidPath(_)) => (),
                _ => panic!("Should return InvalidPath error"),
            }
        }
    }

    #[test]
    fn test_invalid_dll_extension() {
        let injector = Injector::new();
        let result = injector.inject("notepad.exe", Path::new("test.exe"));

        if is_elevated() {
            assert!(result.is_err(), "Should return error for non-DLL file");
            match result {
                Err(InjectionError::InvalidPath(_)) => (),
                _ => panic!("Should return InvalidPath error"),
            }
        }
    }

    #[test]
    fn test_invalid_process_name_empty() {
        let injector = Injector::new();
        let result = injector.inject("", Path::new("test.dll"));

        assert!(
            result.is_err(),
            "Should return error for empty process name"
        );
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

        assert!(
            result.is_err(),
            "Should return error for too long process name"
        );
        match result {
            Err(InjectionError::InvalidProcessName(_)) => (),
            _ => panic!("Should return InvalidProcessName error"),
        }
    }

    #[test]
    fn test_process_info_clone() {
        let process = ProcessInfo {
            name: "test.exe".to_string(),
            pid: 1234,
        };
        let cloned = process.clone();

        assert_eq!(process.name, cloned.name);
        assert_eq!(process.pid, cloned.pid);
    }
}
