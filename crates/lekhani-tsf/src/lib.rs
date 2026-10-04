//! Lekhani Text Services Framework (TSF) COM Input Method Engine
//! Provides native Windows 10 & 11 in-process Text Input Processor (TIP) integration
//! for true inline composing preedit, rich OpenType script display, and direct document range commits.

#[cfg(windows)]
mod com;
#[cfg(windows)]
mod factory;
pub mod guid;
#[cfg(windows)]
mod registration;
#[cfg(windows)]
mod tip;

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use std::sync::atomic::Ordering;
#[cfg(windows)]
use windows_sys::core::{GUID, HRESULT};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{BOOL, HMODULE};

#[cfg(windows)]
use com::{
    CLASS_E_NOAGGREGATION, E_NOINTERFACE, E_POINTER, IID_ICLASSFACTORY, IID_IUNKNOWN, S_FALSE, S_OK,
};
#[cfg(windows)]
use factory::{LekhaniClassFactory, SERVER_LOCK_COUNT};
#[cfg(windows)]
use guid::{guid_eq, CLSID_LEKHANI_TIP};

#[cfg(windows)]
static mut DLL_INSTANCE: HMODULE = 0 as _;

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst_dll: HMODULE,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if fdw_reason == DLL_PROCESS_ATTACH {
        DLL_INSTANCE = hinst_dll;
    }
    1
}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || rclsid.is_null() || riid.is_null() {
        return E_POINTER;
    }
    *ppv = std::ptr::null_mut();

    let clsid = &*rclsid;
    if !guid_eq(clsid, &CLSID_LEKHANI_TIP) {
        return CLASS_E_NOAGGREGATION;
    }

    let iid = &*riid;
    if !guid_eq(iid, &IID_IUNKNOWN) && !guid_eq(iid, &IID_ICLASSFACTORY) {
        return E_NOINTERFACE;
    }

    let factory = LekhaniClassFactory::new();
    *ppv = factory as *mut c_void;
    S_OK
}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if SERVER_LOCK_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    registration::register_server(DLL_INSTANCE as _)
}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    registration::unregister_server()
}

#[cfg(not(windows))]
pub fn tsf_info() -> &'static str {
    "Lekhani Windows Text Services Framework (TSF) TIP is designed for Windows 10/11."
}
