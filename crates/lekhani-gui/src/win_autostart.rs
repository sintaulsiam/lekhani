//! Windows Autostart Registry Helper
//! Manages automatic startup on Windows login via HKCU Run registry key.

#![cfg(windows)]

use std::ptr;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

pub fn is_autostart_enabled() -> bool {
    unsafe {
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "Lekhani\0".encode_utf16().collect();
        let mut hkey: HKEY = ptr::null_mut();

        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        ) == ERROR_SUCCESS
        {
            let mut data_type = 0;
            let mut data_len = 0;
            let result = RegQueryValueExW(
                hkey,
                value_name.as_ptr(),
                ptr::null_mut(),
                &mut data_type,
                ptr::null_mut(),
                &mut data_len,
            );
            RegCloseKey(hkey);
            result == ERROR_SUCCESS && data_len > 0
        } else {
            false
        }
    }
}

pub fn set_autostart(enable: bool) -> bool {
    unsafe {
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "Lekhani\0".encode_utf16().collect();
        let mut hkey: HKEY = ptr::null_mut();

        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_WRITE,
            &mut hkey,
        ) == ERROR_SUCCESS
        {
            let res = if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = format!("\"{}\"\0", exe_path.to_string_lossy());
                    let path_utf16: Vec<u16> = path_str.encode_utf16().collect();
                    RegSetValueExW(
                        hkey,
                        value_name.as_ptr(),
                        0,
                        REG_SZ,
                        path_utf16.as_ptr() as *const u8,
                        (path_utf16.len() * 2) as u32,
                    ) == ERROR_SUCCESS
                } else {
                    false
                }
            } else {
                let _ = RegDeleteValueW(hkey, value_name.as_ptr());
                true
            };
            RegCloseKey(hkey);
            res
        } else {
            false
        }
    }
}
