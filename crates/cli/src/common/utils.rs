/// Expands a path string, resolving home directory references (~/)
pub fn expand_path(path: &str) -> String {
    let path = path.trim();

    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.strip_prefix("~/").unwrap()).to_string_lossy().into_owned();
        }
    }

    path.to_string()
}

/// Checks if a process with the given PID is running
#[cfg(unix)]
pub fn check_process_running(pid: u32) -> bool {
    use std::process::Command;

    // Different command for different Unix variants
    #[cfg(target_os = "macos")]
    let output = Command::new("ps").args(["-p", &pid.to_string()]).output();

    #[cfg(all(unix, not(target_os = "macos")))]
    let output = Command::new("ps").args(&["-p", &pid.to_string()]).output();

    match output {
        Ok(output) => {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

#[cfg(windows)]
pub fn check_process_running(pid: u32) -> bool {
    // For Windows, we'd need to use the Windows API
    // This is a simplified implementation
    type DWORD = u32;
    type HANDLE = *mut std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: i32, dwProcessId: DWORD) -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> i32;
    }

    const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        let exists = !handle.is_null();

        if !handle.is_null() {
            CloseHandle(handle);
        }

        exists
    }
}

#[allow(clashing_extern_declarations)]
/// Gets the current memory usage of the process in kilobytes
pub fn get_process_memory_usage() -> u64 {
    #[cfg(target_os = "linux")]
    {
        // Linux implementation - read from /proc/self/status
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        match File::open("/proc/self/status") {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // VmRSS gives the resident set size (physical memory used)
                        if line.starts_with("VmRSS:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                if let Ok(kb) = parts[1].parse::<u64>() {
                                    // Already in KB on Linux
                                    return kb;
                                }
                            }
                        }
                    }
                }
                0
            }
            Err(_) => 0,
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::mem::size_of;

        type MachPort = u32;

        #[repr(C)]
        struct MachTaskBasicInfo {
            virtual_size: u64,
            resident_size: u64,
            resident_size_max: u64,
            user_time: u64,
            system_time: u64,
            policy: i32,
            suspend_count: i32,
        }

        const MACH_TASK_BASIC_INFO_COUNT: u32 =
            (size_of::<MachTaskBasicInfo>() / size_of::<u32>()) as u32;
        const MACH_TASK_BASIC_INFO: u32 = 20;

        extern "C" {
            fn mach_task_self() -> MachPort;
            fn task_info(
                task: MachPort,
                flavor: u32,
                task_info_out: *mut MachTaskBasicInfo,
                task_info_outCnt: *mut u32,
            ) -> i32;
        }

        let mut info = MachTaskBasicInfo {
            virtual_size: 0,
            resident_size: 0,
            resident_size_max: 0,
            user_time: 0,
            system_time: 0,
            policy: 0,
            suspend_count: 0,
        };

        let mut count = MACH_TASK_BASIC_INFO_COUNT;

        unsafe {
            let kr =
                task_info(mach_task_self(), MACH_TASK_BASIC_INFO, &mut info as *mut _, &mut count);

            if kr == 0 {
                // Convert from bytes to kilobytes
                return info.resident_size / 1024;
            }
        }

        0
    }

    #[cfg(target_os = "windows")]
    {
        use std::mem::{size_of, zeroed};

        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        type HANDLE = *mut std::ffi::c_void;

        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> HANDLE;
        }

        #[link(name = "psapi")]
        extern "system" {
            fn GetProcessMemoryInfo(
                Process: HANDLE,
                ppsmemCounters: *mut ProcessMemoryCounters,
                cb: u32,
            ) -> i32;
        }

        unsafe {
            let mut pmc: ProcessMemoryCounters = zeroed();
            pmc.cb = size_of::<ProcessMemoryCounters>() as u32;

            if GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut pmc,
                size_of::<ProcessMemoryCounters>() as u32,
            ) != 0
            {
                // Convert from bytes to kilobytes
                return (pmc.working_set_size / 1024) as u64;
            }
        }

        0
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        0
    }
}
