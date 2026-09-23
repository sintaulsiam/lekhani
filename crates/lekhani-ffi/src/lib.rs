//! Lekhani C-ABI FFI Library
//!
//! Provides a C-compatible interface for Lekhani Core, used by the native
//! Fcitx5 shared library plugin and external integrations.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use lekhani_core::{
    ActiveLayoutType, InputSession, KeycodeMapper, MODIFIER_ALT_GR, MODIFIER_SHIFT, VC_UNKNOWN,
};
use lekhani_settings::{ConfigManager, LayoutManager};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{OnceLock, RwLock};

// Special X11 Keysyms
const KEY_BACKSPACE: u32 = 0xff08;
const KEY_RETURN: u32 = 0xff0d;
const KEY_KP_ENTER: u32 = 0xff8d;
const KEY_SPACE: u32 = 0x0020;
const KEY_LEFT: u32 = 0xff51;
const KEY_UP: u32 = 0xff52;
const KEY_RIGHT: u32 = 0xff53;
const KEY_DOWN: u32 = 0xff54;
const KEY_TAB: u32 = 0xff09;
const KEY_ALT_R: u32 = 0xffea;
const KEY_ISO_LEVEL3_SHIFT: u32 = 0xfe03;
const KEY_ESCAPE: u32 = 0xff1b;
const KEY_1: u32 = 0x0031;
const KEY_5: u32 = 0x0035;
const KEY_KP_1: u32 = 0xffb1;
const KEY_KP_5: u32 = 0xffb5;

struct GlobalSharedResources {
    config_mgr: ConfigManager,
    layout_mgr: LayoutManager,
    session_template: InputSession,
}

static GLOBAL_RESOURCES: OnceLock<RwLock<GlobalSharedResources>> = OnceLock::new();

fn get_global_resources() -> &'static RwLock<GlobalSharedResources> {
    GLOBAL_RESOURCES.get_or_init(|| {
        let config_mgr = ConfigManager::new();
        let mut layout_mgr = LayoutManager::new();

        let system_dir = ConfigManager::get_system_layout_dir();
        let user_dir = config_mgr.get_user_layout_dir();
        layout_mgr.discover_layouts(system_dir, user_dir);

        let mut session_template = InputSession::new();
        let system_data = ConfigManager::get_system_data_dir();
        let user_ac = config_mgr.get_user_autocorrect_path();
        let user_learned = config_mgr.get_user_learned_path();
        let stats_path = config_mgr.get_user_stats_path();
        session_template.load_database(&system_data);
        session_template.load_user_autocorrect(&user_ac);
        session_template.load_user_learned(&user_learned);
        session_template.load_stats(&stats_path);
        config_mgr.config.apply_to_session(&mut session_template);

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
            session_template.set_layout(layout_type, &json);
        }

        RwLock::new(GlobalSharedResources {
            config_mgr,
            layout_mgr,
            session_template,
        })
    })
}

pub struct LekhaniEngineContext {
    pub session: InputSession,
    pub config_mgr: ConfigManager,
    pub layout_mgr: LayoutManager,
    pub mapper: KeycodeMapper,
    pub alt_gr: bool,
    // Cached strings to return safely over FFI
    last_commit: Option<CString>,
    last_preedit: Option<CString>,
    last_aux: Option<CString>,
    last_candidates: Vec<CString>,
    pub commit_count: u64,
}

impl Default for LekhaniEngineContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LekhaniEngineContext {
    pub fn new() -> Self {
        let (config_mgr, layout_mgr, session) = {
            let lock = get_global_resources().read().unwrap();
            (
                lock.config_mgr.clone(),
                lock.layout_mgr.clone(),
                lock.session_template.clone(),
            )
        };

        Self {
            session,
            config_mgr,
            layout_mgr,
            mapper: KeycodeMapper::new(),
            alt_gr: false,
            last_commit: None,
            last_preedit: None,
            last_aux: None,
            last_candidates: Vec::new(),
            commit_count: 0,
        }
    }

