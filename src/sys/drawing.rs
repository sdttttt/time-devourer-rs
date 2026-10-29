//! 绘制原语与 BeginPaint/EndPaint 配对。

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HDC, PAINTSTRUCT, Rectangle, RoundRect};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

/// BeginPaint/EndPaint 的 RAII 配对。
pub struct PaintScope {
    hwnd: HWND,
    ps: PAINTSTRUCT,
    hdc: HDC,
}

impl PaintScope {
    pub fn begin(hwnd: HWND) -> Self {
        let mut ps = PAINTSTRUCT::default();
        // SAFETY: hwnd 有效；ps 在 EndPaint 前保持有效。
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        Self { hwnd, ps, hdc }
    }

    pub fn hdc(&self) -> HDC {
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

pub fn client_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    // SAFETY: rect 是有效的可写指针。
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
    }
    rect
}

pub fn fill_rect(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) {
    // SAFETY: hdc 由 PaintScope 提供；越界部分由 GDI 裁剪。
    unsafe {
        let _ = Rectangle(hdc, left, top, right, bottom);
    }
}

pub fn fill_rounded_rect(hdc: HDC, rect: RECT, corner: i32) {
    // SAFETY: hdc 由 PaintScope 提供。
    unsafe {
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            corner,
            corner,
        );
    }
}
