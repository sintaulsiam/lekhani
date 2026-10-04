//! Native Windows Floating Mode-Switch On-Screen Display (OSD / HUD Toast)
//! Displays a transient, modern HUD confirming language mode toggles (e.g. বাংলা / English).

#![cfg(windows)]

use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE,
    DT_VCENTER, HDC, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetSystemMetrics, KillTimer, RegisterClassW, SetTimer,
    SetWindowPos, ShowWindow, CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, WM_ERASEBKGND, WM_PAINT, WM_TIMER, WNDCLASSW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

const OSD_TIMER_ID: usize = 0x105D;
const OSD_DURATION_MS: u32 = 850;

#[derive(Clone, Default)]
struct OsdData {
    title: String,
    subtitle: String,
    is_bengali: bool,
}

static OSD_DATA: Mutex<Option<OsdData>> = Mutex::new(None);
static OSD_HWND: Mutex<Option<usize>> = Mutex::new(None);

pub struct OsdWindow {
    pub hwnd: HWND,
}

unsafe impl Send for OsdWindow {}
unsafe impl Sync for OsdWindow {}

unsafe fn get_osd_dpi(hwnd: HWND) -> u32 {
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

impl OsdWindow {
    pub fn new() -> Option<Self> {
        unsafe {
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            let hmod = GetModuleHandleW(std::ptr::null());

            let class_name: Vec<u16> = "LekhaniOsdWin\0".encode_utf16().collect();
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(osd_wnd_proc),
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
                240,
                54,
                0 as _,
                0 as _,
                hmod,
                std::ptr::null(),
            );

            if hwnd == 0 as _ {
                return None;
            }

            *OSD_HWND.lock().unwrap() = Some(hwnd as usize);
            Some(Self { hwnd })
        }
    }

    /// Show the mode-switch OSD HUD with the given layout title and language state
    pub fn show(&self, title: &str, subtitle: &str, is_bengali: bool) {
        {
            let mut data = OSD_DATA.lock().unwrap();
            *data = Some(OsdData {
                title: title.to_string(),
                subtitle: subtitle.to_string(),
                is_bengali,
            });
        }

        unsafe {
            let scale = (get_osd_dpi(self.hwnd) as f32 / 96.0).max(1.0);
            let width = (240.0 * scale).round() as i32;
            let height = (54.0 * scale).round() as i32;

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);

            // Centered horizontally, positioned ~15% from the bottom of the screen
            let pos_x = (screen_w - width) / 2;
            let pos_y = screen_h - (screen_h / 5) - height;

            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                pos_x,
                pos_y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            InvalidateRect(self.hwnd, std::ptr::null(), 1);

            // Reset auto-hide timer
            KillTimer(self.hwnd, OSD_TIMER_ID);
            SetTimer(self.hwnd, OSD_TIMER_ID, OSD_DURATION_MS, None);
        }
    }

    #[allow(dead_code)]
    pub fn hide(&self) {
        {
            let mut data = OSD_DATA.lock().unwrap();
            *data = None;
        }
        unsafe {
            KillTimer(self.hwnd, OSD_TIMER_ID);
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

#[allow(dead_code)]
pub fn show_global_osd(title: &str, subtitle: &str, is_bengali: bool) {
    if let Ok(guard) = OSD_HWND.lock() {
        if let Some(hwnd) = *guard {
            let osd = OsdWindow { hwnd: hwnd as HWND };
            osd.show(title, subtitle, is_bengali);
        }
    }
}

unsafe extern "system" fn osd_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            paint_osd(hwnd, hdc);

            EndPaint(hwnd, &ps);
            0
        }
        WM_TIMER => {
            if wparam == OSD_TIMER_ID {
                KillTimer(hwnd, OSD_TIMER_ID);
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }
        WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_osd(hwnd: HWND, hdc: HDC) {
    let data = {
        let guard = OSD_DATA.lock().unwrap();
        guard.clone()
    };

    let Some(data) = data else {
        return;
    };

    let scale = (get_osd_dpi(hwnd) as f32 / 96.0).max(1.0);
    let mut client_rect: RECT = std::mem::zeroed();
    windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

    // Background fill: #181825 (Deep dark glass)
    let bg_brush = CreateSolidBrush(0x00251818);
    FillRect(hdc, &client_rect, bg_brush);
    DeleteObject(bg_brush as _);

    // Accent border pill
    let border_color = if data.is_bengali {
        0x0081B910 // Emerald green (#10B981)
    } else {
        0x00A6ADC8 // Slate grey
    };

    let border_radius = (14.0 * scale).round() as i32;
    let inner_radius = (12.0 * scale).round() as i32;

    let border_brush = CreateSolidBrush(border_color);
    let old_brush = SelectObject(hdc, border_brush as _);
    RoundRect(
        hdc,
        client_rect.left,
        client_rect.top,
        client_rect.right,
        client_rect.bottom,
        border_radius,
        border_radius,
    );
    SelectObject(hdc, old_brush);
    DeleteObject(border_brush as _);

    // Inner background
    let inner_brush = CreateSolidBrush(0x00251818);
    let old_inner = SelectObject(hdc, inner_brush as _);
    RoundRect(
        hdc,
        client_rect.left + 2,
        client_rect.top + 2,
        client_rect.right - 2,
        client_rect.bottom - 2,
        inner_radius,
        inner_radius,
    );
    SelectObject(hdc, old_inner);
    DeleteObject(inner_brush as _);

    SetBkMode(hdc, TRANSPARENT as _);

    // Primary Title (e.g. "বাংলা" or "English")
    let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    let font_size_title = (-18.0 * scale).round() as i32;
    let title_font = CreateFontW(
        font_size_title,
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
    let old_font = SelectObject(hdc, title_font as _);

    if data.is_bengali {
        SetTextColor(hdc, 0x00A1E3A6); // Mint green
    } else {
        SetTextColor(hdc, 0x00F4D6CD); // Clean white/lavender
    }

    let mut title_utf16: Vec<u16> = data.title.encode_utf16().collect();
    let pad_x = (8.0 * scale).round() as i32;
    let title_h = (24.0 * scale).round() as i32;
    let pad_top = (4.0 * scale).round() as i32;
    let mut title_rect = RECT {
        left: client_rect.left + pad_x,
        top: client_rect.top + pad_top,
        right: client_rect.right - pad_x,
        bottom: client_rect.top + pad_top + title_h,
    };
    DrawTextW(
        hdc,
        title_utf16.as_mut_ptr(),
        title_utf16.len() as i32,
        &mut title_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    // Subtitle (e.g. "Avro Phonetic" or "Standard US")
    let font_size_sub = (-11.0 * scale).round() as i32;
    let sub_font = CreateFontW(
        font_size_sub,
        0,
        0,
        0,
        400,
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
    SelectObject(hdc, sub_font as _);
    SetTextColor(hdc, 0x00A6ADC8); // Muted subtext

    let mut sub_utf16: Vec<u16> = data.subtitle.encode_utf16().collect();
    let mut sub_rect = RECT {
        left: client_rect.left + pad_x,
        top: title_rect.bottom,
        right: client_rect.right - pad_x,
        bottom: client_rect.bottom - pad_top,
    };
    DrawTextW(
        hdc,
        sub_utf16.as_mut_ptr(),
        sub_utf16.len() as i32,
        &mut sub_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    SelectObject(hdc, old_font);
    DeleteObject(title_font as _);
    DeleteObject(sub_font as _);
}
