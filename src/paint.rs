use windows::Win32::Foundation::{HWND, RECT};

use crate::consts::{PANEL_CORNER, PANEL_MARGIN};
use crate::segments::{self, Palette};
use crate::sys::drawing::{self, PaintScope};
use crate::time::ClockTime;

pub fn paint(hwnd: HWND) {
    let scope = PaintScope::begin(hwnd);
    let hdc = scope.hdc();

    // GDI 对象在 PaintScope 之后声明，因此会先于 EndPaint 析构。
    let palette = Palette::new();

    let client = drawing::client_rect(hwnd);
    let panel = RECT {
        left: client.left + PANEL_MARGIN,
        top: client.top + PANEL_MARGIN,
        right: client.right - PANEL_MARGIN,
        bottom: client.bottom - PANEL_MARGIN,
    };

    {
        let _brush = palette.panel_brush.select(hdc);
        let _pen = palette.panel_pen.select(hdc);
        drawing::fill_rounded_rect(hdc, panel, PANEL_CORNER);
    }

    let time = ClockTime::now().format_hms();
    segments::draw_clock(hdc, &time, &palette);
}
