//! Native Windows Global Low-Level Keyboard Hook & Text Injection Engine
//! Emulates the classic Avro Keyboard system-hook architecture for universal Windows compatibility.

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DOWN, VK_ESCAPE, VK_F12, VK_LWIN,
    VK_MENU, VK_NUMPAD1, VK_NUMPAD5, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6,
    VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_RETURN, VK_RWIN, VK_SHIFT,
    VK_SPACE, VK_TAB, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::win_candidate::CandidateWindow;
use crate::win_osd::OsdWindow;
use crate::win_tray::WindowsTray;
use lekhani_core::{ActiveLayoutType, InputSession, KeycodeMapper, MODIFIER_SHIFT};
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
        }
    }

    fn reset(&mut self) {
        self.session.clear_context();
        self.uncommitted_units = 0;
        if let Some(ref win) = self.candidate_win {
            win.hide();
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

pub fn toggle_bengali_mode() {
    let new_state = !BENGALI_ACTIVE.load(Ordering::SeqCst);
    set_bengali_mode(new_state);
}

pub fn set_bengali_mode(active: bool) {
    BENGALI_ACTIVE.store(active, Ordering::SeqCst);

    let mut layout_name = String::new();
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
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
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), hmod, 0);
        if hook != 0 as _ {
            *HOOK_HANDLE.lock().unwrap() = Some(hook as usize);
            tracing::info!("Windows Low-Level Keyboard Hook installed successfully");

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
                DispatchMessageW(&msg);
            }

            if let Some(h) = *HOOK_HANDLE.lock().unwrap() {
                UnhookWindowsHookEx(h as HHOOK);
            }
        } else {
            tracing::error!(
                "Failed to install Windows Low-Level Keyboard Hook: {}",
                windows_sys::Win32::Foundation::GetLastError()
            );
        }
    });
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

    let ctrl_down = (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0;
    let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
    let alt_down = (GetKeyState(VK_MENU as i32) & 0x8000u16 as i16) != 0;
    let win_down =
        ((GetKeyState(VK_LWIN as i32) | GetKeyState(VK_RWIN as i32)) & 0x8000u16 as i16) != 0;

    // Check global toggle hotkeys (F12, Ctrl+Space, and configurable Shift+Space)
    let is_toggle = if vk == VK_F12 && !ctrl_down && !alt_down && !win_down {
        true
    } else if vk == VK_SPACE && ctrl_down && !alt_down && !win_down {
        true
    } else if vk == VK_SPACE && shift_down && !ctrl_down && !alt_down && !win_down {
        let configured_shift_space = if let Ok(guard) = HOOK_STATE.lock() {
            guard.as_ref().map_or(false, |s| {
                s.config_mgr
                    .config
                    .general
                    .toggle_key
                    .eq_ignore_ascii_case("Shift+Space")
            })
        } else {
            false
        };
        configured_shift_space
    } else {
        false
    };

    if is_toggle {
        toggle_bengali_mode();
        return 1;
    }

    if !BENGALI_ACTIVE.load(Ordering::SeqCst) {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // If Ctrl, Alt, or Win are held down, let native shortcut pass through
    if ctrl_down || alt_down || win_down {
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

    let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
    let caps_locked = (GetKeyState(VK_CAPITAL as i32) & 1) != 0;

    // Direct Selection via 1..5 and Candidate Navigation in Phonetic Mode
    if state.session.active_layout_type == ActiveLayoutType::Phonetic && state.uncommitted_units > 0
    {
        let candidates = state.session.get_candidates();
        if !candidates.is_empty() {
            // Direct candidate selection with 1..5
            let sel_num = match vk {
                0x31..=0x35 if !shift_down => Some((vk - 0x31) as usize),
                VK_NUMPAD1..=VK_NUMPAD5 => Some((vk - VK_NUMPAD1) as usize),
                _ => None,
            };

            if let Some(idx) = sel_num {
                if idx < candidates.len() {
                    if let Some(committed) = state.session.commit(idx) {
                        inject_backspaces(state.uncommitted_units);
                        inject_unicode_str(&committed);
                        state.uncommitted_units = 0;
                        if state.config_mgr.config.phonetic.enable_predictive_next_words {
                            if state.session.populate_predictions() {
                                let preds = state.session.get_candidates();
                                if let Some(ref win) = state.candidate_win {
                                    win.update(preds, 0);
                                }
                            } else if let Some(ref win) = state.candidate_win {
                                win.hide();
                            }
                        } else if let Some(ref win) = state.candidate_win {
                            win.hide();
                        }
                        return 1;
                    }
                }
            }

            // Candidate navigation via Tab, Down, Up
            if vk == VK_TAB || vk == VK_DOWN {
                state.session.select_next();
                let candidate = state.session.get_preedit_text();
                if !candidate.is_empty() {
                    inject_backspaces(state.uncommitted_units);
                    state.uncommitted_units = inject_unicode_str(&candidate);
                }
                if let Some(ref win) = state.candidate_win {
                    win.update(
                        state.session.get_candidates(),
                        state.session.get_selected_index(),
                    );
                }
                return 1;
            }

            if vk == VK_UP {
                state.session.select_prev();
                let candidate = state.session.get_preedit_text();
                if !candidate.is_empty() {
                    inject_backspaces(state.uncommitted_units);
                    state.uncommitted_units = inject_unicode_str(&candidate);
                }
                if let Some(ref win) = state.candidate_win {
                    win.update(
                        state.session.get_candidates(),
                        state.session.get_selected_index(),
                    );
                }
                return 1;
            }
        }
    }

    // Direct Selection via 1..5 in Zero-Preedit Prediction Mode
    if state.session.active_layout_type == ActiveLayoutType::Phonetic
        && state.uncommitted_units == 0
        && state.session.is_prediction_mode()
    {
        let candidates = state.session.get_candidates();
        let sel_num = match vk {
            0x31..=0x35 if !shift_down => Some((vk - 0x31) as usize),
            VK_NUMPAD1..=VK_NUMPAD5 => Some((vk - VK_NUMPAD1) as usize),
            _ => None,
        };

        if let Some(idx) = sel_num {
            if idx < candidates.len() {
                if let Some(committed) = state.session.commit(idx) {
                    inject_unicode_str(&committed);
                    inject_unicode_str(" ");
                    if state.config_mgr.config.phonetic.enable_predictive_next_words {
                        if state.session.populate_predictions() {
                            let preds = state.session.get_candidates();
                            if let Some(ref win) = state.candidate_win {
                                win.update(preds, 0);
                            }
                        } else if let Some(ref win) = state.candidate_win {
                            win.hide();
                        }
                    } else if let Some(ref win) = state.candidate_win {
                        win.hide();
                    }
                    return 1;
                }
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
                    if let Some(ref win) = state.candidate_win {
                        win.update(
                            state.session.get_candidates(),
                            state.session.get_selected_index(),
                        );
                    }
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
            state.uncommitted_units = 0;
            inject_unicode_str(" ");
            if state.config_mgr.config.phonetic.enable_predictive_next_words {
                if state.session.populate_predictions() {
                    let preds = state.session.get_candidates();
                    if let Some(ref win) = state.candidate_win {
                        win.update(preds, 0);
                    }
                } else if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
            } else if let Some(ref win) = state.candidate_win {
                win.hide();
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

    // Handle Enter
    if vk == VK_RETURN {
        if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.uncommitted_units = 0;
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        } else if state.session.is_prediction_mode() {
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

    // Translate Virtual Key to character
    if let Some(ch) = vk_to_char(vk, shift_down, caps_locked) {
        if state.session.is_prediction_mode() && state.uncommitted_units == 0 {
            state.session.reset();
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
        let keycode = state.mapper.map_keyval(ch as u32);
        let modifier_mask = if shift_down { MODIFIER_SHIFT } else { 0 };

        let consumed = state.session.process_key(keycode, modifier_mask);
        if consumed {
            let candidate = state.session.get_preedit_text();
            if !candidate.is_empty() {
                inject_backspaces(state.uncommitted_units);
                state.uncommitted_units = inject_unicode_str(&candidate);
                if state.session.active_layout_type == ActiveLayoutType::Phonetic {
                    if let Some(ref win) = state.candidate_win {
                        win.update(
                            state.session.get_candidates(),
                            state.session.get_selected_index(),
                        );
                    }
                } else if let Some(ref win) = state.candidate_win {
                    win.hide();
                }
                return 1;
            }
        } else if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.uncommitted_units = 0;
            if let Some(ref win) = state.candidate_win {
                win.hide();
            }
        }
    } else if state.uncommitted_units > 0 {
        state.session.commit(state.session.get_selected_index());
        state.uncommitted_units = 0;
        if let Some(ref win) = state.candidate_win {
            win.hide();
        }
    }

    CallNextHookEx(0 as _, n_code, w_param, l_param)
}

fn vk_to_char(vk: u16, shift: bool, caps: bool) -> Option<char> {
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
