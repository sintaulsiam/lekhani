//! Lekhani Text Input Processor (TIP) Core COM Implementation

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_BACK, VK_DOWN, VK_ESCAPE, VK_F12, VK_LEFT, VK_OEM_1, VK_OEM_2, VK_OEM_3,
    VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS,
    VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};

use crate::com::*;
use crate::guid::guid_eq;
use lekhani_core::{ActiveLayoutType, InputSession, KeycodeMapper};
use lekhani_settings::{ConfigManager, LayoutManager};

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
        let mut layout_mgr = LayoutManager::new();
        layout_mgr.discover_layouts(
            ConfigManager::get_system_layout_dir(),
            config_mgr.get_user_layout_dir(),
        );
        let mut session = InputSession::new();
        let system_data = ConfigManager::get_system_data_dir();
        session.load_database(&system_data);
        session.load_user_autocorrect(config_mgr.get_user_autocorrect_path());
        session.load_user_learned(config_mgr.get_user_learned_path());
        config_mgr.config.apply_to_session(&mut session);
        let active_name = &config_mgr.config.general.active_layout;
        if let Some(json) = layout_mgr.load_layout_json(active_name) {
            let layout_type = if layout_mgr
                .get_layout(active_name)
                .map(|i| i.layout_type.as_str())
                == Some("fixed")
            {
                ActiveLayoutType::Fixed
            } else {
                ActiveLayoutType::Phonetic
            };
            session.set_layout(layout_type, &json);
        }

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
            // Desktop rule: Left and Right arrow keys MUST move the cursor in the host app,
            // never cycle candidates or block caret movement.
            if vk == VK_LEFT || vk == VK_RIGHT {
                *pf_eaten = 0;
                return S_OK;
            }

            if inner.session.is_active() {
                let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
                if !shift_down && (0x31..=0x35).contains(&vk) {
                    *pf_eaten = 1;
                    return S_OK;
                }
                match vk {
                    VK_BACK | VK_RETURN | VK_SPACE | VK_ESCAPE | VK_TAB | VK_DOWN | VK_UP => {
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    _ => {}
                }
            }

            let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
            if map_vk(vk, shift_down).is_some() {
                *pf_eaten = 1;
                return S_OK;
            }
        }
    }

    *pf_eaten = 0;
    S_OK
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditSessionKind {
    SetPreedit,
    CommitAndEnd,
}

#[repr(C)]
struct LekhaniEditSession {
    vtbl: *const ITfEditSessionVtbl,
    ref_count: AtomicU32,
    context: *mut c_void,
    comp_sink: *mut c_void,
    comp_ptr: *mut *mut c_void,
    text_utf16: Vec<u16>,
    kind: EditSessionKind,
}

static EDIT_SESSION_VTBL: ITfEditSessionVtbl = ITfEditSessionVtbl {
    parent: IUnknownVtbl {
        QueryInterface: edit_session_qi,
        AddRef: edit_session_add_ref,
        Release: edit_session_release,
    },
    DoEditSession: edit_session_do,
};

