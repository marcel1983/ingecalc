// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
fn disable_extension_point_injection() {
    const PROCESS_EXTENSION_POINT_DISABLE_POLICY: i32 = 6;

    #[repr(C)]
    struct ExtensionPointDisablePolicy {
        flags: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn SetProcessMitigationPolicy(
            mitigation_policy: i32,
            lp_buffer: *const std::ffi::c_void,
            dw_length: usize,
        ) -> i32;
    }

    let policy = ExtensionPointDisablePolicy { flags: 1 };
    unsafe {
        let _ = SetProcessMitigationPolicy(
            PROCESS_EXTENSION_POINT_DISABLE_POLICY,
            &policy as *const ExtensionPointDisablePolicy as *const std::ffi::c_void,
            std::mem::size_of::<ExtensionPointDisablePolicy>(),
        );
    }
}

fn main() {
    #[cfg(windows)]
    disable_extension_point_injection();
    ingecalc_desktop_lib::run()
}
