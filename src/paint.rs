use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HDC, PAINTSTRUCT, RoundRect};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

use crate::consts::{PANEL_CORNER, PANEL_MARGIN};
use crate::segments::{self, Palette};
use crate::time::ClockTime;

/// BeginPaint/EndPaint 的 RAII 配对。
struct PaintScope {
    hwnd: HWND,
    ps: PAINTSTRUCT,
    hdc: HDC,
}

impl PaintScope {
    fn begin(hwnd: HWND) -> Self {
        let mut ps = PAINTSTRUCT::default();
        // SAFETY: hwnd 有效；ps 在 EndPaint 前保持有效。
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        Self { hwnd, ps, hdc }
    }

    fn hdc(&self) -> HDC {
        self.hdc
    }
}

impl Drop for PaintScope {
    fn drop(&mut self) {
        // SAFETY: hdc 由 BeginPaint 获得，ps 未被移动。
        unsafe {
            let _ = EndPaint(self.hwnd, &self.ps);
        }
    }
}

pub fn paint(hwnd: HWND) {
    let scope = PaintScope::begin(hwnd);
    let hdc = scope.hdc();

    // GDI 对象在 PaintScope 之后声明，因此会先于 EndPaint 析构。
    let palette = Palette::new();

    let mut client = RECT::default();
    // SAFETY: client 是有效的可写指针。
    unsafe {
        let _ = GetClientRect(hwnd, &mut client);
    }

    let panel = RECT {
        left: client.left + PANEL_MARGIN,
        top: client.top + PANEL_MARGIN,
        right: client.right - PANEL_MARGIN,
        bottom: client.bottom - PANEL_MARGIN,
    };

    {
        let _brush = palette.panel_brush.select(hdc);
        let _pen = palette.panel_pen.select(hdc);
        // SAFETY: hdc 由 BeginPaint 提供。
        unsafe {
            let _ = RoundRect(
                hdc,
                panel.left,
                panel.top,
                panel.right,
                panel.bottom,
                PANEL_CORNER,
                PANEL_CORNER,
            );
        }
    }

    let time = ClockTime::now().format_hms();
    segments::draw_clock(hdc, &time, &palette);
}
