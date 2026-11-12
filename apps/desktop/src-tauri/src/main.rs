// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows_support {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use windows_sys::core::BOOL;
    use windows_sys::Win32::Foundation::HINSTANCE;
    use windows_sys::Win32::System::Services::{
        CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatus,
        SC_MANAGER_CONNECT, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS, SERVICE_RUNNING,
        SERVICE_STATUS, SERVICE_STOP, SERVICE_STOPPED,
    };
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    use windows_sys::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    const NAHIMIC_SERVICE: &str = "NahimicService";
    const SAFE_LAUNCH_ARG: &str = "--nahimic-safe-launch";

    #[link(name = "kernel32")]
    extern "system" {
        fn SetProcessMitigationPolicy(
            mitigation_policy: i32,
            lp_buffer: *const c_void,
            dw_length: usize,
        ) -> i32;
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn set_process_mitigation_policy(mitigation_policy: i32, flags: u32) {
        #[repr(C)]
        struct PolicyFlags {
            flags: u32,
        }

        let policy = PolicyFlags { flags };
        unsafe {
            let _ = SetProcessMitigationPolicy(
                mitigation_policy,
                &policy as *const PolicyFlags as *const c_void,
                std::mem::size_of::<PolicyFlags>(),
            );
        }
    }

    fn harden_process() {
        // Disable legacy extension points, AppInit_DLLs and global hook injection.
        const PROCESS_EXTENSION_POINT_DISABLE_POLICY: i32 = 6;
        set_process_mitigation_policy(PROCESS_EXTENSION_POINT_DISABLE_POLICY, 1);
    }

    fn open_sc_manager() -> *mut c_void {
        unsafe { OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT) }
    }

    fn service_state() -> Option<u32> {
        let manager = open_sc_manager();
        if manager.is_null() {
            return None;
        }

        let name = wide(NAHIMIC_SERVICE);
        let service = unsafe { OpenServiceW(manager, name.as_ptr(), SERVICE_QUERY_STATUS) };
        unsafe {
            CloseServiceHandle(manager);
        }
        if service.is_null() {
            return None;
        }

        let mut status = SERVICE_STATUS::default();
        let queried = unsafe { QueryServiceStatus(service, &mut status) };
        unsafe {
            CloseServiceHandle(service);
        }
        if queried == 0 {
            None
        } else {
            Some(status.dwCurrentState)
        }
    }

    fn stop_service() {
        let manager = open_sc_manager();
        if manager.is_null() {
            return;
        }

        let name = wide(NAHIMIC_SERVICE);
        let service =
            unsafe { OpenServiceW(manager, name.as_ptr(), SERVICE_QUERY_STATUS | SERVICE_STOP) };
        if !service.is_null() {
            let mut status = SERVICE_STATUS::default();
            unsafe {
                ControlService(service, SERVICE_CONTROL_STOP, &mut status);
            }
            for _ in 0..20 {
                let mut current = SERVICE_STATUS::default();
                let queried = unsafe { QueryServiceStatus(service, &mut current) };
                if queried == 0 || current.dwCurrentState == SERVICE_STOPPED {
                    break;
                }
                thread::sleep(Duration::from_millis(250));
            }
            unsafe {
                CloseServiceHandle(service);
            }
        }
        unsafe {
            CloseServiceHandle(manager);
        }

        // Ensure the service helper processes are gone as well.
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "NahimicSvc64.exe", "/IM", "NahimicSvc32.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    fn schedule_service_restart() {
        // A detached helper restarts the service even if IngeCalc closes early.
        let _ = Command::new("cmd")
            .args([
                "/C",
                "timeout /t 20 /nobreak >nul & sc start NahimicService",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    fn relaunch_as_admin() -> bool {
        let Ok(executable) = std::env::current_exe() else {
            return false;
        };
        let executable = executable
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let operation = wide("runas");
        let parameters = wide(SAFE_LAUNCH_ARG);
        let result: HINSTANCE = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                executable.as_ptr(),
                parameters.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
        (result as isize) > 32
    }

    pub fn prepare_nahimic_launch() {
        if service_state() != Some(SERVICE_RUNNING) {
            return;
        }

        let safe_launch = std::env::args().any(|arg| arg == SAFE_LAUNCH_ARG);
        let is_admin: BOOL = unsafe { IsUserAnAdmin() };

        if !safe_launch && is_admin == 0 {
            if relaunch_as_admin() {
                std::process::exit(0);
            }
            // If UAC was cancelled, do not create a window that would crash.
            std::process::exit(1);
        }

        if is_admin == 0 {
            return;
        }

        stop_service();
        schedule_service_restart();
    }

    pub fn initialize() {
        harden_process();
        prepare_nahimic_launch();
    }
}

fn main() {
    #[cfg(windows)]
    windows_support::initialize();
    ingecalc_desktop_lib::run()
}
