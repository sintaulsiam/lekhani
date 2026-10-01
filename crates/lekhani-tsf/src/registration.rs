//! COM & Windows Text Services Framework (TSF) Registration Helper

#![cfg(windows)]

use std::ptr;
use windows_sys::core::HRESULT;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteKeyW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT, REG_SZ,
};

use crate::com::*;

pub unsafe fn register_server(hmodule: *mut std::ffi::c_void) -> HRESULT {
    let mut module_path = [0u16; 1024];
    let len = GetModuleFileNameW(hmodule as _, module_path.as_mut_ptr(), 1024);
    if len == 0 {
        return E_FAIL;
    }

    // Register COM InprocServer32 in HKCR\CLSID\{7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}
    let clsid_key = "CLSID\\{7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}\0";
    let inproc_key = "CLSID\\{7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}\\InprocServer32\0";
    let desc = "Lekhani Bengali Text Input Processor\0";
    let threading = "Apartment\0";

    if !create_reg_key_value(HKEY_CLASSES_ROOT, clsid_key, "", desc) {
        return E_FAIL;
    }
    if !create_reg_key_value(
        HKEY_CLASSES_ROOT,
        inproc_key,
        "",
        &String::from_utf16_lossy(&module_path[..len as usize]),
    ) {
        return E_FAIL;
    }
    if !create_reg_key_value(HKEY_CLASSES_ROOT, inproc_key, "ThreadingModel", threading) {
        return E_FAIL;
    }

    tracing::info!("Registered Lekhani TSF InprocServer32 in Windows Registry");
    S_OK
}

pub unsafe fn unregister_server() -> HRESULT {
    let inproc_key = "CLSID\\{7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}\\InprocServer32\0";
    let inproc_utf16: Vec<u16> = inproc_key.encode_utf16().collect();
    let clsid_key = "CLSID\\{7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}\0";
    let clsid_utf16: Vec<u16> = clsid_key.encode_utf16().collect();

    let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, inproc_utf16.as_ptr());
    let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, clsid_utf16.as_ptr());
    tracing::info!("Unregistered Lekhani TSF from Windows Registry");
    S_OK
}

unsafe fn create_reg_key_value(root: HKEY, subkey: &str, value_name: &str, data: &str) -> bool {
    let subkey_utf16: Vec<u16> = subkey.encode_utf16().collect();
    let val_name_utf16: Vec<u16> = if value_name.is_empty() {
        vec![0]
    } else {
        format!("{}\0", value_name).encode_utf16().collect()
    };
    let data_utf16: Vec<u16> = format!("{}\0", data).encode_utf16().collect();

    let mut hkey: HKEY = ptr::null_mut();
    if RegCreateKeyW(root, subkey_utf16.as_ptr(), &mut hkey) != ERROR_SUCCESS {
        return false;
    }

    let res = RegSetValueExW(
        hkey,
        if value_name.is_empty() {
            ptr::null()
        } else {
            val_name_utf16.as_ptr()
        },
        0,
        REG_SZ,
        data_utf16.as_ptr() as *const u8,
        (data_utf16.len() * 2) as u32,
    );
    RegCloseKey(hkey);
    res == ERROR_SUCCESS
}
