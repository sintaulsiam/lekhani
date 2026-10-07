//! Native Windows Global Low-Level Keyboard Hook & Text Injection Engine
//! Emulates the classic Avro Keyboard system-hook architecture for universal Windows compatibility.

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SendInput, ToUnicode, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DOWN,
    VK_ESCAPE, VK_F12, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LWIN, VK_MENU, VK_OEM_1, VK_OEM_2,
    VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD,
    VK_OEM_PLUS, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
    VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
};

use crate::win_candidate::CandidateWindow;
use crate::win_osd::OsdWindow;
use crate::win_tray::WindowsTray;
use lekhani_core::{
    ActiveLayoutType, InputSession, KeycodeMapper, MODIFIER_ALT_GR, MODIFIER_SHIFT,
};
use lekhani_settings::{ConfigManager, LayoutManager};

static BENGALI_ACTIVE: AtomicBool = AtomicBool::new(false);

pub type ModeChangeCallback = Arc<dyn Fn(bool, &str) + Send + Sync>;
static MODE_CALLBACK: Mutex<Option<ModeChangeCallback>> = Mutex::new(None);

pub fn set_mode_change_callback(cb: ModeChangeCallback) {
    *MODE_CALLBACK.lock().unwrap() = Some(cb);
}

struct HookState {
    session: InputSession,
    mapper: KeycodeMapper,
    uncommitted_units: usize,
    tray: Option<Arc<WindowsTray>>,
    active_layout_name: String,
    layout_mgr: LayoutManager,
    candidate_win: Option<CandidateWindow>,
    osd_win: Option<OsdWindow>,
    config_mgr: ConfigManager,
    commit_counter: usize,
}

impl HookState {
    fn new(initial_layout: String) -> Self {
        let config_mgr = ConfigManager::new();
        let mut layout_mgr = LayoutManager::new();
        let mut session = InputSession::new();

        let system_dir = ConfigManager::get_system_layout_dir();
        let user_dir = config_mgr.get_user_layout_dir();
        layout_mgr.discover_layouts(&system_dir, &user_dir);

        let system_data = ConfigManager::get_system_data_dir();
        session.load_database(&system_data);
        session.load_user_autocorrect(config_mgr.get_user_autocorrect_path());
        session.load_user_learned(config_mgr.get_user_learned_path());
        config_mgr.config.apply_to_session(&mut session);

        // Apply active layout
        if let Some(info) = layout_mgr.get_layout(&initial_layout) {
            if let Some(val) = layout_mgr.load_layout_json(&initial_layout) {
                let l_type = if info.layout_type == "fixed" {
                    ActiveLayoutType::Fixed
                } else {
                    ActiveLayoutType::Phonetic
                };
                session.set_layout(l_type, &val);
            }
        }

        Self {
            session,
            mapper: KeycodeMapper::new(),
            uncommitted_units: 0,
            tray: None,
            active_layout_name: initial_layout,
            layout_mgr,
            candidate_win: None,
            osd_win: None,
            config_mgr,
            commit_counter: 0,
        }
    }

    fn on_commit(&mut self) {
        self.commit_counter += 1;
        if self.commit_counter % 60 == 0 {
            self.flush_learned();
        }
    }

    fn flush_learned(&self) {
        let learned_path = self.config_mgr.get_user_learned_path();
        let _ = self.session.save_user_learned(&learned_path);
        let stats_path = self.config_mgr.get_user_stats_path();
        let _ = self.session.save_stats(&stats_path);
    }

    fn reset(&mut self) {
        self.flush_learned();
        self.session.clear_context();
        self.uncommitted_units = 0;
        if let Some(ref win) = self.candidate_win {
            win.hide();
        }
    }

    fn update_candidate_window(&self, candidates: &[String], selected_index: usize) {
        if let Some(ref win) = self.candidate_win {
            win.update(
                candidates,
                selected_index,
                self.config_mgr.config.ui.horizontal_candidates,
            );
        }
    }

    fn set_layout(&mut self, layout_name: &str) {
        if let Some(info) = self.layout_mgr.get_layout(layout_name) {
            if let Some(val) = self.layout_mgr.load_layout_json(layout_name) {
                let l_type = if info.layout_type == "fixed" {
                    ActiveLayoutType::Fixed
                } else {
                    ActiveLayoutType::Phonetic
                };
                self.session.set_layout(l_type, &val);
                self.active_layout_name = layout_name.to_string();
                self.reset();
            }
        }
    }
}

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);
static HOOK_HANDLE: Mutex<Option<usize>> = Mutex::new(None);
static MOUSE_HOOK_HANDLE: Mutex<Option<usize>> = Mutex::new(None);

