//! Lekhani Text Input Processor (TIP) Core COM Implementation

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_F12, VK_SPACE};

use crate::com::*;
use crate::guid::guid_eq;
use lekhani_core::{InputSession, KeycodeMapper};
use lekhani_settings::ConfigManager;

#[repr(C)]
pub struct LekhaniTextInputProcessor {
    pub lp_tip_ex_vtbl: *const ITfTextInputProcessorExVtbl,
    pub lp_key_sink_vtbl: *const ITfKeyEventSinkVtbl,
    pub lp_comp_sink_vtbl: *const ITfCompositionSinkVtbl,
    pub ref_count: AtomicU32,
    pub inner: Mutex<TipInner>,
}

pub struct TipInner {
    pub thread_mgr: *mut c_void,
    pub client_id: TfClientId,
    pub session: InputSession,
    pub mapper: KeycodeMapper,
    pub bengali_active: bool,
    pub active_composition: *mut c_void,
}

impl LekhaniTextInputProcessor {
    pub fn new() -> *mut Self {
        let config_mgr = ConfigManager::new();
        let mut session = InputSession::new();
        let system_data = ConfigManager::get_system_data_dir();
        session.load_database(&system_data);
        session.load_user_autocorrect(config_mgr.get_user_autocorrect_path());
        session.load_user_learned(config_mgr.get_user_learned_path());
        config_mgr.config.apply_to_session(&mut session);

        let inner = TipInner {
            thread_mgr: std::ptr::null_mut(),
            client_id: 0,
            session,
            mapper: KeycodeMapper::new(),
            bengali_active: true,
            active_composition: std::ptr::null_mut(),
        };

        let obj = Box::new(Self {
            lp_tip_ex_vtbl: &TIP_EX_VTBL,
            lp_key_sink_vtbl: &KEY_SINK_VTBL,
            lp_comp_sink_vtbl: &COMP_SINK_VTBL,
            ref_count: AtomicU32::new(1),
            inner: Mutex::new(inner),
        });

        Box::into_raw(obj)
    }
}

// ── Vtables ──────────────────────────────────────────────────────────────────

static TIP_EX_VTBL: ITfTextInputProcessorExVtbl = ITfTextInputProcessorExVtbl {
    parent: ITfTextInputProcessorVtbl {
        parent: IUnknownVtbl {
            QueryInterface: tip_query_interface,
            AddRef: tip_add_ref,
            Release: tip_release,
        },
        Activate: tip_activate,
        Deactivate: tip_deactivate,
    },
    ActivateEx: tip_activate_ex,
};

static KEY_SINK_VTBL: ITfKeyEventSinkVtbl = ITfKeyEventSinkVtbl {
    parent: IUnknownVtbl {
        QueryInterface: key_sink_query_interface,
        AddRef: key_sink_add_ref,
        Release: key_sink_release,
    },
    OnSetFocus: key_sink_on_set_focus,
    OnTestKeyDown: key_sink_on_test_key_down,
    OnKeyDown: key_sink_on_key_down,
    OnTestKeyUp: key_sink_on_test_key_up,
    OnKeyUp: key_sink_on_key_up,
    OnPreservedKey: key_sink_on_preserved_key,
};

static COMP_SINK_VTBL: ITfCompositionSinkVtbl = ITfCompositionSinkVtbl {
    parent: IUnknownVtbl {
        QueryInterface: comp_sink_query_interface,
        AddRef: comp_sink_add_ref,
        Release: comp_sink_release,
    },
    OnCompositionTerminated: comp_sink_on_composition_terminated,
};

// ── IUnknown Implementations ─────────────────────────────────────────────────

