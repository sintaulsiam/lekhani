//! Native Windows Floating Candidate / Suggestion Window
//! Displays live phonetic candidate suggestions (1..5) near the active text cursor with
//! dynamic width scaling, Indic complex font shaping (Nirmala UI), and mouse interactivity.

#![cfg(windows)]

use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetDC, GetTextExtentPoint32W, InvalidateRect, ReleaseDC, RoundRect, SelectObject,
    SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetCursorPos, GetForegroundWindow, GetGUIThreadInfo,
    GetSystemMetrics, GetWindowRect, RegisterClassW, SetWindowPos, ShowWindow, CS_DROPSHADOW,
    CS_HREDRAW, CS_VREDRAW, GUITHREADINFO, HWND_TOPMOST, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, WM_ERASEBKGND, WM_LBUTTONUP, WM_PAINT, WNDCLASSW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};


#[derive(Clone, Default)]
pub struct CandidateData {
    pub candidates: Vec<String>,
    pub item_widths: Vec<i32>,
    pub selected_index: usize,
    pub horizontal: bool,
    pub total_width: i32,
    pub height: i32,
    pub screen_pos: (i32, i32),
    pub scale: f32,
}

static CANDIDATE_DATA: Mutex<Option<CandidateData>> = Mutex::new(None);
static CANDIDATE_HWND: Mutex<Option<usize>> = Mutex::new(None);

