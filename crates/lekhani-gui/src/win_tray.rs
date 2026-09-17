//! Native Windows System Tray Support (Shell_NotifyIconW)

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, TrackPopupMenu, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP,
    WNDCLASSEXW,
};

const WM_TRAYICON: u32 = WM_APP + 1;
const ID_TRAY_TOGGLE: usize = 1001;
const ID_TRAY_SETTINGS: usize = 1002;
const ID_TRAY_EXIT: usize = 1003;

static IS_BENGALI: AtomicBool = AtomicBool::new(false);

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
                let toggle_label: Vec<u16> = if IS_BENGALI.load(Ordering::SeqCst) {
                    "Switch to English (F12)\0".encode_utf16().collect()
                } else {
                    "Switch to Bengali (F12)\0".encode_utf16().collect()
                };
                let settings_label: Vec<u16> = "Settings...\0".encode_utf16().collect();
                let exit_label: Vec<u16> = "Exit Lekhani\0".encode_utf16().collect();

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
                // Left click toggles language state
                crate::win_hook::toggle_bengali_mode();
            }
            0
        }
        WM_COMMAND => {
            let id = wparam as usize;
            match id {
                ID_TRAY_TOGGLE => {
                    crate::win_hook::toggle_bengali_mode();
                }
                ID_TRAY_EXIT => {
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

pub fn spawn_windows_tray(active_layout: String) -> Option<Arc<WindowsTray>> {
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
