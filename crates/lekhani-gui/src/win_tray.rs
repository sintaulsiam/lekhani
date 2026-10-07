//! Native Windows System Tray Support (Shell_NotifyIconW)
//!
//! Provides a responsive system tray icon with dynamic language badges ([বা] / [En]),
//! quick layout switching submenu (Avro, Flow, National, Probhat), TopBar visibility
//! toggle, and settings navigation.

#![cfg(windows)]

use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, RoundRect, SelectObject,
    SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE, DT_VCENTER, TRANSPARENT,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics,
    LoadIconW, PostMessageW, PostQuitMessage, RegisterClassExW, RegisterWindowMessageW,
    SetForegroundWindow, TrackPopupMenu, HICON, ICONINFO, IDI_APPLICATION, MF_CHECKED, MF_POPUP,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSMICON, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW,
};

const WM_TRAYICON: u32 = WM_APP + 1;
const ID_TRAY_RESTORE: usize = 1000;
const ID_TRAY_TOGGLE: usize = 1001;
const ID_TRAY_SETTINGS: usize = 1002;
const ID_TRAY_EXIT: usize = 1003;

const ID_LAYOUT_AVRO: usize = 1100;
const ID_LAYOUT_PROBAHO: usize = 1101;
const ID_LAYOUT_NATIONAL: usize = 1102;
const ID_LAYOUT_PROBHAT: usize = 1103;

static IS_BENGALI: AtomicBool = AtomicBool::new(false);
static IS_TOPBAR_VISIBLE: AtomicBool = AtomicBool::new(true);
static APP_WEAK: Mutex<Option<slint::Weak<crate::TopBarWindow>>> = Mutex::new(None);
static ACTIVE_LAYOUT: Mutex<String> = Mutex::new(String::new());
static WM_TASKBARCREATED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

static ICON_DEFAULT: Mutex<Option<usize>> = Mutex::new(None);
static ICON_EN: Mutex<Option<usize>> = Mutex::new(None);
static ICON_BN: Mutex<Option<usize>> = Mutex::new(None);

pub struct WindowsTray {
    hwnd: HWND,
}

unsafe impl Send for WindowsTray {}
unsafe impl Sync for WindowsTray {}

impl WindowsTray {
    pub fn new(initial_layout: String) -> Option<Self> {
        let class_name: Vec<u16> = "LekhaniTrayClass\0".encode_utf16().collect();

        unsafe {
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            let hmod = GetModuleHandleW(std::ptr::null());
            let app_icon = LoadIconW(hmod, 1 as _);
            let hicon_default = if app_icon != 0 as _ {
                app_icon
            } else {
                LoadIconW(0 as _, IDI_APPLICATION)
            };

            // Pre-create dynamic badge icons for English and Bengali
            let icon_sz = GetSystemMetrics(SM_CXSMICON).max(16);
            let mut hicon_bn = create_badge_icon("বা", 0x0081B910, 0x00FFFFFF, icon_sz, true);
            let mut hicon_en = create_badge_icon("En", 0x00F6823B, 0x00FFFFFF, icon_sz, false);
            if hicon_bn == 0 as _ {
                hicon_bn = hicon_default;
            }
            if hicon_en == 0 as _ {
                hicon_en = hicon_default;
            }

            *ICON_DEFAULT.lock().unwrap() = Some(hicon_default as usize);
            *ICON_EN.lock().unwrap() = Some(hicon_en as usize);
            *ICON_BN.lock().unwrap() = Some(hicon_bn as usize);
            *ACTIVE_LAYOUT.lock().unwrap() = initial_layout.clone();

            let taskbar_str: Vec<u16> = "TaskbarCreated\0".encode_utf16().collect();
            let tb_msg = RegisterWindowMessageW(taskbar_str.as_ptr());
            WM_TASKBARCREATED.store(tb_msg, Ordering::SeqCst);

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(tray_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hmod,
                hIcon: hicon_en,
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: hicon_en,
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
                0 as _, // Top-level unmapped window (required for Shell_NotifyIconW notifications)
                0 as _,
                hmod,
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
                hIcon: hicon_en,
                szTip: [0; 128],
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: std::mem::zeroed(),
                szInfoTitle: [0; 64],
                dwInfoFlags: 1, // NIIF_INFO
                guidItem: std::mem::zeroed(),
                hBalloonIcon: 0 as _,
            };

            let tip = format!("Lekhani [English - {}]\0", initial_layout);
            for (i, c) in tip.encode_utf16().enumerate() {
                if i < 127 {
                    nid.szTip[i] = c;
                }
            }

            let info_title = "Lekhani is Ready (লেখনী প্রস্তুত)\0";
            for (i, c) in info_title.encode_utf16().enumerate() {
                if i < 63 {
                    nid.szInfoTitle[i] = c;
                }
            }

            let info_msg = "Press F12 to toggle English / Bengali. Use Tab to cycle suggestions. Right-click for layouts.\0";
            for (i, c) in info_msg.encode_utf16().enumerate() {
                if i < 255 {
                    nid.szInfo[i] = c;
                }
            }

            // Shell_NotifyIconW(NIM_ADD) must NOT include NIF_INFO on Windows 7/10/11!
            let mut ok = Shell_NotifyIconW(NIM_ADD, &nid);
            if ok == 0 {
                tracing::warn!("First NIM_ADD failed, retrying with default application icon...");
                nid.hIcon = hicon_default;
                ok = Shell_NotifyIconW(NIM_ADD, &nid);
            }

            if ok == 0 {
                tracing::error!(
                    "Shell_NotifyIconW(NIM_ADD) failed: {}",
                    windows_sys::Win32::Foundation::GetLastError()
                );
            } else {
                tracing::info!("Windows System Tray Icon registered successfully");
                // Optional welcome balloon notification via NIM_MODIFY
                let mut nid_info = nid;
                nid_info.uFlags |= 0x00000010; // NIF_INFO
                let _ = Shell_NotifyIconW(NIM_MODIFY, &nid_info);
            }

            Some(Self { hwnd })
        }
    }

