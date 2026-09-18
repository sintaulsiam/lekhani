//! Native Windows Floating Candidate / Suggestion Window
//! Displays live phonetic candidate suggestions (1..5) near the active text cursor.

#![cfg(windows)]

use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER, HDC, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetCursorPos, GetGUIThreadInfo, RegisterClassW, SetWindowPos,
    ShowWindow, CS_HREDRAW, CS_VREDRAW, GUITHREADINFO, HWND_TOPMOST, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, SW_HIDE, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

#[derive(Clone, Default)]
pub struct CandidateData {
    pub candidates: Vec<String>,
    pub selected_index: usize,
}

static CANDIDATE_DATA: Mutex<Option<CandidateData>> = Mutex::new(None);
static CANDIDATE_HWND: Mutex<Option<usize>> = Mutex::new(None);

pub struct CandidateWindow {
    pub hwnd: HWND,
}

unsafe impl Send for CandidateWindow {}
unsafe impl Sync for CandidateWindow {}

impl CandidateWindow {
    pub fn new() -> Option<Self> {
        unsafe {
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            let hmod = GetModuleHandleW(std::ptr::null());

            let class_name: Vec<u16> = "LekhaniCandidateWin\0".encode_utf16().collect();
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(candidate_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hmod,
                hIcon: 0 as _,
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                0,
                0,
                320,
                38,
                0 as _,
                0 as _,
                hmod,
                std::ptr::null(),
            );

            if hwnd == 0 as _ {
                return None;
            }

            *CANDIDATE_HWND.lock().unwrap() = Some(hwnd as usize);
            Some(Self { hwnd })
        }
    }

    pub fn update(&self, candidates: &[String], selected_index: usize) {
        if candidates.is_empty() {
            self.hide();
            return;
        }

        let display_cands: Vec<String> = candidates.iter().take(5).cloned().collect();
        {
            let mut data = CANDIDATE_DATA.lock().unwrap();
            *data = Some(CandidateData {
                candidates: display_cands.clone(),
                selected_index,
            });
        }

        unsafe {
            let count = display_cands.len().max(1);
            let item_width = 68;
            let total_width = (count as i32 * item_width) + 16;
            let height = 38;

            let mut pt = POINT { x: 0, y: 0 };
            let mut gui_info: GUITHREADINFO = std::mem::zeroed();
            gui_info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

            let has_caret = GetGUIThreadInfo(0, &mut gui_info) != 0
                && gui_info.hwndCaret != 0 as _
                && (gui_info.rcCaret.left != 0
                    || gui_info.rcCaret.top != 0
                    || gui_info.rcCaret.right != 0
                    || gui_info.rcCaret.bottom != 0);

            if has_caret {
                pt.x = gui_info.rcCaret.left;
                pt.y = gui_info.rcCaret.bottom + 4;
                ClientToScreen(gui_info.hwndCaret, &mut pt);
            } else {
                GetCursorPos(&mut pt);
                pt.y += 24;
            }

            // Screen boundary clamping
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
            };
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);

            if pt.x + total_width > screen_w - 8 {
                pt.x = (screen_w - total_width - 8).max(8);
            }
            if pt.x < 8 {
                pt.x = 8;
            }

            if pt.y + height > screen_h - 48 {
                pt.y = (pt.y - height - 32).max(8);
            }

            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                pt.x,
                pt.y,
                total_width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            InvalidateRect(self.hwnd, std::ptr::null(), 1);
        }
    }

    pub fn hide(&self) {
        {
            let mut data = CANDIDATE_DATA.lock().unwrap();
            *data = None;
        }
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

#[allow(dead_code)]
pub fn hide_candidate_window() {
    if let Ok(guard) = CANDIDATE_HWND.lock() {
        if let Some(hwnd) = *guard {
            unsafe {
                ShowWindow(hwnd as HWND, SW_HIDE);
            }
        }
    }
    if let Ok(mut data) = CANDIDATE_DATA.lock() {
        *data = None;
    }
}

unsafe extern "system" fn candidate_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        windows_sys::Win32::UI::WindowsAndMessaging::WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            paint_candidates(hwnd, hdc);

            EndPaint(hwnd, &ps);
            0
        }
        windows_sys::Win32::UI::WindowsAndMessaging::WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_candidates(hwnd: HWND, hdc: HDC) {
    let data = {
        let guard = CANDIDATE_DATA.lock().unwrap();
        guard.clone()
    };

    let Some(data) = data else {
        return;
    };

    let mut client_rect: RECT = std::mem::zeroed();
    windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

    // Background: #1e1e2e
    let bg_brush = CreateSolidBrush(0x002E1E1E);
    FillRect(hdc, &client_rect, bg_brush);
    DeleteObject(bg_brush as _);

    // Border: #45475a
    let border_brush = CreateSolidBrush(0x005A4745);
    windows_sys::Win32::Graphics::Gdi::FrameRect(hdc, &client_rect, border_brush);
    DeleteObject(border_brush as _);

    SetBkMode(hdc, TRANSPARENT as _);

    let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    let font = CreateFontW(
        -14,
        0,
        0,
        0,
        600,
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
    let old_font = SelectObject(hdc, font as _);

    let count = data.candidates.len().min(5);
    let item_width = 68;

    for i in 0..count {
        let item_rect = RECT {
            left: 8 + (i as i32 * item_width),
            top: 4,
            right: 8 + ((i as i32 + 1) * item_width) - 4,
            bottom: client_rect.bottom - 4,
        };

        if i == data.selected_index {
            // Highlight active pill: #89b4fa
            let active_brush = CreateSolidBrush(0x00FAB489);
            let old_brush = SelectObject(hdc, active_brush as _);
            RoundRect(
                hdc,
                item_rect.left,
                item_rect.top,
                item_rect.right,
                item_rect.bottom,
                8,
                8,
            );
            SelectObject(hdc, old_brush);
            DeleteObject(active_brush as _);

            // Active text color: #11111b (dark)
            SetTextColor(hdc, 0x001B1111);
        } else {
            // Inactive text color: #cdd6f4 (light)
            SetTextColor(hdc, 0x00F4D6CD);
        }

        let label = format!("{}. {}", i + 1, data.candidates[i]);
        let mut label_utf16: Vec<u16> = label.encode_utf16().collect();

        let mut text_rect = item_rect;
        DrawTextW(
            hdc,
            label_utf16.as_mut_ptr(),
            label_utf16.len() as i32,
            &mut text_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }

    SelectObject(hdc, old_font);
    DeleteObject(font as _);
}
