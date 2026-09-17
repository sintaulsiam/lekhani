//! Native Windows Global Low-Level Keyboard Hook & Text Injection Engine
//! Emulates the classic Avro Keyboard system-hook architecture for universal Windows compatibility.

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_ESCAPE, VK_F12, VK_LWIN, VK_MENU,
    VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA,
    VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_RETURN, VK_RWIN, VK_SHIFT, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::win_tray::WindowsTray;
use lekhani_core::{ActiveLayoutType, InputSession, KeycodeMapper, MODIFIER_SHIFT};
use lekhani_settings::{ConfigManager, LayoutManager};

static BENGALI_ACTIVE: AtomicBool = AtomicBool::new(false);

struct HookState {
    session: InputSession,
    mapper: KeycodeMapper,
    uncommitted_units: usize,
    tray: Option<Arc<WindowsTray>>,
    active_layout_name: String,
    layout_mgr: LayoutManager,
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
        }
    }

    fn reset(&mut self) {
        self.session.clear_context();
        self.uncommitted_units = 0;
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
    BENGALI_ACTIVE.store(new_state, Ordering::SeqCst);

    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            state.reset();
            if let Some(ref tray) = state.tray {
                tray.set_bengali_active(new_state, &state.active_layout_name);
            }
        }
    }
}

pub fn update_active_layout(name: &str) {
    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(ref mut state) = *guard {
            state.set_layout(name);
            if let Some(ref tray) = state.tray {
                tray.set_bengali_active(BENGALI_ACTIVE.load(Ordering::SeqCst), name);
            }
        }
    }
}

pub fn spawn_windows_hook(initial_layout: String, tray: Option<Arc<WindowsTray>>) {
    {
        let mut state = HookState::new(initial_layout.clone());
        state.tray = tray;
        let mut guard = HOOK_STATE.lock().unwrap();
        *guard = Some(state);
    }

    std::thread::spawn(|| {
        unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0 as _, 0);
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
                tracing::error!("Failed to install Windows Low-Level Keyboard Hook");
            }
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

    // F12 Global Hotkey toggles Bengali mode
    if vk == VK_F12 {
        toggle_bengali_mode();
        return 1;
    }

    if !BENGALI_ACTIVE.load(Ordering::SeqCst) {
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // If Ctrl, Alt, or Win are held down, let native shortcut pass through
    let ctrl_down = (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0;
    let alt_down = (GetKeyState(VK_MENU as i32) & 0x8000u16 as i16) != 0;
    let win_down = ((GetKeyState(VK_LWIN as i32) | GetKeyState(VK_RWIN as i32))
        & 0x8000u16 as i16)
        != 0;

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

    // Handle Backspace
    if vk == VK_BACK {
        if state.uncommitted_units > 0 {
            state.session.process_backspace();
            inject_backspaces(state.uncommitted_units);
            let candidate = state.session.get_preedit_text();
            if !candidate.is_empty() {
                state.uncommitted_units = inject_unicode_str(&candidate);
            } else {
                state.uncommitted_units = 0;
            }
            return 1;
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Space
    if vk == VK_SPACE {
        if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.uncommitted_units = 0;
            inject_unicode_str(" ");
            return 1;
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Enter
    if vk == VK_RETURN {
        if state.uncommitted_units > 0 {
            state.session.commit(state.session.get_selected_index());
            state.uncommitted_units = 0;
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Handle Escape
    if vk == VK_ESCAPE {
        if state.uncommitted_units > 0 {
            inject_backspaces(state.uncommitted_units);
            state.session.clear_context();
            state.uncommitted_units = 0;
            return 1;
        }
        return CallNextHookEx(0 as _, n_code, w_param, l_param);
    }

    // Translate Virtual Key to character
    let shift_down = (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0;
    let caps_locked = (GetKeyState(VK_CAPITAL as i32) & 1) != 0;

    if let Some(ch) = vk_to_char(vk, shift_down, caps_locked) {
        let keycode = state.mapper.map_keyval(ch as u32);
        let modifier_mask = if shift_down { MODIFIER_SHIFT } else { 0 };

        match state.session.active_layout_type {
            ActiveLayoutType::Phonetic => {
                let consumed = state.session.process_key(keycode, modifier_mask);
                if consumed {
                    let candidate = state.session.get_preedit_text();
                    if !candidate.is_empty() {
                        inject_backspaces(state.uncommitted_units);
                        state.uncommitted_units = inject_unicode_str(&candidate);
                        return 1;
                    }
                }
            }
            ActiveLayoutType::Fixed => {
                if state.session.process_key(keycode, modifier_mask) {
                    let committed = state.session.fixed.commit();
                    if !committed.is_empty() {
                        inject_unicode_str(&committed);
                        return 1;
                    }
                }
            }
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