pub fn toggle_bengali_mode() {
    let new_state = !BENGALI_ACTIVE.load(Ordering::SeqCst);
    set_bengali_mode(new_state);
}

pub fn set_bengali_mode(active: bool) {
    BENGALI_ACTIVE.store(active, Ordering::SeqCst);

    let mut layout_name = String::new();
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            if state.config_mgr.check_and_reload() {
                let cfg = state.config_mgr.config.clone();
                cfg.apply_to_session(&mut state.session);
                let current_layout = cfg.general.active_layout.clone();
                if current_layout != state.active_layout_name {
                    state.set_layout(&current_layout);
                }
            }
            state.reset();
            layout_name = state.active_layout_name.clone();
            let show_osd = state.config_mgr.config.general.show_osd;
            if let Some(ref tray) = state.tray {
                tray.set_bengali_active(active, &layout_name);
            }
            if show_osd {
                if let Some(ref osd) = state.osd_win {
                    if active {
                        osd.show("বাংলা", &format!("Lekhani • {}", layout_name), true);
                    } else {
                        osd.show("English", "Standard Typing Mode", false);
                    }
                }
            }
        }
    }

    if let Ok(guard) = MODE_CALLBACK.lock() {
        if let Some(ref cb) = *guard {
            cb(active, &layout_name);
        }
    }
}

pub fn update_active_layout(name: &str) {
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            state.set_layout(name);
            let active = BENGALI_ACTIVE.load(Ordering::SeqCst);
            if let Some(ref tray) = state.tray {
                tray.set_bengali_active(active, name);
            }
            if state.config_mgr.config.general.show_osd && active {
                if let Some(ref osd) = state.osd_win {
                    osd.show("বাংলা", &format!("Lekhani • {}", name), true);
                }
            }
        }
    }
}

pub fn reload_config() {
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            state.config_mgr.load();
            state.config_mgr.config.apply_to_session(&mut state.session);
            tracing::info!(
                "HookState reloaded configuration: toggle_key = {}",
                state.config_mgr.config.general.toggle_key
            );
        }
    }
}

/// Commit candidate by explicit index (e.g. from mouse click or shortcut)
pub fn commit_candidate_by_index(idx: usize) {
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            let candidates = state.session.get_candidates();
            if idx < candidates.len() {
                if let Some(committed) = state.session.commit(idx) {
                    state.on_commit();
                    inject_backspaces(state.uncommitted_units);
                    inject_unicode_str(&committed);
                    state.uncommitted_units = 0;
                    if state
                        .config_mgr
                        .config
                        .phonetic
                        .enable_predictive_next_words
                    {
                        if state.session.populate_predictions() {
                            let preds = state.session.get_candidates();
                            state.update_candidate_window(preds, 0);
                        } else if let Some(ref win) = state.candidate_win {
                            win.hide();
                        }
                    } else if let Some(ref win) = state.candidate_win {
                        win.hide();
                    }
                }
            }
        }
    }
}