unsafe fn get_window_dpi(hwnd: HWND) -> u32 {
    use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSY};
    let hdc = GetDC(hwnd);
    if hdc != 0 as _ {
        let dpi = GetDeviceCaps(hdc, LOGPIXELSY as _);
        ReleaseDC(hwnd, hdc);
        if dpi > 0 {
            return dpi as u32;
        }
    }
    96
}

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
                style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
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

    pub fn is_visible(&self) -> bool {
        let guard = CANDIDATE_DATA.lock().unwrap();
        guard.as_ref().map_or(false, |d| !d.candidates.is_empty())
    }

    pub fn hit_test(&self, screen_x: i32, screen_y: i32) -> Option<usize> {
        let guard = CANDIDATE_DATA.lock().unwrap();
        let data = guard.as_ref()?;
        hit_test_data(data, screen_x, screen_y)
    }

    pub fn update(&self, candidates: &[String], selected_index: usize, horizontal: bool) {
        if candidates.is_empty() {
            self.hide();
            return;
        }

        let scale = unsafe { (get_window_dpi(self.hwnd) as f32 / 96.0).max(1.0) };
        let display_cands: Vec<String> = candidates.iter().take(6).cloned().collect();
        let count = display_cands.len().max(1);

        // Dynamically compute precise width per candidate using GDI font metrics (GetTextExtentPoint32W)
        // scaled to monitor DPI to prevent clipping complex Bengali conjuncts/juktoborno.
        let mut item_widths = Vec::with_capacity(count);
        unsafe {
            let hdc_screen = GetDC(0 as _);
            let font_name: Vec<u16> = "Nirmala UI\0".encode_utf16().collect();
            let font_size = (-15.0 * scale).round() as i32;
            let font = CreateFontW(
                font_size,
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
            let old_font = SelectObject(hdc_screen, font as _);

            for (i, c) in display_cands.iter().enumerate() {
                let label = format!("{}. {}", i + 1, c);
                let label_utf16: Vec<u16> = label.encode_utf16().collect();
                let mut sz: SIZE = std::mem::zeroed();
                GetTextExtentPoint32W(
                    hdc_screen,
                    label_utf16.as_ptr(),
                    label_utf16.len() as i32,
                    &mut sz,
                );
                let min_w = (68.0 * scale).round() as i32;
                let pad = (20.0 * scale).round() as i32;
                let w = (sz.cx + pad).max(min_w);
                item_widths.push(w);
            }

            SelectObject(hdc_screen, old_font);
            DeleteObject(font as _);
            ReleaseDC(0 as _, hdc_screen);
        }

        let (total_width, height) = if horizontal {
            let sum_w: i32 = item_widths.iter().sum();
            let spacing = (count.saturating_sub(1) as i32) * (6.0 * scale).round() as i32;
            (sum_w + spacing + (20.0 * scale).round() as i32, (38.0 * scale).round() as i32)
        } else {
            let default_w = (160.0 * scale).round() as i32;
            let min_col_w = (180.0 * scale).round() as i32;
            let max_w = item_widths.iter().cloned().max().unwrap_or(default_w).max(min_col_w);
            let pad = (16.0 * scale).round() as i32;
            let row_h = (32.0 * scale).round() as i32;
            let margin = (12.0 * scale).round() as i32;
            (max_w + pad, (count as i32 * row_h) + margin)
        };


        unsafe {
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
                // Sanity check against foreground window boundaries
                let fg_hwnd = GetForegroundWindow();
                if fg_hwnd != 0 as _ {
                    let mut fg_rect: RECT = std::mem::zeroed();
                    if GetWindowRect(fg_hwnd, &mut fg_rect) != 0 {
                        let inside_fg = pt.x >= fg_rect.left - 100
                            && pt.x <= fg_rect.right + 100
                            && pt.y >= fg_rect.top - 100
                            && pt.y <= fg_rect.bottom + 100;
                        if !inside_fg {
                            // Mouse is on another monitor or far away; anchor near bottom-left of target window
                            pt.x = fg_rect.left + 30;
                            pt.y = fg_rect.bottom - height - 40;
                        } else {
                            pt.y += 24;
                        }
                    } else {
                        pt.y += 24;
                    }
                } else {
                    pt.y += 24;
                }
            }

            // Screen boundary clamping
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

            {
                let mut data = CANDIDATE_DATA.lock().unwrap();
                *data = Some(CandidateData {
                    candidates: display_cands,
                    item_widths,
                    selected_index,
                    horizontal,
                    total_width,
                    height,
                    screen_pos: (pt.x, pt.y),
                    scale,
                });
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

fn hit_test_data(data: &CandidateData, screen_x: i32, screen_y: i32) -> Option<usize> {
    let (win_x, win_y) = data.screen_pos;
    if screen_x < win_x || screen_x > win_x + data.total_width {
        return None;
    }
    if screen_y < win_y || screen_y > win_y + data.height {
        return None;
    }

    let rel_x = screen_x - win_x;
    let rel_y = screen_y - win_y;
    let scale = data.scale.max(1.0);
    let count = data.candidates.len().min(5);

    if data.horizontal {
        let mut cur_x = (8.0 * scale).round() as i32;
        let pad_x = (6.0 * scale).round() as i32;
        for (i, &w) in data.item_widths.iter().take(count).enumerate() {
            if rel_x >= cur_x && rel_x <= cur_x + w {
                return Some(i);
            }
            cur_x += w + pad_x;
        }
    } else {
        let item_h = (32.0 * scale).round() as i32;
        let pad_top = (6.0 * scale).round() as i32;
        for i in 0..count {
            let top = pad_top + (i as i32 * item_h);
            if rel_y >= top && rel_y <= top + item_h {
                return Some(i);
            }
        }
    }
    None
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
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            paint_candidates(hwnd, hdc);
            EndPaint(hwnd, &ps);
            0
        }
        WM_ERASEBKGND => 1,
        WM_LBUTTONUP => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            let mut pt = POINT { x, y };
            ClientToScreen(hwnd, &mut pt);

            let cand_idx = {
                let guard = CANDIDATE_DATA.lock().unwrap();
                guard.as_ref().and_then(|d| hit_test_data(d, pt.x, pt.y))
            };

            if let Some(idx) = cand_idx {
                crate::win_hook::commit_candidate_by_index(idx);
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn is_windows_dark_mode() -> bool {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyW, RegQueryValueExW, HKEY_CURRENT_USER, REG_DWORD,
    };
    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let val_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
    let mut hkey = std::ptr::null_mut();

    if RegOpenKeyW(HKEY_CURRENT_USER, subkey.as_ptr(), &mut hkey) == 0 {
        let mut data: u32 = 0;
        let mut size: u32 = std::mem::size_of::<u32>() as u32;
        let mut val_type: u32 = 0;
        let res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            &mut data as *mut u32 as *mut u8,
            &mut size,
        );
        RegCloseKey(hkey);
        if res == 0 && val_type == REG_DWORD {
            return data == 0; // 0 = dark mode, 1 = light mode
        }
    }
    true // default to dark mode
}

unsafe fn paint_candidates(hwnd: HWND, hdc: HDC) {
    let data = {
        let guard = CANDIDATE_DATA.lock().unwrap();
        guard.clone()
    };

    let Some(data) = data else {
        return;
    };

    let scale = data.scale.max(1.0);
    let mut client_rect: RECT = std::mem::zeroed();
    windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

    let dark = is_windows_dark_mode();

    // Theme colors (in 0x00BBGGRR format for GDI):
    // Dark: #0f1a15 (rich emerald dark slate), Light: #f8fafc (clean frosted white)
    let bg_color = if dark { 0x00151A0F } else { 0x00FCFAF8 };
    // Border: #2f443b (subtle emerald border) or #cbd5e1 (soft light slate)
    let border_color = if dark { 0x003B442F } else { 0x00E1D5CB };
    // Active pill: Emerald brand #10b981 (R=16, G=185, B=129 -> 0x0081B910)
    let active_pill_color = 0x0081B910;
    // Active pill text: White in both modes for optimal contrast
    let active_text_color = 0x00FFFFFF;
    // Inactive text: #f1f5f9 (light text on dark bg) or #0f172a (dark slate text on light bg)
    let inactive_text_color = if dark { 0x00F9F5F1 } else { 0x002A170F };

    let bg_brush = CreateSolidBrush(bg_color);
    FillRect(hdc, &client_rect, bg_brush);
    DeleteObject(bg_brush as _);

    let border_brush = CreateSolidBrush(border_color);
    windows_sys::Win32::Graphics::Gdi::FrameRect(hdc, &client_rect, border_brush);
    DeleteObject(border_brush as _);

    SetBkMode(hdc, TRANSPARENT as _);

    // Prefer Nirmala UI (standard Indic font on Windows 10/11) with fallback to Vrinda and Segoe UI
    let font_name: Vec<u16> = "Nirmala UI\0".encode_utf16().collect();
    let font_size = (-15.0 * scale).round() as i32;
    let font = CreateFontW(
        font_size,
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
    let mut cur_x = (8.0 * scale).round() as i32;
    let item_height = (28.0 * scale).round() as i32;
    let pad_y = (4.0 * scale).round() as i32;
    let pad_x = (6.0 * scale).round() as i32;
    let radius = (8.0 * scale).round() as i32;
    let row_h = (32.0 * scale).round() as i32;

    for i in 0..count {
        let item_w = data.item_widths.get(i).copied().unwrap_or((68.0 * scale).round() as i32);
        let item_rect = if data.horizontal {
            let r = RECT {
                left: cur_x,
                top: pad_y,
                right: cur_x + item_w,
                bottom: client_rect.bottom - pad_y,
            };
            cur_x += item_w + pad_x;
            r
        } else {
            RECT {
                left: pad_x,
                top: pad_x + (i as i32 * row_h),
                right: client_rect.right - pad_x,
                bottom: pad_x + (i as i32 * row_h) + item_height,
            }
        };

        if i == data.selected_index {
            let active_brush = CreateSolidBrush(active_pill_color);
            let old_brush = SelectObject(hdc, active_brush as _);
            RoundRect(
                hdc,
                item_rect.left,
                item_rect.top,
                item_rect.right,
                item_rect.bottom,
                radius,
                radius,
            );
            SelectObject(hdc, old_brush);
            DeleteObject(active_brush as _);

            SetTextColor(hdc, active_text_color);
        } else {
            SetTextColor(hdc, inactive_text_color);
        }

        let label = format!("{}. {}", i + 1, data.candidates[i]);
        let mut label_utf16: Vec<u16> = label.encode_utf16().collect();

        let mut text_rect = item_rect;
        let align_flags = if data.horizontal {
            DT_CENTER | DT_VCENTER | DT_SINGLELINE
        } else {
            text_rect.left += (8.0 * scale).round() as i32;
            DT_VCENTER | DT_SINGLELINE
        };
        DrawTextW(
            hdc,
            label_utf16.as_mut_ptr(),
            label_utf16.len() as i32,
            &mut text_rect,
            align_flags,
        );
    }

    SelectObject(hdc, old_font);
    DeleteObject(font as _);
}