    pub fn set_topbar_visible(&self, visible: bool) {
        IS_TOPBAR_VISIBLE.store(visible, Ordering::SeqCst);
    }

    pub fn set_bengali_active(&self, active: bool, layout_name: &str) {
        IS_BENGALI.store(active, Ordering::SeqCst);
        *ACTIVE_LAYOUT.lock().unwrap() = layout_name.to_string();

        let icon = if active {
            ICON_BN.lock().unwrap().unwrap_or(0) as HICON
        } else {
            ICON_EN.lock().unwrap().unwrap_or(0) as HICON
        };

        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                uFlags: NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: icon,
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

            let mode_str = if active {
                "বাংলা (Active)"
            } else {
                "English"
            };
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

            if let Some(hicon) = *ICON_BN.lock().unwrap() {
                DestroyIcon(hicon as _);
            }
            if let Some(hicon) = *ICON_EN.lock().unwrap() {
                DestroyIcon(hicon as _);
            }
        }
    }
}

unsafe fn create_badge_icon(
    text: &str,
    bg_color: u32,
    fg_color: u32,
    size: i32,
    is_bengali: bool,
) -> HICON {
    let hdc_screen = GetDC(0 as _);
    if hdc_screen == 0 as _ {
        return 0 as _;
    }
    let hdc_mem = CreateCompatibleDC(hdc_screen);
    if hdc_mem == 0 as _ {
        ReleaseDC(0 as _, hdc_screen);
        return 0 as _;
    }
    let hbm_color = CreateCompatibleBitmap(hdc_screen, size, size);
    if hbm_color == 0 as _ {
        DeleteDC(hdc_mem);
        ReleaseDC(0 as _, hdc_screen);
        return 0 as _;
    }
    let old_bmp = SelectObject(hdc_mem, hbm_color as _);

    // Clear and fill badge with solid color (eliminating uninitialized memory)
    let brush = CreateSolidBrush(bg_color);
    let old_brush = SelectObject(hdc_mem, brush as _);
    let full_rect = RECT {
        left: 0,
        top: 0,
        right: size,
        bottom: size,
    };
    FillRect(hdc_mem, &full_rect, brush);
    RoundRect(hdc_mem, 0, 0, size, size, 6, 6);
    SelectObject(hdc_mem, old_brush);
    DeleteObject(brush as _);

    SetBkMode(hdc_mem, TRANSPARENT as _);
    SetTextColor(hdc_mem, fg_color);

    let font_name: Vec<u16> = if is_bengali {
        "Nirmala UI\0".encode_utf16().collect()
    } else {
        "Segoe UI\0".encode_utf16().collect()
    };
    let font_size = if is_bengali {
        -((size * 7) / 10).max(10)
    } else {
        -((size * 6) / 10).max(9)
    };

    let font = CreateFontW(
        font_size,
        0,
        0,
        0,
        700,
        0,
        0,
        0,
        1,
        0,
        0,
        0,
        0,
        font_name.as_ptr(),
    );
    let old_font = SelectObject(hdc_mem, font as _);

    let mut text_utf16: Vec<u16> = text.encode_utf16().collect();
    let mut draw_rect = RECT {
        left: 0,
        top: if is_bengali { -1 } else { 0 },
        right: size,
        bottom: size,
    };
    DrawTextW(
        hdc_mem,
        text_utf16.as_mut_ptr(),
        text_utf16.len() as i32,
        &mut draw_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    SelectObject(hdc_mem, old_font);
    DeleteObject(font as _);
    SelectObject(hdc_mem, old_bmp);
    DeleteDC(hdc_mem);
    ReleaseDC(0 as _, hdc_screen);

    // Create 1-bit monochrome mask initialized to 0s (all 0s = 100% opaque badge)
    let mask_pitch = ((size + 15) / 16) * 2;
    let mask_bytes = vec![0u8; (mask_pitch * size) as usize];
    let hbm_mask = CreateBitmap(size, size, 1, 1, mask_bytes.as_ptr() as _);

    let icon_info = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };
    let hicon = CreateIconIndirect(&icon_info);

    DeleteObject(hbm_mask as _);
    DeleteObject(hbm_color as _);

    hicon
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

unsafe fn update_tray_tooltip(hwnd: HWND, active: bool, layout_name: &str) {
    let mode_str = if active {
        "বাংলা (Active)"
    } else {
        "English"
    };
    let tip = format!("Lekhani [{} - {}]\0", mode_str, layout_name);

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
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
    for (i, c) in tip.encode_utf16().enumerate() {
        if i < 127 {
            nid.szTip[i] = c;
        }
    }
    Shell_NotifyIconW(NIM_MODIFY, &nid);
}

unsafe fn switch_layout_from_tray(hwnd: HWND, new_layout: &str) {
    *ACTIVE_LAYOUT.lock().unwrap() = new_layout.to_string();
    crate::win_hook::update_active_layout(new_layout);
    crate::win_hook::set_bengali_mode(true);

    // Save to user configuration
    let mut cfg_mgr = lekhani_settings::ConfigManager::new();
    cfg_mgr.config.general.active_layout = new_layout.to_string();
    let _ = cfg_mgr.save();

    // Sync Slint TopBar UI
    let layout_str = new_layout.to_string();
    let _ = slint::invoke_from_event_loop(move || {
        if let Ok(guard) = APP_WEAK.lock() {
            if let Some(ref weak) = *guard {
                if let Some(app) = weak.upgrade() {
                    app.set_is_bengali_mode(true);
                    app.set_active_layout_name(layout_str.into());
                }
            }
        }
    });

    update_tray_tooltip(hwnd, true, new_layout);
}

unsafe fn show_tray_menu(hwnd: HWND) {
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

    // Layout submenu
    let hsub_layout = CreatePopupMenu();
    let cur_layout = ACTIVE_LAYOUT.lock().unwrap().clone();

    let is_avro = cur_layout == "Avro Phonetic" || cur_layout.is_empty();
    let is_probaho = cur_layout.contains("Flow") || cur_layout.contains("প্রবাহ");
    let is_national = cur_layout.contains("National") || cur_layout.contains("জাতীয়");
    let is_probhat = cur_layout.contains("Probhat") || cur_layout.contains("प्रभात");

    let avro_label: Vec<u16> = "Avro Phonetic\0".encode_utf16().collect();
    let probaho_label: Vec<u16> = "Lekhani প্রবাহ (Flow)\0".encode_utf16().collect();
    let national_label: Vec<u16> = "জাতীয় (National)\0".encode_utf16().collect();
    let probhat_label: Vec<u16> = "Probhat (प्रभात)\0".encode_utf16().collect();

    AppendMenuW(
        hsub_layout,
        MF_STRING | if is_avro { MF_CHECKED } else { MF_UNCHECKED },
        ID_LAYOUT_AVRO,
        avro_label.as_ptr(),
    );
    AppendMenuW(
        hsub_layout,
        MF_STRING | if is_probaho { MF_CHECKED } else { MF_UNCHECKED },
        ID_LAYOUT_PROBAHO,
        probaho_label.as_ptr(),
    );
    AppendMenuW(
        hsub_layout,
        MF_STRING
            | if is_national {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            },
        ID_LAYOUT_NATIONAL,
        national_label.as_ptr(),
    );
    AppendMenuW(
        hsub_layout,
        MF_STRING | if is_probhat { MF_CHECKED } else { MF_UNCHECKED },
        ID_LAYOUT_PROBHAT,
        probhat_label.as_ptr(),
    );

    let layout_sub_title: Vec<u16> = "Keyboard Layout\0".encode_utf16().collect();
    AppendMenuW(
        hmenu,
        MF_POPUP,
        hsub_layout as usize,
        layout_sub_title.as_ptr(),
    );

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
    PostMessageW(hwnd, 0, 0, 0); // WM_NULL (mandatory per Win32 tray documentation)
    DestroyMenu(hmenu);
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let taskbar_msg = WM_TASKBARCREATED.load(Ordering::Relaxed);
    if taskbar_msg != 0 && msg == taskbar_msg {
        // Re-register tray icon if explorer restarted
        let is_bn = IS_BENGALI.load(Ordering::SeqCst);
        let cur_layout = ACTIVE_LAYOUT.lock().unwrap().clone();
        let icon = if is_bn {
            ICON_BN.lock().unwrap().unwrap_or(0) as HICON
        } else {
            ICON_EN.lock().unwrap().unwrap_or(0) as HICON
        };
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: icon,
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
        let tip = format!("Lekhani [{}]\0", cur_layout);
        for (i, c) in tip.encode_utf16().enumerate() {
            if i < 127 {
                nid.szTip[i] = c;
            }
        }
        Shell_NotifyIconW(NIM_ADD, &nid);
        return 0;
    }

    match msg {
        WM_TRAYICON => {
            let event = (lparam as u32) & 0xFFFF;
            if event == WM_RBUTTONUP || event == 0x007B
            /* WM_CONTEXTMENU */
            {
                show_tray_menu(hwnd);
            } else if event == WM_LBUTTONUP {
                // Left click gives options if topbar is visible, or restores topbar if hidden
                if !IS_TOPBAR_VISIBLE.load(Ordering::SeqCst) {
                    restore_topbar();
                } else {
                    show_tray_menu(hwnd);
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
                ID_LAYOUT_AVRO => {
                    switch_layout_from_tray(hwnd, "Avro Phonetic");
                }
                ID_LAYOUT_PROBAHO => {
                    switch_layout_from_tray(hwnd, "Lekhani প্রবাহ (Flow)");
                }
                ID_LAYOUT_NATIONAL => {
                    switch_layout_from_tray(hwnd, "জাতীয় (National)");
                }
                ID_LAYOUT_PROBHAT => {
                    switch_layout_from_tray(hwnd, "Probhat (प्रभात)");
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

    let (tx, rx) = std::sync::mpsc::channel();
    let initial_layout = active_layout.clone();

    std::thread::spawn(move || {
        let tray_opt = WindowsTray::new(initial_layout);
        if let Some(tray) = tray_opt {
            let tray_arc = Arc::new(tray);
            let _ = tx.send(Some(Arc::clone(&tray_arc)));

            let mut msg: MSG = unsafe { std::mem::zeroed() };
            unsafe {
                while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
                    DispatchMessageW(&msg);
                }
            }
            drop(tray_arc);
        } else {
            let _ = tx.send(None);
        }
    });

    rx.recv().ok().flatten()
}