pub fn spawn_windows_hook(initial_layout: String, tray: Option<Arc<WindowsTray>>) {
    let cand_win = CandidateWindow::new();
    let osd_win = OsdWindow::new();
    {
        let mut state = HookState::new(initial_layout.clone());
        state.tray = tray;
        state.candidate_win = cand_win;
        state.osd_win = osd_win;
        let mut guard = HOOK_STATE.lock().unwrap();
        *guard = Some(state);
    }

    std::thread::spawn(|| unsafe {
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        let hmod = GetModuleHandleW(std::ptr::null());

        let kbd_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), hmod, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), hmod, 0);

        if kbd_hook != 0 as _ {
            *HOOK_HANDLE.lock().unwrap() = Some(kbd_hook as usize);
            if mouse_hook != 0 as _ {
                *MOUSE_HOOK_HANDLE.lock().unwrap() = Some(mouse_hook as usize);
            }
            tracing::info!("Windows Low-Level Keyboard & Mouse Hooks installed successfully");

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
                DispatchMessageW(&msg);
            }

            if let Some(h) = *HOOK_HANDLE.lock().unwrap() {
                UnhookWindowsHookEx(h as HHOOK);
            }
            if let Some(mh) = *MOUSE_HOOK_HANDLE.lock().unwrap() {
                UnhookWindowsHookEx(mh as HHOOK);
            }
        } else {
            tracing::error!(
                "Failed to install Windows Low-Level Keyboard Hook: {}",
                windows_sys::Win32::Foundation::GetLastError()
            );
        }
    });
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    let msg = w_param as u32;
    if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN {
        let mouse_struct = *(l_param as *const MSLLHOOKSTRUCT);
        let pt = mouse_struct.pt;

        // Check if user clicked inside the candidate window
        let mut clicked_cand_idx = None;
        let mut is_inside_cand_win = false;
        if let Ok(guard) = HOOK_STATE.lock() {
            if let Some(ref state) = *guard {
                if let Some(ref cand_win) = state.candidate_win {
                    if cand_win.is_visible() {
                        is_inside_cand_win = cand_win.is_point_inside(pt.x, pt.y);
                        clicked_cand_idx = cand_win.hit_test(pt.x, pt.y);
                    }
                }
            }
        }

        if let Some(idx) = clicked_cand_idx {
            commit_candidate_by_index(idx);
            return 1; // Consume click event to select candidate
        }

        if is_inside_cand_win {
            return 1; // Consume stray click on candidate window padding/border so it does not dismiss
        }

        // Click outside candidate window:
        // Safely commit uncommitted preedit and hide candidate window so subsequent typing
        // in the newly clicked window does not send backspaces or corrupted preedit!
        if let Ok(mut guard) = HOOK_STATE.lock() {
            if let Some(ref mut state) = *guard {
                if state.uncommitted_units > 0 || state.session.is_prediction_mode() {
                    state.session.commit(state.session.get_selected_index());
                    state.on_commit();
                    state.uncommitted_units = 0;
                    if let Some(ref win) = state.candidate_win {
                        win.hide();
                    }
                }
            }
        }
    }

    CallNextHookEx(0 as _, n_code, w_param, l_param)
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    let kbd = *(l_param as *const KBDLLHOOKSTRUCT);

    // Ignore synthetic/injected keystrokes to prevent recursive feedback loop
    if (kbd.flags & 0x10) != 0 {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    let is_key_down = (w_param as u32) == WM_KEYDOWN || (w_param as u32) == WM_SYSKEYDOWN;
    if !is_key_down {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    let vk = kbd.vkCode as u16;

    let is_alt_gr = (GetKeyState(VK_RMENU as i32) & 0x8000u16 as i16) != 0;
    let l_ctrl_down = (GetKeyState(VK_LCONTROL as i32) & 0x8000u16 as i16) != 0;
    let r_ctrl_down = (GetKeyState(VK_RCONTROL as i32) & 0x8000u16 as i16) != 0;
    let l_alt_down = (GetKeyState(VK_LMENU as i32) & 0x8000u16 as i16) != 0;
    let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
    let win_down =
        ((GetKeyState(VK_LWIN as i32) | GetKeyState(VK_RWIN as i32)) & 0x8000u16 as i16) != 0;

    // Distinguish genuine shortcuts (Ctrl+C, Alt+Tab, Win+Key) from AltGr (VK_RMENU)
    let real_ctrl_down = r_ctrl_down || (l_ctrl_down && !is_alt_gr);
    let real_alt_down = l_alt_down;

    // Check global toggle hotkeys based on user configuration with F12 safe fallback
    let configured_toggle = if let Ok(guard) = HOOK_STATE.lock() {
        guard
            .as_ref()
            .map(|s| s.config_mgr.config.general.toggle_key.clone())
            .unwrap_or_else(|| "F12".to_string())
    } else {
        "F12".to_string()
    };

    let is_toggle = if configured_toggle.eq_ignore_ascii_case("Ctrl+Space") {
        (vk == VK_SPACE && real_ctrl_down && !real_alt_down && !win_down)
            || (vk == VK_F12 && !real_ctrl_down && !real_alt_down && !win_down)
    } else if configured_toggle.eq_ignore_ascii_case("Shift+Space") {
        (vk == VK_SPACE && shift_down && !real_ctrl_down && !real_alt_down && !win_down)
            || (vk == VK_F12 && !real_ctrl_down && !real_alt_down && !win_down)
    } else if configured_toggle.eq_ignore_ascii_case("Right Alt")
        || configured_toggle.eq_ignore_ascii_case("AltGr")
    {
        (vk == VK_RMENU && !real_ctrl_down && !win_down)
            || (vk == VK_F12 && !real_ctrl_down && !real_alt_down && !win_down)
    } else {
        // Default F12, plus allow Ctrl+Space as standard convenience
        (vk == VK_F12 && !real_ctrl_down && !real_alt_down && !win_down)
            || (vk == VK_SPACE && real_ctrl_down && !real_alt_down && !win_down)
    };

    if is_toggle {
        toggle_bengali_mode();
        return 1;
    }

    if !BENGALI_ACTIVE.load(Ordering::SeqCst) {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Optional instant candidate selection via Ctrl+1..Ctrl+5
    if real_ctrl_down && !real_alt_down && !shift_down && !win_down {
        if (0x31..=0x35).contains(&vk) {
            let target_idx = (vk - 0x31) as usize;
            let mut committed_candidate = false;
            if let Ok(guard) = HOOK_STATE.lock() {
                if let Some(ref s) = *guard {
                    let candidates = s.session.get_candidates();
                    let visible_count = candidates.len().min(5);
                    if target_idx < visible_count {
                        committed_candidate = true;
                    }
                }
            }
            if committed_candidate {
                commit_candidate_by_index(target_idx);
                return 1;
            }
        }
    }

    // If a genuine Ctrl, Alt, or Win shortcut is held down, pass through to native OS
    if real_ctrl_down || real_alt_down || win_down {
        if let Ok(mut guard) = HOOK_STATE.lock() {
            if let Some(ref mut state) = *guard {
                state.reset();
            }
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    let mut guard = match HOOK_STATE.lock() {
        Ok(g) => g,
        Err(_) => return CallNextHookEx(0 as _, n_code, w_param, l_param),
    };

    let state = match guard.as_mut() {
        Some(s) => s,
        None => return CallNextHookEx(0 as _, n_code, w_param, l_param),
    };

    let caps_locked = (GetKeyState(VK_CAPITAL as i32) & 1) != 0;

    // Candidate Navigation in Phonetic Mode (Uncommitted Preedit)
    if state.session.active_layout_type == ActiveLayoutType::Phonetic && state.uncommitted_units > 0
    {
        let candidates = state.session.get_candidates();
        if !candidates.is_empty() {
            let visible_count = candidates.len().min(5);

            // Left / Right Arrow while composing:
            // Commit active candidate, close suggestion window, and pass through to OS so the text cursor moves naturally!
            if vk == VK_LEFT || vk == VK_RIGHT {
                state.session.commit(state.session.get_selected_index());
                state.on_commit();
                state.uncommitted_units = 0;
                if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
                return CallNextHookEx(0 as _, n_code, w_param, l_param);
            }

            // Candidate navigation strictly via Tab / Shift+Tab / Down
            if vk == VK_TAB || vk == VK_DOWN {
                if shift_down {
                    state.session.select_prev();
                    while state.session.get_selected_index() >= visible_count {
                        state.session.select_prev();
                    }
                } else {
                    state.session.select_next();
                    if state.session.get_selected_index() >= visible_count {
                        while state.session.get_selected_index() != 0 {
                            state.session.select_next();
                        }
                    }
                }
                let candidate = state.session.get_preedit_text();
                if !candidate.is_empty() {
                    inject_backspaces(state.uncommitted_units);
                    state.uncommitted_units = inject_unicode_str(&candidate);
                }
                state.update_candidate_window(
                    state.session.get_candidates(),
                    state.session.get_selected_index(),
                );
                return 1;
            }

            // Candidate navigation via Up
            if vk == VK_UP {
                state.session.select_prev();
                while state.session.get_selected_index() >= visible_count {
                    state.session.select_prev();
                }
                let candidate = state.session.get_preedit_text();
                if !candidate.is_empty() {
                    inject_backspaces(state.uncommitted_units);
                    state.uncommitted_units = inject_unicode_str(&candidate);
                }
                state.update_candidate_window(
                    state.session.get_candidates(),
                    state.session.get_selected_index(),
                );
                return 1;
            }
        }
    }

    // Prediction Mode Navigation (Zero Uncommitted Preedit)
    if state.session.active_layout_type == ActiveLayoutType::Phonetic
        && state.uncommitted_units == 0
        && state.session.is_prediction_mode()
    {
        let candidates = state.session.get_candidates();
        if !candidates.is_empty() {
            let visible_count = candidates.len().min(5);

            // Left / Right / Home / End / PageUp / PageDown in Prediction Mode:
            // Dismiss suggestions and pass through so the user moves their text cursor smoothly!
            if vk == VK_LEFT || vk == VK_RIGHT || vk == 0x24 /* VK_HOME */ || vk == 0x23 /* VK_END */
                || vk == 0x21 /* VK_PRIOR */ || vk == 0x22 /* VK_NEXT */
            {
                state.session.reset();
                if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
                return CallNextHookEx(0 as _, n_code, w_param, l_param);
            }

            // Tab / Shift+Tab / Down navigation
            if vk == VK_TAB || vk == VK_DOWN {
                if shift_down {
                    state.session.select_prev();
                    while state.session.get_selected_index() >= visible_count {
                        state.session.select_prev();
                    }
                } else {
                    state.session.select_next();
                    if state.session.get_selected_index() >= visible_count {
                        while state.session.get_selected_index() != 0 {
                            state.session.select_next();
                        }
                    }
                }
                state.update_candidate_window(
                    state.session.get_candidates(),
                    state.session.get_selected_index(),
                );
                return 1;
            }

            // Up navigation
            if vk == VK_UP {
                state.session.select_prev();
                while state.session.get_selected_index() >= visible_count {
                    state.session.select_prev();
                }
                state.update_candidate_window(
                    state.session.get_candidates(),
                    state.session.get_selected_index(),
                );
                return 1;
            }
        }
    }

    // Handle Backspace
    if vk == VK_BACK {
        if state.uncommitted_units > 0 {
            state.session.process_backspace();
            inject_backspaces(state.uncommitted_units);
            let candidate = state.session.get_preedit_text();
            if !candidate.is_empty() {
                state.uncommitted_units = inject_unicode_str(&candidate);
                if state.session.active_layout_type == ActiveLayoutType::Phonetic {
                    state.update_candidate_window(
                        state.session.get_candidates(),
                        state.session.get_selected_index(),
                    );
                }
            } else {
                state.uncommitted_units = 0;
                if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
            }
            return 1;
        } else if state.session.is_prediction_mode() {
            state.session.reset();
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Space
    if vk == VK_SPACE {
        if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.on_commit();
            state.uncommitted_units = 0;
            inject_unicode_str(" ");
            if state
                .config_mgr
                .config
                .phonetic
                .enable_predictive_next_words
            {
                if state.session.populate_predictions() {
                    let preds = state.session.get_candidates();
                    state.update_candidate_window(preds, 0);
                } else if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
            } else if let Some(ref win) = state.candidate_win {
                win.hide();
            }
            return 1;
        } else if state.session.is_prediction_mode() {
            if state.session.is_prediction_navigated() {
                let idx = state.session.get_selected_index();
                if let Some(committed) = state.session.commit(idx) {
                    state.on_commit();
                    inject_unicode_str(&committed);
                    inject_unicode_str(" ");
                    if state
                        .config_mgr
                        .config
                        .phonetic
                        .enable_predictive_next_words
                    {
                        if state.session.populate_predictions() {
                            let preds = state.session.get_candidates();
                            state.update_candidate_window(preds, 0);
                        } else if let Some(ref win) = state.candidate_win {
                            win.hide();
                        }
                    } else if let Some(ref win) = state.candidate_win {
                        win.hide();
                    }
                    return 1;
                }
            }
            state.session.reset();
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Enter
    if vk == VK_RETURN {
        if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.on_commit();
            state.uncommitted_units = 0;
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        } else if state.session.is_prediction_mode() {
            if state.session.is_prediction_navigated() {
                let idx = state.session.get_selected_index();
                if let Some(committed) = state.session.commit(idx) {
                    state.on_commit();
                    inject_unicode_str(&committed);
                    if let Some(ref win) = state.candidate_win {
                        win.hide();
                    }
                    return 1;
                }
            }
            state.session.reset();
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Escape
    if vk == VK_ESCAPE {
        if state.uncommitted_units > 0 || state.session.is_prediction_mode() {
            if state.uncommitted_units > 0 {
                inject_backspaces(state.uncommitted_units);
            }
            state.session.clear_context();
            state.uncommitted_units = 0;
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
            return 1;
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Standalone modifier keys (Shift, CapsLock, Ctrl, Alt, Win) must pass through
    // without committing or altering active uncommitted preedit text
    if is_modifier_key(vk) {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Translate Virtual Key to character
    if let Some(ch) = vk_to_char(vk, shift_down, caps_locked) {
        if state.session.is_prediction_mode() && state.uncommitted_units == 0 {
            state.session.reset();
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
        let keycode = state.mapper.map_keyval(ch as u32);
        let mut modifier_mask = 0;
        if shift_down {
            modifier_mask |= MODIFIER_SHIFT;
        }
        if is_alt_gr {
            modifier_mask |= MODIFIER_ALT_GR;
        }

        let consumed = state.session.process_key(keycode, modifier_mask);
        if consumed {
            let candidate = state.session.get_preedit_text();
            if !candidate.is_empty() {
                inject_backspaces(state.uncommitted_units);
                state.uncommitted_units = inject_unicode_str(&candidate);
                if state.session.active_layout_type == ActiveLayoutType::Phonetic {
                    state.update_candidate_window(
                        state.session.get_candidates(),
                        state.session.get_selected_index(),
                    );
                } else if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
                return 1;
            }
        } else if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.on_commit();
            state.uncommitted_units = 0;
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
    } else if state.uncommitted_units > 0 {
        state.session.commit(state.session.get_selected_index());
        state.on_commit();
        state.uncommitted_units = 0;
        if let Some(ref win) = state.candidate_win {
            win.hide();
        }
    }

    CallNextHookEx(0 as _, n_code, w_param, l_param)
}

fn vk_to_char(vk: u16, shift: bool, caps: bool) -> Option<char> {
    // 1. Try Win32 ToUnicode with active keyboard state for international layouts (UK, AZERTY, QWERTZ)
    unsafe {
        let mut key_state = [0u8; 256];
        if GetKeyboardState(key_state.as_mut_ptr()) != 0 {
            if shift {
                key_state[VK_SHIFT as usize] = 0x80;
            } else {
                key_state[VK_SHIFT as usize] = 0;
            }
            if caps {
                key_state[VK_CAPITAL as usize] = 0x01;
            } else {
                key_state[VK_CAPITAL as usize] = 0;
            }
            // Clear Ctrl/Alt so AltGr or Ctrl states don't suppress base character conversion
            key_state[VK_CONTROL as usize] = 0;
            key_state[VK_LCONTROL as usize] = 0;
            key_state[VK_RCONTROL as usize] = 0;
            key_state[VK_MENU as usize] = 0;
            key_state[VK_LMENU as usize] = 0;
            key_state[VK_RMENU as usize] = 0;

            let mut buf = [0u16; 4];
            let res = ToUnicode(vk as u32, 0, key_state.as_ptr(), buf.as_mut_ptr(), 4, 0);
            if res > 0 {
                if let Some(Ok(c)) = char::decode_utf16(buf[..res as usize].iter().cloned()).next()
                {
                    if !c.is_control() {
                        return Some(c);
                    }
                }
            }
        }
    }

    // 2. Fallback to standard US QWERTY mapping
    fallback_vk_to_char(vk, shift, caps)
}

fn fallback_vk_to_char(vk: u16, shift: bool, caps: bool) -> Option<char> {
    match vk {
        0x41..=0x5A => {
            let base = (vk - 0x41) as u8;
            let uppercase = shift ^ caps;
            if uppercase {
                Some((b'A' + base) as char)
            } else {
                Some((b'a' + base) as char)
            }
        }
        0x30..=0x39 => {
            if shift {
                let shift_digits = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
                let idx = (vk - 0x30) as usize;
                Some(shift_digits[idx])
            } else {
                let base = (vk - 0x30) as u8;
                Some((b'0' + base) as char)
            }
        }
        // Numpad 0..9
        0x60..=0x69 => {
            let base = (vk - 0x60) as u8;
            Some((b'0' + base) as char)
        }
        0x6A => Some('*'), // VK_MULTIPLY
        0x6B => Some('+'), // VK_ADD
        0x6D => Some('-'), // VK_SUBTRACT
        0x6E => Some('.'), // VK_DECIMAL
        0x6F => Some('/'), // VK_DIVIDE
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

fn is_modifier_key(vk: u16) -> bool {
    matches!(
        vk,
        VK_SHIFT | VK_CONTROL | VK_MENU | VK_LWIN | VK_RWIN | VK_CAPITAL | 0xA0..=0xA5 // VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU
    )
}

fn inject_backspaces(count: usize) {
    if count == 0 {
        return;
    }
    let mut inputs: Vec<INPUT> = Vec::with_capacity(count * 2);
    for _ in 0..count {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

fn inject_unicode_str(text: &str) -> usize {
    let units: Vec<u16> = text.encode_utf16().collect();
    let len = units.len();
    if len == 0 {
        return 0;
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(len * 2);
    for unit in &units {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: *unit,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: *unit,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }

    len
}