    /// Commit candidate and flush learning & stats to disk for real-time telemetry
    pub fn commit(&mut self, idx: usize) -> Option<String> {
        let committed = self.session.commit(idx)?;
        self.commit_count += 1;
        if self.commit_count.is_multiple_of(60) {
            let is_dirty = self
                .session
                .phonetic
                .suggestion_engine
                .database
                .learner
                .read()
                .map(|l| l.dirty)
                .unwrap_or(false);
            if is_dirty {
                let user_learned = self.config_mgr.get_user_learned_path();
                let stats_path = self.config_mgr.get_user_stats_path();
                let stats_clone = self.session.get_stats();
                let learner_arc = self.session.phonetic.suggestion_engine.database.learner.clone();
                std::thread::spawn(move || {
                    let _ = stats_clone.save_to_path(&stats_path);
                    if let Ok(mut l) = learner_arc.write() {
                        let _ = l.save_to_path(&user_learned);
                    }
                });
            }
        }
        Some(committed)
    }

    pub fn set_layout(&mut self, layout_name: &str) -> bool {
        if let Some(json) = self.layout_mgr.load_layout_json(layout_name) {
            let layout_type = if self
                .layout_mgr
                .get_layout(layout_name)
                .map(|i| i.layout_type.as_str())
                == Some("fixed")
            {
                ActiveLayoutType::Fixed
            } else {
                ActiveLayoutType::Phonetic
            };
            self.session.set_layout(layout_type, &json);
            true
        } else {
            false
        }
    }

