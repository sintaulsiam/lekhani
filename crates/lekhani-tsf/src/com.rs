//! Text Services Framework (TSF) COM Interface Vtables and Definitions

#![cfg(windows)]
#![allow(non_snake_case, dead_code)]

use std::ffi::c_void;
use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{BOOL, LPARAM, WPARAM};

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_FAIL: HRESULT = 0x80004005_u32 as i32;
pub const E_NOINTERFACE: HRESULT = 0x80004002_u32 as i32;
pub const E_POINTER: HRESULT = 0x80004003_u32 as i32;
pub const E_INVALIDARG: HRESULT = 0x80070057_u32 as i32;
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110_u32 as i32;

// Standard COM IIDs
pub const IID_IUNKNOWN: GUID = GUID::from_u128(0x00000000_0000_0000_c000_000000000046);
pub const IID_ICLASSFACTORY: GUID = GUID::from_u128(0x00000001_0000_0000_c000_000000000046);

// TSF Interface IIDs
pub const IID_ITFTEXTINPUTPROCESSOR: GUID = GUID::from_u128(0xaa80e7f7_2021_11d2_93e0_0060b067b86e);
pub const IID_ITFTEXTINPUTPROCESSOREX: GUID =
    GUID::from_u128(0xa99b7b7e_0c8d_44db_8892_1084227f2c20);
pub const IID_ITFKEYEVENTSINK: GUID = GUID::from_u128(0x3d61bf11_acff_428f_a87f_9007357d6457);
pub const IID_ITFCOMPOSITIONSINK: GUID = GUID::from_u128(0xd7540241_f9a1_4364_befc_dbcd2c4395b7);
pub const IID_ITFTHREADMGREVENTSINK: GUID = GUID::from_u128(0x08466369_00e2_43dd_b43d_5382920b22a7);
pub const IID_ITFTHREADMGR: GUID = GUID::from_u128(0xaa80e7f0_2021_11d2_93e0_0060b067b86e);
pub const IID_ITFKEYSTROKEMGR: GUID = GUID::from_u128(0xaaa274aa_2020_11d2_93e0_0060b067b86e);
pub const IID_ITFCONTEXT: GUID = GUID::from_u128(0xaa80e7fd_2021_11d2_93e0_0060b067b86e);
pub const IID_ITFEDITSESSION: GUID = GUID::from_u128(0xaa80e803_2021_11d2_93e0_0060b067b86e);
pub const IID_ITFRANGE: GUID = GUID::from_u128(0xaa80e7ff_2021_11d2_93e0_0060b067b86e);
pub const IID_ITFCOMPOSITION: GUID = GUID::from_u128(0x20168d11_5a76_4a5b_a61b_2577314b55e3);
pub const IID_ITFINSERTATSELECTION: GUID = GUID::from_u128(0x55ce4538_a6f4_41b1_a753_367322c6db55);

pub type TfClientId = u32;
pub type TfEditCookie = u32;

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
}

#[repr(C)]
pub struct IClassFactoryVtbl {
    pub parent: IUnknownVtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub LockServer: unsafe extern "system" fn(*mut c_void, BOOL) -> HRESULT,
}

#[repr(C)]
pub struct ITfTextInputProcessorVtbl {
    pub parent: IUnknownVtbl,
    pub Activate: unsafe extern "system" fn(*mut c_void, *mut c_void, TfClientId) -> HRESULT,
    pub Deactivate: unsafe extern "system" fn(*mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct ITfTextInputProcessorExVtbl {
    pub parent: ITfTextInputProcessorVtbl,
    pub ActivateEx: unsafe extern "system" fn(*mut c_void, *mut c_void, TfClientId, u32) -> HRESULT,
}

#[repr(C)]
pub struct ITfKeyEventSinkVtbl {
    pub parent: IUnknownVtbl,
    pub OnSetFocus: unsafe extern "system" fn(*mut c_void, BOOL) -> HRESULT,
    pub OnTestKeyDown:
        unsafe extern "system" fn(*mut c_void, *mut c_void, WPARAM, LPARAM, *mut BOOL) -> HRESULT,
    pub OnKeyDown:
        unsafe extern "system" fn(*mut c_void, *mut c_void, WPARAM, LPARAM, *mut BOOL) -> HRESULT,
    pub OnTestKeyUp:
        unsafe extern "system" fn(*mut c_void, *mut c_void, WPARAM, LPARAM, *mut BOOL) -> HRESULT,
    pub OnKeyUp:
        unsafe extern "system" fn(*mut c_void, *mut c_void, WPARAM, LPARAM, *mut BOOL) -> HRESULT,
    pub OnPreservedKey:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *const GUID, *mut BOOL) -> HRESULT,
}

#[repr(C)]
pub struct ITfCompositionSinkVtbl {
    pub parent: IUnknownVtbl,
    pub OnCompositionTerminated:
        unsafe extern "system" fn(*mut c_void, TfEditCookie, *mut c_void) -> HRESULT,
}
