//! Native Windows System Tray Support (Shell_NotifyIconW)

#![cfg(windows)]

use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, TrackPopupMenu, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW,
};

const WM_TRAYICON: u32 = WM_APP + 1;
const ID_TRAY_RESTORE: usize = 1000;
const ID_TRAY_TOGGLE: usize = 1001;
const ID_TRAY_SETTINGS: usize = 1002;
const ID_TRAY_EXIT: usize = 1003;

static IS_BENGALI: AtomicBool = AtomicBool::new(false);
static IS_TOPBAR_VISIBLE: AtomicBool = AtomicBool::new(true);
static APP_WEAK: Mutex<Option<slint::Weak<crate::TopBarWindow>>> = Mutex::new(None);

pub struct WindowsTray {
    hwnd: HWND,
}

unsafe impl Send for WindowsTray {}
unsafe impl Sync for WindowsTray {}

impl WindowsTray {
    pub fn new(initial_layout: String) -> Option<Self> {
        let class_name: Vec<u16> = "LekhaniTrayClass\0".encode_utf16().collect();

        unsafe {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(tray_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0 as _,
                hIcon: LoadIconW(0 as _, IDI_APPLICATION),
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: LoadIconW(0 as _, IDI_APPLICATION),
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                windows_sys::Win32::UI::WindowsAndMessaging::HWND_MESSAGE,
                0 as _,
                0 as _,
                std::ptr::null(),
            );

            if hwnd == 0 as _ {
                return None;
            }

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: LoadIconW(0 as _, IDI_APPLICATION),
                szTip: [0; 128],
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: std::mem::zeroed(),
                szInfoTitle: [0; 64],
                dwInfoFlags: 0,
                guidItem: std::mem::zeroed(),
                hBalloonIcon: 0 as _,
            };

            let tip = format!("Lekhani ({})\0", initial_layout);
            for (i, c) in tip.encode_utf16().enumerate() {
                if i < 127 {
                    nid.szTip[i] = c;
                }
            }

            Shell_NotifyIconW(NIM_ADD, &nid);

            Some(Self { hwnd })
        }
    }

    pub fn set_topbar_visible(&self, visible: bool) {
        IS_TOPBAR_VISIBLE.store(visible, Ordering::SeqCst);
    }

    pub fn set_bengali_active(&self, active: bool, layout_name: &str) {
        IS_BENGALI.store(active, Ordering::SeqCst);
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                uFlags: NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: 0 as _,
                szTip: [0; 128],
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: std::mem::zeroed(),
                szInfoTitle: [0; 64],
                dwInfoFlags: 0,
                guidItem: std::mem::zeroed(),
                hBalloonIcon: 0 as _,
            };

            let mode_str = if active { "বাংলা" } else { "ENG" };
            let tip = format!("Lekhani [{} - {}]\0", mode_str, layout_name);
            for (i, c) in tip.encode_utf16().enumerate() {
                if i < 127 {
                    nid.szTip[i] = c;
                }
            }

            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }
}

impl Drop for WindowsTray {
    fn drop(&mut self) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                uFlags: 0,
                uCallbackMessage: 0,
                hIcon: 0 as _,
                szTip: [0; 128],
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: std::mem::zeroed(),
                szInfoTitle: [0; 64],
                dwInfoFlags: 0,
                guidItem: std::mem::zeroed(),
                hBalloonIcon: 0 as _,
            };
            Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(self.hwnd);
        }
    }
}

pub fn restore_topbar() {
    IS_TOPBAR_VISIBLE.store(true, Ordering::SeqCst);
    if let Ok(guard) = APP_WEAK.lock() {
        if let Some(ref weak) = *guard {
            let weak_clone = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak_clone.upgrade() {
                    use i_slint_backend_winit::WinitWindowAccessor;
                    let _ = app.window().with_winit_window(|w| {
                        w.set_visible(true);
                        w.set_minimized(false);
                        w.focus_window();
                    });
                    let _ = app.show();
                }
            });
        }
    }
}