    pub fn update_cached_strings(&mut self) {
        let preedit = self.session.get_preedit_text();
        self.last_preedit = if !preedit.is_empty() {
            CString::new(preedit).ok()
        } else {
            None
        };

        let aux = self.session.get_auxiliary_text();
        self.last_aux = if !aux.is_empty() {
            CString::new(aux).ok()
        } else {
            None
        };

        self.last_candidates.clear();
        for cand in self.session.get_candidates() {
            if let Ok(c_str) = CString::new(cand.as_str()) {
                self.last_candidates.push(c_str);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// C-ABI Exported Functions
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn lekhani_engine_new() -> *mut LekhaniEngineContext {
    let ctx = Box::new(LekhaniEngineContext::new());
    Box::into_raw(ctx)
}

#[no_mangle]
pub extern "C" fn lekhani_engine_free(ctx: *mut LekhaniEngineContext) {
    if !ctx.is_null() {
        unsafe {
            let engine = &mut *ctx;
            let is_dirty = engine
                .session
                .phonetic
                .suggestion_engine
                .database
                .learner
                .read()
                .map(|l| l.dirty)
                .unwrap_or(false);
            let user_learned = engine.config_mgr.get_user_learned_path();
            let stats_path = engine.config_mgr.get_user_stats_path();
            let stats_clone = engine.session.get_stats();
            let learner_arc = engine.session.phonetic.suggestion_engine.database.learner.clone();
            std::thread::spawn(move || {
                let _ = stats_clone.save_to_path(&stats_path);
                if is_dirty {
                    if let Ok(mut l) = learner_arc.write() {
                        let _ = l.save_to_path(&user_learned);
                    }
                }
            });
            drop(Box::from_raw(ctx));
        }
    }
}

#[no_mangle]
pub extern "C" fn lekhani_engine_set_layout(
    ctx: *mut LekhaniEngineContext,
    layout_name: *const c_char,
) -> bool {
    if ctx.is_null() || layout_name.is_null() {
        return false;
    }
    let engine = unsafe { &mut *ctx };
    let c_str = unsafe { CStr::from_ptr(layout_name) };
    if let Ok(name) = c_str.to_str() {
        engine.set_layout(name)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn lekhani_engine_reload_config(ctx: *mut LekhaniEngineContext) {
    if ctx.is_null() {
        return;
    }
    let engine = unsafe { &mut *ctx };
    engine.config_mgr.load();
    let system_dir = ConfigManager::get_system_layout_dir();
    let user_dir = engine.config_mgr.get_user_layout_dir();
    engine.layout_mgr.discover_layouts(system_dir, user_dir);
    let active_name = engine.config_mgr.config.general.active_layout.clone();
    engine.set_layout(&active_name);
}

#[no_mangle]
pub extern "C" fn lekhani_engine_reset(ctx: *mut LekhaniEngineContext) {
    if ctx.is_null() {
        return;
    }
    let engine = unsafe { &mut *ctx };
    engine.session.reset();
    engine.last_commit = None;
    engine.update_cached_strings();
}

#[no_mangle]
pub extern "C" fn lekhani_engine_process_key(
    ctx: *mut LekhaniEngineContext,
    keyval: u32,
    _keycode: u32,
    state_mask: u32,
    is_release: bool,
) -> bool {
    if ctx.is_null() {
        return false;
    }
    let engine = unsafe { &mut *ctx };
    engine.last_commit = None;

    if is_release {
        if keyval == KEY_ALT_R || keyval == KEY_ISO_LEVEL3_SHIFT {
            engine.alt_gr = false;
        }
        return false;
    }

    // Standalone modifier keys (Shift_L, Shift_R, Ctrl, Alt, Super, AltGr, CapsLock)
    // must NOT commit active preedit or dismiss the candidate window.
    if matches!(keyval, 0xffe1..=0xffee | 0xfe03 | 0xfe08 | 0xfe09) {
        if keyval == KEY_ALT_R || keyval == KEY_ISO_LEVEL3_SHIFT {
            engine.alt_gr = true;
        }
        return false;
    }

    // Auto-sync configuration and autocorrect if modified externally
    if engine.config_mgr.check_and_reload() {
        let cfg = engine.config_mgr.config.clone();
        cfg.apply_to_session(&mut engine.session);
        let user_ac = engine.config_mgr.get_user_autocorrect_path();
        engine.session.load_user_autocorrect(&user_ac);
        let system_dir = ConfigManager::get_system_layout_dir();
        let user_dir = engine.config_mgr.get_user_layout_dir();
        engine.layout_mgr.discover_layouts(system_dir, user_dir);
        let active_name = engine.config_mgr.config.general.active_layout.clone();
        engine.set_layout(&active_name);
    }

    // Direct Selection via 1..5 in Prediction Mode
    if engine.session.is_prediction_mode() {
        let cand_idx = if (KEY_1..=KEY_5).contains(&keyval) {
            Some((keyval - KEY_1) as usize)
        } else if (KEY_KP_1..=KEY_KP_5).contains(&keyval) {
            Some((keyval - KEY_KP_1) as usize)
        } else {
            None
        };

        if let Some(idx) = cand_idx {
            if idx < engine.session.get_candidates().len() {
                if let Some(committed) = engine.commit(idx) {
                    engine.last_commit = CString::new(committed).ok();
                    if engine
                        .config_mgr
                        .config
                        .phonetic
                        .enable_predictive_next_words
                    {
                        engine.session.populate_predictions();
                    }
                    engine.update_cached_strings();
                    return true;
                }
            }
        }
    }

    // Backspace
    if keyval == KEY_BACKSPACE {
        if engine.session.is_active() {
            let handled = engine.session.process_backspace();
            engine.update_cached_strings();
            return handled;
        }
        return false;
    }

    // Return
    if keyval == KEY_RETURN {
        if engine.session.is_prediction_mode() {
            if engine.session.is_prediction_navigated() {
                let idx = engine.session.get_selected_index();
                if let Some(committed) = engine.commit(idx) {
                    engine.last_commit = CString::new(committed).ok();
                    if engine
                        .config_mgr
                        .config
                        .phonetic
                        .enable_predictive_next_words
                    {
                        engine.session.populate_predictions();
                    }
                }
                engine.update_cached_strings();
                return true;
            } else {
                engine.session.reset();
                engine.update_cached_strings();
                return false;
            }
        }
        if engine.session.is_active() {
            let idx = engine.session.get_selected_index();
            if let Some(committed) = engine.commit(idx) {
                engine.last_commit = CString::new(committed).ok();
                if engine
                    .config_mgr
                    .config
                    .phonetic
                    .enable_predictive_next_words
                {
                    engine.session.populate_predictions();
                }
            }
            engine.update_cached_strings();
            return engine
                .config_mgr
                .config
                .phonetic
                .enter_key_closes_candidate_window;
        }
        return false;
    }

    // Space or Keypad Enter
    if keyval == KEY_SPACE || keyval == KEY_KP_ENTER {
        if engine.session.is_prediction_mode() {
            if engine.session.is_prediction_navigated() {
                let idx = engine.session.get_selected_index();
                if let Some(committed) = engine.commit(idx) {
                    engine.last_commit = CString::new(committed).ok();
                    if engine
                        .config_mgr
                        .config
                        .phonetic
                        .enable_predictive_next_words
                    {
                        engine.session.populate_predictions();
                    }
                }
                engine.update_cached_strings();
                return true;
            } else {
                engine.session.reset();
                engine.update_cached_strings();
                return false;
            }
        }
        if engine.session.is_active() {
            let idx = engine.session.get_selected_index();
            if let Some(committed) = engine.commit(idx) {
                engine.last_commit = CString::new(committed).ok();
                if engine
                    .config_mgr
                    .config
                    .phonetic
                    .enable_predictive_next_words
                {
                    engine.session.populate_predictions();
                }
            }
            engine.update_cached_strings();
        }
        return false;
    }

    // Navigation & Candidate Selection
    if keyval == KEY_RIGHT || keyval == KEY_DOWN || keyval == KEY_TAB {
        if engine.session.is_active() {
            engine.session.select_next();
            engine.update_cached_strings();
            return true;
        }
        return false;
    }

    if keyval == KEY_LEFT || keyval == KEY_UP {
        if engine.session.is_active() {
            engine.session.select_prev();
            engine.update_cached_strings();
            return true;
        }
        return false;
    }

    if keyval == KEY_ESCAPE {
        if engine.session.is_active() {
            engine.session.reset();
            engine.update_cached_strings();
            return true;
        }
        return false;
    }

    if keyval == KEY_ALT_R || keyval == KEY_ISO_LEVEL3_SHIFT {
        engine.alt_gr = true;
        return engine.session.is_active();
    }

    // Pass modifier hotkeys (Ctrl+C, Alt+Tab, etc.) through to app
    let is_ctrl = (state_mask & (1 << 2)) != 0;
    let is_alt = (state_mask & (1 << 3)) != 0;
    if is_ctrl || (is_alt && !engine.alt_gr) {
        if engine.session.is_active() {
            let idx = engine.session.get_selected_index();
            if let Some(committed) = engine.commit(idx) {
                engine.last_commit = CString::new(committed).ok();
            }
            engine.update_cached_strings();
        }
        return false;
    }

    let mut mod_mask = 0u8;
    if (state_mask & (1 << 0)) != 0 {
        mod_mask |= MODIFIER_SHIFT;
    }
    if engine.alt_gr || ((state_mask & (1 << 2)) != 0 && (state_mask & (1 << 3)) != 0) {
        mod_mask |= MODIFIER_ALT_GR;
    }

    let vc = engine.mapper.map_keyval(keyval);
    if vc == VC_UNKNOWN {
        if engine.session.is_active() {
            let idx = engine.session.get_selected_index();
            if let Some(committed) = engine.commit(idx) {
                engine.last_commit = CString::new(committed).ok();
            }
            engine.update_cached_strings();
        }
        return false;
    }

    let handled = engine.session.process_key(vc, mod_mask);
    engine.update_cached_strings();
    handled
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_commit_text(ctx: *mut LekhaniEngineContext) -> *const c_char {
    if ctx.is_null() {
        return ptr::null();
    }
    let engine = unsafe { &*ctx };
    engine
        .last_commit
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn lekhani_engine_clear_commit_text(ctx: *mut LekhaniEngineContext) {
    if !ctx.is_null() {
        let engine = unsafe { &mut *ctx };
        engine.last_commit = None;
    }
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_preedit_text(ctx: *mut LekhaniEngineContext) -> *const c_char {
    if ctx.is_null() {
        return ptr::null();
    }
    let engine = unsafe { &*ctx };
    engine
        .last_preedit
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_auxiliary_text(
    ctx: *mut LekhaniEngineContext,
) -> *const c_char {
    if ctx.is_null() {
        return ptr::null();
    }
    let engine = unsafe { &*ctx };
    engine
        .last_aux
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_candidate_count(ctx: *mut LekhaniEngineContext) -> usize {
    if ctx.is_null() {
        return 0;
    }
    let engine = unsafe { &*ctx };
    engine.last_candidates.len()
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_candidate_at(
    ctx: *mut LekhaniEngineContext,
    index: usize,
) -> *const c_char {
    if ctx.is_null() {
        return ptr::null();
    }
    let engine = unsafe { &*ctx };
    if index < engine.last_candidates.len() {
        engine.last_candidates[index].as_ptr()
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn lekhani_engine_get_selected_candidate_index(
    ctx: *mut LekhaniEngineContext,
) -> usize {
    if ctx.is_null() {
        return 0;
    }
    let engine = unsafe { &*ctx };
    engine.session.get_selected_index()
}

#[no_mangle]
pub extern "C" fn lekhani_engine_select_candidate(
    ctx: *mut LekhaniEngineContext,
    index: usize,
) -> bool {
    if ctx.is_null() {
        return false;
    }
    let engine = unsafe { &mut *ctx };
    if let Some(committed) = engine.commit(index) {
        engine.last_commit = CString::new(committed).ok();
        if engine
            .config_mgr
            .config
            .phonetic
            .enable_predictive_next_words
        {
            engine.session.populate_predictions();
        }
        engine.update_cached_strings();
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn lekhani_engine_is_active(ctx: *mut LekhaniEngineContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    let engine = unsafe { &*ctx };
    engine.session.is_active()
}

#[no_mangle]
pub extern "C" fn lekhani_engine_is_prediction_mode(ctx: *mut LekhaniEngineContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    let engine = unsafe { &*ctx };
    engine.session.is_prediction_mode()
}

#[no_mangle]
pub extern "C" fn lekhani_engine_is_prediction_navigated(ctx: *mut LekhaniEngineContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    let engine = unsafe { &*ctx };
    engine.session.is_prediction_navigated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_typing_and_suggestions() {
        let engine_ptr = lekhani_engine_new();
        assert!(!engine_ptr.is_null());
        unsafe {
            (*engine_ptr).set_layout("Avro Phonetic");
            (*engine_ptr)
                .session
                .phonetic
                .suggestion_engine
                .database
                .learner
                .write()
                .unwrap()
                .clear_user_data();
            (*engine_ptr).config_mgr.config.general.active_layout = "Avro Phonetic".to_string();
            (*engine_ptr)
                .config_mgr
                .config
                .phonetic
                .enable_predictive_next_words = true;
        }

        // Type 'a' (0x61), 'm' (0x6d), 'i' (0x69)
        lekhani_engine_process_key(engine_ptr, 0x0061, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x006d, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0069, 0, 0, false);

        let count = lekhani_engine_get_candidate_count(engine_ptr);
        assert!(count > 0, "Candidate count should be greater than 0");

        let first_cand_ptr = lekhani_engine_get_candidate_at(engine_ptr, 0);
        assert!(!first_cand_ptr.is_null());
        let first_cand = unsafe { CStr::from_ptr(first_cand_ptr).to_str().unwrap() };
        assert_eq!(first_cand, "আমি");

        // Commit with Space (0x20)
        lekhani_engine_process_key(engine_ptr, KEY_SPACE, 0, 0, false);
        let commit_ptr = lekhani_engine_get_commit_text(engine_ptr);
        assert!(!commit_ptr.is_null());
        let commit_str = unsafe { CStr::from_ptr(commit_ptr).to_str().unwrap() };
        assert_eq!(commit_str, "আমি");

        // Verify proactive predictions are active after committing "আমি"
        let pred_count = lekhani_engine_get_candidate_count(engine_ptr);
        assert!(pred_count > 0, "Proactive prediction count should be > 0");

        // Verify Enter key in unnavigated prediction mode does NOT eat Enter, but passes through (false) and clears predictions for newline
        let enter_handled = lekhani_engine_process_key(engine_ptr, KEY_RETURN, 0, 0, false);
        assert!(
            !enter_handled,
            "Enter key should pass through to application to insert newline"
        );
        assert_eq!(lekhani_engine_get_candidate_count(engine_ptr), 0);

        // Repopulate predictions and test Direct Selection with '2' (KEY_2 = 0x32 -> candidate index 1)
        unsafe {
            (*engine_ptr).session.populate_predictions();
            (*engine_ptr).update_cached_strings();
        }
        let expected_cand_1 = unsafe {
            let p = lekhani_engine_get_candidate_at(engine_ptr, 1);
            assert!(!p.is_null());
            CStr::from_ptr(p).to_str().unwrap().to_string()
        };
        let handled_2 = lekhani_engine_process_key(engine_ptr, KEY_1 + 1, 0, 0, false);
        assert!(handled_2, "Pressing '2' should directly select candidate 2");
        let pred_commit_str = unsafe {
            CStr::from_ptr(lekhani_engine_get_commit_text(engine_ptr))
                .to_str()
                .unwrap()
        };
        assert_eq!(pred_commit_str, expected_cand_1);

        // Test Tab navigation followed by Enter committing navigated candidate
        let tab_handled = lekhani_engine_process_key(engine_ptr, KEY_TAB, 0, 0, false);
        assert!(tab_handled, "Tab should navigate prediction candidates");
        let enter_nav_handled = lekhani_engine_process_key(engine_ptr, KEY_RETURN, 0, 0, false);
        assert!(
            enter_nav_handled,
            "Enter after Tab navigation should commit selected prediction"
        );

        // Type 's', 'h', 'i', 'r', 't'
        for ch in "shirt".chars() {
            lekhani_engine_process_key(engine_ptr, ch as u32, 0, 0, false);
        }
        let shirt_cand = unsafe {
            CStr::from_ptr(lekhani_engine_get_candidate_at(engine_ptr, 0))
                .to_str()
                .unwrap()
        };
        assert_eq!(shirt_cand, "শার্ট");
        lekhani_engine_process_key(engine_ptr, KEY_SPACE, 0, 0, false);

        // Type 'p', 'o', 'r', 'a' after "শার্ট" -> should rank "পরা" first
        for ch in "pora".chars() {
            lekhani_engine_process_key(engine_ptr, ch as u32, 0, 0, false);
        }
        let pora_cand = unsafe {
            CStr::from_ptr(lekhani_engine_get_candidate_at(engine_ptr, 0))
                .to_str()
                .unwrap()
        };
        assert_eq!(pora_cand, "পরা");
        lekhani_engine_process_key(engine_ptr, KEY_SPACE, 0, 0, false);

        // Type emoji shortcode :smile: -> ':' (0x3a), 's', 'm', 'i', 'l', 'e', ':'
        lekhani_engine_process_key(engine_ptr, 0x003a, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0073, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x006d, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0069, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x006c, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0065, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x003a, 0, 0, false);

        let emoji_cand_ptr = lekhani_engine_get_candidate_at(engine_ptr, 0);
        assert!(!emoji_cand_ptr.is_null());
        let emoji_cand = unsafe { CStr::from_ptr(emoji_cand_ptr).to_str().unwrap() };
        assert_eq!(emoji_cand, "😊");

        // Commit emoji with Space
        lekhani_engine_process_key(engine_ptr, KEY_SPACE, 0, 0, false);
        let emoji_commit_ptr = lekhani_engine_get_commit_text(engine_ptr);
        assert!(!emoji_commit_ptr.is_null());
        let emoji_commit = unsafe { CStr::from_ptr(emoji_commit_ptr).to_str().unwrap() };
        assert_eq!(emoji_commit, "😊");

        // Type live prefix emoji :sm -> ':' (0x3a), 's', 'm'
        lekhani_engine_process_key(engine_ptr, 0x003a, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0073, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x006d, 0, 0, false);

        let prefix_cand_ptr = lekhani_engine_get_candidate_at(engine_ptr, 0);
        assert!(!prefix_cand_ptr.is_null());
        let prefix_cand = unsafe { CStr::from_ptr(prefix_cand_ptr).to_str().unwrap() };
        assert_eq!(prefix_cand, "😊");
        lekhani_engine_reset(engine_ptr);

        // Type math formula =125*8 -> '=' (0x3d), '1', '2', '5', '*', '8'
        lekhani_engine_process_key(engine_ptr, 0x003d, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0031, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0032, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0035, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x002a, 0, 0, false);
        lekhani_engine_process_key(engine_ptr, 0x0038, 0, 0, false);

        let math_cand_ptr = lekhani_engine_get_candidate_at(engine_ptr, 0);
        assert!(!math_cand_ptr.is_null());
        let math_cand = unsafe { CStr::from_ptr(math_cand_ptr).to_str().unwrap() };
        assert_eq!(math_cand, "১,০০০");

        // Clean up
        lekhani_engine_free(engine_ptr);
    }

    #[test]
    fn test_engine_layout_switching() {
        let engine_ptr = lekhani_engine_new();
        assert!(!engine_ptr.is_null());

        // Switch to Probhat
        let probhat = CString::new("Probhat").unwrap();
        let switched = lekhani_engine_set_layout(engine_ptr, probhat.as_ptr());
        assert!(switched, "Should switch to Probhat layout");

        // Probhat: 'k' -> 'ক'
        lekhani_engine_process_key(engine_ptr, 0x006b, 0, 0, false);
        let preedit_ptr = lekhani_engine_get_preedit_text(engine_ptr);
        assert!(!preedit_ptr.is_null());
        let preedit = unsafe { CStr::from_ptr(preedit_ptr).to_str().unwrap() };
        assert_eq!(preedit, "ক");

        // Switch to Unijoy: 'k' -> 'ত', 'j' -> 'ক'
        let unijoy = CString::new("Unijoy").unwrap();
        let switched_unijoy = lekhani_engine_set_layout(engine_ptr, unijoy.as_ptr());
        assert!(switched_unijoy, "Should switch to Unijoy layout");
        lekhani_engine_reset(engine_ptr);

        lekhani_engine_process_key(engine_ptr, 0x006a, 0, 0, false);
        let preedit_ptr2 = lekhani_engine_get_preedit_text(engine_ptr);
        assert!(!preedit_ptr2.is_null());
        let preedit2 = unsafe { CStr::from_ptr(preedit_ptr2).to_str().unwrap() };
        assert_eq!(preedit2, "ক");

        // Clean up
        lekhani_engine_free(engine_ptr);
    }
}