unsafe extern "system" fn tip_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || riid.is_null() {
        return E_POINTER;
    }
    let p_this = this as *mut LekhaniTextInputProcessor;
    let iid = &*riid;

    if guid_eq(iid, &IID_IUNKNOWN)
        || guid_eq(iid, &IID_ITFTEXTINPUTPROCESSOR)
        || guid_eq(iid, &IID_ITFTEXTINPUTPROCESSOREX)
    {
        *ppv = this;
    } else if guid_eq(iid, &IID_ITFKEYEVENTSINK) {
        *ppv = &mut (*p_this).lp_key_sink_vtbl as *mut _ as *mut c_void;
    } else if guid_eq(iid, &IID_ITFCOMPOSITIONSINK) {
        *ppv = &mut (*p_this).lp_comp_sink_vtbl as *mut _ as *mut c_void;
    } else {
        *ppv = std::ptr::null_mut();
        return E_NOINTERFACE;
    }

    (*p_this).ref_count.fetch_add(1, Ordering::SeqCst);
    S_OK
}

unsafe extern "system" fn tip_add_ref(this: *mut c_void) -> u32 {
    let p_this = this as *mut LekhaniTextInputProcessor;
    (*p_this).ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn tip_release(this: *mut c_void) -> u32 {
    let p_this = this as *mut LekhaniTextInputProcessor;
    let prev = (*p_this).ref_count.fetch_sub(1, Ordering::SeqCst);
    if prev == 1 {
        let _ = Box::from_raw(p_this);
        0
    } else {
        prev - 1
    }
}

unsafe extern "system" fn key_sink_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_key_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_query_interface(base, riid, ppv)
}

unsafe extern "system" fn key_sink_add_ref(this: *mut c_void) -> u32 {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_key_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_add_ref(base)
}

unsafe extern "system" fn key_sink_release(this: *mut c_void) -> u32 {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_key_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_release(base)
}

unsafe extern "system" fn comp_sink_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_comp_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_query_interface(base, riid, ppv)
}

unsafe extern "system" fn comp_sink_add_ref(this: *mut c_void) -> u32 {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_comp_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_add_ref(base)
}

unsafe extern "system" fn comp_sink_release(this: *mut c_void) -> u32 {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_comp_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut c_void;
    tip_release(base)
}

// ── ITfTextInputProcessor(Ex) Implementations ─────────────────────────────────

unsafe extern "system" fn tip_activate(
    this: *mut c_void,
    ptim: *mut c_void,
    tid: TfClientId,
) -> HRESULT {
    tip_activate_ex(this, ptim, tid, 0)
}

unsafe extern "system" fn tip_activate_ex(
    this: *mut c_void,
    ptim: *mut c_void,
    tid: TfClientId,
    _dw_flags: u32,
) -> HRESULT {
    let p_this = this as *mut LekhaniTextInputProcessor;
    if let Ok(mut inner) = (*p_this).inner.lock() {
        inner.thread_mgr = ptim;
        inner.client_id = tid;
    }
    tracing::info!("Lekhani TSF TIP Activated with ClientId: {}", tid);
    S_OK
}

unsafe extern "system" fn tip_deactivate(this: *mut c_void) -> HRESULT {
    let p_this = this as *mut LekhaniTextInputProcessor;
    if let Ok(mut inner) = (*p_this).inner.lock() {
        inner.thread_mgr = std::ptr::null_mut();
        inner.client_id = 0;
        inner.session.clear_context();
    }
    tracing::info!("Lekhani TSF TIP Deactivated");
    S_OK
}

// ── ITfKeyEventSink Implementations ──────────────────────────────────────────

unsafe extern "system" fn key_sink_on_set_focus(
    _this: *mut c_void,
    _f_foreground: BOOL,
) -> HRESULT {
    S_OK
}

unsafe extern "system" fn key_sink_on_test_key_down(
    this: *mut c_void,
    _pic: *mut c_void,
    w_param: WPARAM,
    _l_param: LPARAM,
    pf_eaten: *mut BOOL,
) -> HRESULT {
    if pf_eaten.is_null() {
        return E_POINTER;
    }

    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_key_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut LekhaniTextInputProcessor;

    let vk = w_param as u16;
    let is_toggle = vk == VK_F12;

    if let Ok(inner) = (*base).inner.lock() {
        if is_toggle {
            *pf_eaten = 1;
            return S_OK;
        }

        if inner.bengali_active {
            if (0x41..=0x5A).contains(&vk)
                || (0x30..=0x39).contains(&vk)
                || vk == VK_SPACE
                || inner.session.is_active()
            {
                *pf_eaten = 1;
                return S_OK;
            }
        }
    }

    *pf_eaten = 0;
    S_OK
}

