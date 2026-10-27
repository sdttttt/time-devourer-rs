use std::marker::PhantomData;

use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DeleteObject, HDC, HGDIOBJ, PEN_STYLE, SelectObject,
};

/// COLORREF 的字节序是 0x00BBGGRR。
pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

/// 独占一个 GDI 对象，析构时 DeleteObject。
pub struct OwnedGdi(HGDIOBJ);

impl OwnedGdi {
    pub fn brush(color: COLORREF) -> Self {
        // SAFETY: CreateSolidBrush 不涉及借用；失败时返回无效句柄，Drop 会跳过。
        Self(unsafe { CreateSolidBrush(color).into() })
    }

    pub fn pen(style: PEN_STYLE, width: i32, color: COLORREF) -> Self {
        // SAFETY: 同上。
        Self(unsafe { CreatePen(style, width, color).into() })
    }

    /// 把对象选入 DC，返回的守卫析构时会恢复之前的对象。
    ///
    /// 守卫借用 `self`，因此编译器保证 GDI 对象比守卫活得久，
    /// 不会出现“删除仍被选中的对象”。
    pub fn select<'a>(&'a self, hdc: HDC) -> Selected<'a> {
        // SAFETY: hdc 由 BeginPaint 提供，self 在守卫生命周期内有效。
        let prev = unsafe { SelectObject(hdc, self.0) };
        Selected {
            hdc,
            prev,
            _owner: PhantomData,
        }
    }
}

impl Drop for OwnedGdi {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: 对象未被选入任何 DC（由 Selected 的借用约束保证）。
            unsafe {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

pub struct Selected<'a> {
    hdc: HDC,
    prev: HGDIOBJ,
    _owner: PhantomData<&'a OwnedGdi>,
}

impl Drop for Selected<'_> {
    fn drop(&mut self) {
        if !self.prev.is_invalid() {
            // SAFETY: 恢复同一个 DC 上的旧对象。
            unsafe {
                SelectObject(self.hdc, self.prev);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rgb;

    #[test]
    fn rgb_uses_bbggrr_layout() {
        assert_eq!(rgb(0x12, 0x34, 0x56).0, 0x0056_3412);
        assert_eq!(rgb(0, 0, 0).0, 0);
        assert_eq!(rgb(255, 255, 255).0, 0x00FF_FFFF);
    }
}