unsafe extern "system" fn edit_session_qi(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || riid.is_null() {
        return E_POINTER;
    }
    let iid = &*riid;
    if guid_eq(iid, &IID_IUNKNOWN) || guid_eq(iid, &IID_ITFEDITSESSION) {
        *ppv = this;
        edit_session_add_ref(this);
        S_OK
    } else {
        *ppv = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn edit_session_add_ref(this: *mut c_void) -> u32 {
    let es = this as *mut LekhaniEditSession;
    (*es).ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn edit_session_release(this: *mut c_void) -> u32 {
    let es = this as *mut LekhaniEditSession;
    let prev = (*es).ref_count.fetch_sub(1, Ordering::SeqCst);
    if prev == 1 {
        let _ = Box::from_raw(es);
        0
    } else {
        prev - 1
    }
}

unsafe fn collapse_and_select_end(ctx: *mut c_void, ec: TfEditCookie, range: *mut c_void) {
    let r_vtbl = *(range as *const *const ITfRangeVtbl);
    let _ = ((*r_vtbl).Collapse)(range, ec, TF_ANCHOR_END);
    let sel = TF_SELECTION {
        range,
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_END,
            fInterimChar: 0,
        },
    };
    let ctx_vtbl = *(ctx as *const *const ITfContextVtbl);
    let _ = ((*ctx_vtbl).SetSelection)(ctx, ec, 1, &sel);
}

unsafe fn insert_at_selection(ctx: *mut c_void, ec: TfEditCookie, utf16: &[u16]) {
    if utf16.is_empty() {
        return;
    }
    let ctx_vtbl = *(ctx as *const *const IUnknownVtbl);
    let mut insert_at_sel: *mut c_void = std::ptr::null_mut();
    let hr = ((*ctx_vtbl).QueryInterface)(ctx, &IID_ITFINSERTATSELECTION, &mut insert_at_sel);
    if hr == S_OK && !insert_at_sel.is_null() {
        let ias_vtbl = *(insert_at_sel as *const *const ITfInsertAtSelectionVtbl);
        let mut range: *mut c_void = std::ptr::null_mut();
        let _ = ((*ias_vtbl).InsertTextAtSelection)(
            insert_at_sel,
            ec,
            TF_IAS_NOQUERY,
            utf16.as_ptr(),
            utf16.len() as i32,
            &mut range,
        );
        if !range.is_null() {
            let r_vtbl = *(range as *const *const IUnknownVtbl);
            ((*r_vtbl).Release)(range);
        }
        ((*ias_vtbl).parent.Release)(insert_at_sel);
    }
}

unsafe extern "system" fn edit_session_do(this: *mut c_void, ec: TfEditCookie) -> HRESULT {
    let es = &*(this as *mut LekhaniEditSession);
    if es.context.is_null() {
        return S_OK;
    }
    let comp_slot = es.comp_ptr;
    let active_comp = if comp_slot.is_null() {
        std::ptr::null_mut()
    } else {
        *comp_slot
    };

    match es.kind {
        EditSessionKind::SetPreedit => {
            if active_comp.is_null() {
                // Start a new composition at current selection
                let ctx_vtbl = *(es.context as *const *const ITfContextVtbl);
                let mut sel = TF_SELECTION {
                    range: std::ptr::null_mut(),
                    style: TF_SELECTIONSTYLE {
                        ase: 0,
                        fInterimChar: 0,
                    },
                };
                let mut fetched: u32 = 0;
                if ((*ctx_vtbl).GetSelection)(
                    es.context,
                    ec,
                    TF_DEFAULT_SELECTION,
                    1,
                    &mut sel,
                    &mut fetched,
                ) == S_OK
                    && fetched > 0
                    && !sel.range.is_null()
                {
                    let mut ctx_comp: *mut c_void = std::ptr::null_mut();
                    let hr = ((*ctx_vtbl).parent.QueryInterface)(
                        es.context,
                        &IID_ITFCONTEXTCOMPOSITION,
                        &mut ctx_comp,
                    );
                    if hr == S_OK && !ctx_comp.is_null() {
                        let cc_vtbl = *(ctx_comp as *const *const ITfContextCompositionVtbl);
                        let mut new_comp: *mut c_void = std::ptr::null_mut();
                        let hr_start = ((*cc_vtbl).StartComposition)(
                            ctx_comp,
                            ec,
                            sel.range,
                            es.comp_sink,
                            &mut new_comp,
                        );
                        if hr_start == S_OK && !new_comp.is_null() && !comp_slot.is_null() {
                            *comp_slot = new_comp;
                            let c_vtbl = *(new_comp as *const *const ITfCompositionVtbl);
                            let mut comp_range: *mut c_void = std::ptr::null_mut();
                            if ((*c_vtbl).GetRange)(new_comp, &mut comp_range) == S_OK
                                && !comp_range.is_null()
                            {
                                let r_vtbl = *(comp_range as *const *const ITfRangeVtbl);
                                let _ = ((*r_vtbl).SetText)(
                                    comp_range,
                                    ec,
                                    0,
                                    es.text_utf16.as_ptr(),
                                    es.text_utf16.len() as i32,
                                );
                                collapse_and_select_end(es.context, ec, comp_range);
                                ((*r_vtbl).parent.Release)(comp_range);
                            }
                        }
                        ((*cc_vtbl).parent.Release)(ctx_comp);
                    }
                    let r_vtbl = *(sel.range as *const *const IUnknownVtbl);
                    ((*r_vtbl).Release)(sel.range);
                }
            } else {
                let c_vtbl = *(active_comp as *const *const ITfCompositionVtbl);
                let mut comp_range: *mut c_void = std::ptr::null_mut();
                if ((*c_vtbl).GetRange)(active_comp, &mut comp_range) == S_OK
                    && !comp_range.is_null()
                {
                    let r_vtbl = *(comp_range as *const *const ITfRangeVtbl);
                    let _ = ((*r_vtbl).SetText)(
                        comp_range,
                        ec,
                        0,
                        es.text_utf16.as_ptr(),
                        es.text_utf16.len() as i32,
                    );
                    collapse_and_select_end(es.context, ec, comp_range);
                    ((*r_vtbl).parent.Release)(comp_range);
                }
            }
        }
        EditSessionKind::CommitAndEnd => {
            if !active_comp.is_null() {
                let c_vtbl = *(active_comp as *const *const ITfCompositionVtbl);
                let mut comp_range: *mut c_void = std::ptr::null_mut();
                if ((*c_vtbl).GetRange)(active_comp, &mut comp_range) == S_OK
                    && !comp_range.is_null()
                {
                    let r_vtbl = *(comp_range as *const *const ITfRangeVtbl);
                    let _ = ((*r_vtbl).SetText)(
                        comp_range,
                        ec,
                        0,
                        es.text_utf16.as_ptr(),
                        es.text_utf16.len() as i32,
                    );
                    collapse_and_select_end(es.context, ec, comp_range);
                    ((*r_vtbl).parent.Release)(comp_range);
                }
                let _ = ((*c_vtbl).EndComposition)(active_comp, ec);
                ((*c_vtbl).parent.Release)(active_comp);
                if !comp_slot.is_null() {
                    *comp_slot = std::ptr::null_mut();
                }
            } else {
                insert_at_selection(es.context, ec, &es.text_utf16);
            }
        }
    }
    S_OK
}

unsafe fn run_edit_session(
    pic: *mut c_void,
    client_id: TfClientId,
    comp_sink: *mut c_void,
    comp_ptr: *mut *mut c_void,
    text: &str,
    kind: EditSessionKind,
) {
    if pic.is_null() || client_id == 0 {
        return;
    }
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let es = Box::into_raw(Box::new(LekhaniEditSession {
        vtbl: &EDIT_SESSION_VTBL,
        ref_count: AtomicU32::new(1),
        context: pic,
        comp_sink,
        comp_ptr,
        text_utf16: utf16,
        kind,
    }));
    let ctx_vtbl = *(pic as *const *const ITfContextVtbl);
    let mut hr_session: HRESULT = S_OK;
    let _ = ((*ctx_vtbl).RequestEditSession)(
        pic,
        client_id,
        es as *mut c_void,
        TF_ES_SYNC | TF_ES_READWRITE,
        &mut hr_session,
    );
    edit_session_release(es as *mut c_void);
}

unsafe fn tsf_set_preedit(
    pic: *mut c_void,
    client_id: TfClientId,
    comp_sink: *mut c_void,
    comp_ptr: *mut *mut c_void,
    text: &str,
) {
    run_edit_session(
        pic,
        client_id,
        comp_sink,
        comp_ptr,
        text,
        EditSessionKind::SetPreedit,
    );
}

unsafe fn tsf_commit_text(
    pic: *mut c_void,
    client_id: TfClientId,
    comp_ptr: *mut *mut c_void,
    text: &str,
) {
    run_edit_session(
        pic,
        client_id,
        std::ptr::null_mut(),
        comp_ptr,
        text,
        EditSessionKind::CommitAndEnd,
    );
}

unsafe extern "system" fn key_sink_on_key_down(
    this: *mut c_void,
    pic: *mut c_void,
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

    let comp_sink = &mut (*base).lp_comp_sink_vtbl as *mut _ as *mut c_void;

    if let Ok(mut inner) = (*base).inner.lock() {
        if inner.bengali_active {
            let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;

            // Arrow keys: if active/prediction, commit current candidate or reset context,
            // but return pf_eaten = 0 so Windows passes the arrow key to the text caret.
            if vk == VK_LEFT || vk == VK_RIGHT {
                if inner.session.is_active() {
                    let sel = inner.session.get_selected_index();
                    if let Some(committed) = inner.session.commit(sel) {
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_commit_text(pic, client_id, comp_ptr, &committed);
                    }
                } else {
                    inner.session.clear_context();
                }
                *pf_eaten = 0;
                return S_OK;
            }

            // IME navigation & control keys when active
            if inner.session.is_active() {
                if !shift_down && (0x31..=0x35).contains(&vk) {
                    let idx = (vk - 0x31) as usize;
                    if let Some(committed) = inner.session.commit(idx) {
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_commit_text(pic, client_id, comp_ptr, &committed);
                        *pf_eaten = 1;
                        return S_OK;
                    }
                }
                match vk {
                    VK_BACK => {
                        let consumed = inner.session.process_backspace();
                        if consumed {
                            let client_id = inner.client_id;
                            let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                            if inner.session.is_active() {
                                let sel = inner.session.get_selected_index();
                                let preedit = inner
                                    .session
                                    .get_candidates()
                                    .get(sel)
                                    .cloned()
                                    .unwrap_or_default();
                                tsf_set_preedit(pic, client_id, comp_sink, comp_ptr, &preedit);
                            } else {
                                tsf_commit_text(pic, client_id, comp_ptr, "");
                            }
                        }
                        *pf_eaten = if consumed { 1 } else { 0 };
                        return S_OK;
                    }
                    VK_RETURN => {
                        let sel = inner.session.get_selected_index();
                        if let Some(committed) = inner.session.commit(sel) {
                            let client_id = inner.client_id;
                            let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                            tsf_commit_text(pic, client_id, comp_ptr, &committed);
                        }
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    VK_SPACE => {
                        let sel = inner.session.get_selected_index();
                        if let Some(mut committed) = inner.session.commit(sel) {
                            committed.push(' ');
                            let client_id = inner.client_id;
                            let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                            tsf_commit_text(pic, client_id, comp_ptr, &committed);
                            *pf_eaten = 1;
                            return S_OK;
                        }
                    }
                    VK_ESCAPE => {
                        inner.session.reset();
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_commit_text(pic, client_id, comp_ptr, "");
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    VK_TAB => {
                        if shift_down {
                            inner.session.select_prev();
                        } else {
                            inner.session.select_next();
                        }
                        let sel = inner.session.get_selected_index();
                        let preedit = inner
                            .session
                            .get_candidates()
                            .get(sel)
                            .cloned()
                            .unwrap_or_default();
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_set_preedit(pic, client_id, comp_sink, comp_ptr, &preedit);
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    VK_DOWN => {
                        inner.session.select_next();
                        let sel = inner.session.get_selected_index();
                        let preedit = inner
                            .session
                            .get_candidates()
                            .get(sel)
                            .cloned()
                            .unwrap_or_default();
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_set_preedit(pic, client_id, comp_sink, comp_ptr, &preedit);
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    VK_UP => {
                        inner.session.select_prev();
                        let sel = inner.session.get_selected_index();
                        let preedit = inner
                            .session
                            .get_candidates()
                            .get(sel)
                            .cloned()
                            .unwrap_or_default();
                        let client_id = inner.client_id;
                        let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                        tsf_set_preedit(pic, client_id, comp_sink, comp_ptr, &preedit);
                        *pf_eaten = 1;
                        return S_OK;
                    }
                    _ => {}
                }
            }

            if let Some(ch) = map_vk(vk, shift_down) {
                let keycode = inner.mapper.map_keyval(ch as u32);
                let consumed = inner
                    .session
                    .process_key(keycode, if shift_down { 1 } else { 0 });
                if consumed {
                    let sel = inner.session.get_selected_index();
                    let preedit = inner
                        .session
                        .get_candidates()
                        .get(sel)
                        .cloned()
                        .unwrap_or_default();
                    let client_id = inner.client_id;
                    let comp_ptr = &mut inner.active_composition as *mut *mut c_void;
                    tsf_set_preedit(pic, client_id, comp_sink, comp_ptr, &preedit);
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
        if !inner.active_composition.is_null() {
            let c_vtbl = *(inner.active_composition as *const *const IUnknownVtbl);
            ((*c_vtbl).Release)(inner.active_composition);
            inner.active_composition = std::ptr::null_mut();
        }
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
        VK_OEM_1 => Some(if shift { ':' } else { ';' }),
        VK_OEM_PLUS => Some(if shift { '+' } else { '=' }),
        VK_OEM_COMMA => Some(if shift { '<' } else { ',' }),
        VK_OEM_MINUS => Some(if shift { '_' } else { '-' }),
        VK_OEM_PERIOD => Some(if shift { '>' } else { '.' }),
        VK_OEM_2 => Some(if shift { '?' } else { '/' }),
        VK_OEM_3 => Some(if shift { '~' } else { '`' }),
        VK_OEM_4 => Some(if shift { '{' } else { '[' }),
        VK_OEM_5 => Some(if shift { '|' } else { '\\' }),
        VK_OEM_6 => Some(if shift { '}' } else { ']' }),
        VK_OEM_7 => Some(if shift { '"' } else { '\'' }),
        _ => None,
    }
}