unsafe extern "system" fn key_sink_on_key_down(
    this: *mut c_void,
    _pic: *mut c_void,
    w_param: WPARAM,
    _l_param: LPARAM,
    pf_eaten: *mut BOOL,
) -> HRESULT {
    if pf_eaten.is_null() {
        return E_POINTER;
    }

    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_key_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut LekhaniTextInputProcessor;

    let vk = w_param as u16;

    if vk == VK_F12 {
        if let Ok(mut inner) = (*base).inner.lock() {
            inner.bengali_active = !inner.bengali_active;
            inner.session.clear_context();
        }
        *pf_eaten = 1;
        return S_OK;
    }

    if let Ok(mut inner) = (*base).inner.lock() {
        if inner.bengali_active {
            let shift_down =
                (GetKeyState(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT as i32)
                    & 0x8000u16 as i16)
                    != 0;
            if let Some(ch) = map_vk(vk, shift_down) {
                let keycode = inner.mapper.map_keyval(ch as u32);
                let consumed = inner
                    .session
                    .process_key(keycode, if shift_down { 1 } else { 0 });
                if consumed {
                    *pf_eaten = 1;
                    return S_OK;
                }
            }
        }
    }

    *pf_eaten = 0;
    S_OK
}

unsafe extern "system" fn key_sink_on_test_key_up(
    _this: *mut c_void,
    _pic: *mut c_void,
    _w_param: WPARAM,
    _l_param: LPARAM,
    pf_eaten: *mut BOOL,
) -> HRESULT {
    if !pf_eaten.is_null() {
        *pf_eaten = 0;
    }
    S_OK
}

unsafe extern "system" fn key_sink_on_key_up(
    _this: *mut c_void,
    _pic: *mut c_void,
    _w_param: WPARAM,
    _l_param: LPARAM,
    pf_eaten: *mut BOOL,
) -> HRESULT {
    if !pf_eaten.is_null() {
        *pf_eaten = 0;
    }
    S_OK
}

unsafe extern "system" fn key_sink_on_preserved_key(
    _this: *mut c_void,
    _pic: *mut c_void,
    _rguid: *const GUID,
    pf_eaten: *mut BOOL,
) -> HRESULT {
    if !pf_eaten.is_null() {
        *pf_eaten = 0;
    }
    S_OK
}

// ── ITfCompositionSink Implementation ────────────────────────────────────────

unsafe extern "system" fn comp_sink_on_composition_terminated(
    this: *mut c_void,
    _ec_write: TfEditCookie,
    _p_composition: *mut c_void,
) -> HRESULT {
    let offset = std::mem::offset_of!(LekhaniTextInputProcessor, lp_comp_sink_vtbl);
    let base = (this as *mut u8).sub(offset) as *mut LekhaniTextInputProcessor;
    if let Ok(mut inner) = (*base).inner.lock() {
        inner.active_composition = std::ptr::null_mut();
        inner.session.clear_context();
    }
    S_OK
}

fn map_vk(vk: u16, shift: bool) -> Option<char> {
    match vk {
        0x41..=0x5A => {
            let base = (vk - 0x41) as u8;
            if shift {
                Some((b'A' + base) as char)
            } else {
                Some((b'a' + base) as char)
            }
        }
        0x30..=0x39 => {
            if shift {
                let shift_digits = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
                Some(shift_digits[(vk - 0x30) as usize])
            } else {
                Some((b'0' + (vk - 0x30) as u8) as char)
            }
        }
        VK_SPACE => Some(' '),
        _ => None,
    }
}
