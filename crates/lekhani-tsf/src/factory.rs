//! COM Class Factory for Lekhani Text Input Processor

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::BOOL;

use crate::com::*;
use crate::guid::guid_eq;
use crate::tip::LekhaniTextInputProcessor;

pub static SERVER_LOCK_COUNT: AtomicI32 = AtomicI32::new(0);

#[repr(C)]
pub struct LekhaniClassFactory {
    pub lp_vtbl: *const IClassFactoryVtbl,
    pub ref_count: AtomicU32,
}

static FACTORY_VTBL: IClassFactoryVtbl = IClassFactoryVtbl {
    parent: IUnknownVtbl {
        QueryInterface: factory_query_interface,
        AddRef: factory_add_ref,
        Release: factory_release,
    },
    CreateInstance: factory_create_instance,
    LockServer: factory_lock_server,
};

impl LekhaniClassFactory {
    pub fn new() -> *mut Self {
        let obj = Box::new(Self {
            lp_vtbl: &FACTORY_VTBL,
            ref_count: AtomicU32::new(1),
        });
        Box::into_raw(obj)
    }
}

unsafe extern "system" fn factory_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || riid.is_null() {
        return E_POINTER;
    }
    let iid = &*riid;
    if guid_eq(iid, &IID_IUNKNOWN) || guid_eq(iid, &IID_ICLASSFACTORY) {
        *ppv = this;
        let p_this = this as *mut LekhaniClassFactory;
        (*p_this).ref_count.fetch_add(1, Ordering::SeqCst);
        S_OK
    } else {
        *ppv = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn factory_add_ref(this: *mut c_void) -> u32 {
    let p_this = this as *mut LekhaniClassFactory;
    (*p_this).ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn factory_release(this: *mut c_void) -> u32 {
    let p_this = this as *mut LekhaniClassFactory;
    let prev = (*p_this).ref_count.fetch_sub(1, Ordering::SeqCst);
    if prev == 1 {
        let _ = Box::from_raw(p_this);
        0
    } else {
        prev - 1
    }
}

unsafe extern "system" fn factory_create_instance(
    _this: *mut c_void,
    p_unk_outer: *mut c_void,
    riid: *const GUID,
    ppv_object: *mut *mut c_void,
) -> HRESULT {
    if ppv_object.is_null() || riid.is_null() {
        return E_POINTER;
    }
    *ppv_object = std::ptr::null_mut();

    if !p_unk_outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let processor = LekhaniTextInputProcessor::new();
    let hr = ((*processor)
        .lp_tip_ex_vtbl
        .as_ref()
        .unwrap()
        .parent
        .parent
        .QueryInterface)(processor as *mut c_void, riid, ppv_object);
    ((*processor)
        .lp_tip_ex_vtbl
        .as_ref()
        .unwrap()
        .parent
        .parent
        .Release)(processor as *mut c_void);
    hr
}

unsafe extern "system" fn factory_lock_server(_this: *mut c_void, f_lock: BOOL) -> HRESULT {
    if f_lock != 0 {
        SERVER_LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
    } else {
        SERVER_LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
    S_OK
}