pub fn hide_topbar() {
    IS_TOPBAR_VISIBLE.store(false, Ordering::SeqCst);
    if let Ok(guard) = APP_WEAK.lock() {
        if let Some(ref weak) = *guard {
            let weak_clone = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak_clone.upgrade() {
                    use i_slint_backend_winit::WinitWindowAccessor;
                    let _ = app.window().with_winit_window(|w| {
                        w.set_visible(false);
                    });
                }
            });
        }
    }
}

pub fn toggle_topbar_visibility() {
    if IS_TOPBAR_VISIBLE.load(Ordering::SeqCst) {
        hide_topbar();
    } else {
        restore_topbar();
    }
}

pub fn open_settings() {
    IS_TOPBAR_VISIBLE.store(true, Ordering::SeqCst);
    if let Ok(guard) = APP_WEAK.lock() {
        if let Some(ref weak) = *guard {
            let weak_clone = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak_clone.upgrade() {
                    use i_slint_backend_winit::WinitWindowAccessor;
                    let _ = app.window().with_winit_window(|w| {
                        w.set_visible(true);
                        w.set_minimized(false);
                        w.focus_window();
                    });
                    let _ = app.show();
                    app.set_active_dialog(5);
                }
            });
        }
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);

                let hmenu = CreatePopupMenu();
                let restore_label: Vec<u16> = if IS_TOPBAR_VISIBLE.load(Ordering::SeqCst) {
                    "Hide TopBar\0".encode_utf16().collect()
                } else {
                    "Show TopBar\0".encode_utf16().collect()
                };
                let toggle_label: Vec<u16> = if IS_BENGALI.load(Ordering::SeqCst) {
                    "Switch to English (F12)\0".encode_utf16().collect()
                } else {
                    "Switch to Bengali (F12)\0".encode_utf16().collect()
                };
                let settings_label: Vec<u16> = "Settings...\0".encode_utf16().collect();
                let exit_label: Vec<u16> = "Exit Lekhani\0".encode_utf16().collect();

                AppendMenuW(hmenu, MF_STRING, ID_TRAY_RESTORE, restore_label.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(hmenu, MF_STRING, ID_TRAY_TOGGLE, toggle_label.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(hmenu, MF_STRING, ID_TRAY_SETTINGS, settings_label.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, exit_label.as_ptr());

                SetForegroundWindow(hwnd);
                TrackPopupMenu(
                    hmenu,
                    TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    std::ptr::null(),
                );
                DestroyMenu(hmenu);
            } else if event == WM_LBUTTONUP {
                // If topbar is hidden/minimized to tray, left-click restores it!
                // If topbar is already visible, left-click toggles language state.
                if !IS_TOPBAR_VISIBLE.load(Ordering::SeqCst) {
                    restore_topbar();
                } else {
                    crate::win_hook::toggle_bengali_mode();
                }
            } else if event == WM_LBUTTONDBLCLK {
                // Double-clicking tray icon always restores TopBar
                restore_topbar();
            }
            0
        }
        WM_COMMAND => {
            let id = wparam as usize;
            match id {
                ID_TRAY_RESTORE => {
                    toggle_topbar_visibility();
                }
                ID_TRAY_TOGGLE => {
                    crate::win_hook::toggle_bengali_mode();
                }
                ID_TRAY_SETTINGS => {
                    open_settings();
                }
                ID_TRAY_EXIT => {
                    let _ = slint::invoke_from_event_loop(|| {
                        let _ = slint::quit_event_loop();
                    });
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    std::process::exit(0);
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn spawn_windows_tray(
    active_layout: String,
    app_weak: slint::Weak<crate::TopBarWindow>,
) -> Option<Arc<WindowsTray>> {
    *APP_WEAK.lock().unwrap() = Some(app_weak);
    IS_TOPBAR_VISIBLE.store(true, Ordering::SeqCst);

    let tray = WindowsTray::new(active_layout)?;
    let tray_arc = Arc::new(tray);
    let tray_clone = Arc::clone(&tray_arc);

    std::thread::spawn(move || {
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        unsafe {
            while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
                DispatchMessageW(&msg);
            }
        }
        drop(tray_clone);
    });

    Some(tray_arc)
}
