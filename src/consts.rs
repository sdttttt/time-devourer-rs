use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::WindowsAndMessaging::WM_USER;
use windows::core::{PCWSTR, w};

/// COLORREF 的字节序是 0x00BBGGRR。
pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

// 窗口类与标题
pub const CLASS_NAME: PCWSTR = w!("TimeWindowClass");
pub const WINDOW_TITLE: PCWSTR = w!("Timer");

// 托盘
pub const TRAY_TIP: &str = "Time Devourer";
pub const TRAY_ICON_ID: u32 = 2;
pub const WM_TRAY_ICON: u32 = WM_USER + 1;
pub const MENU_EXIT_ID: usize = 1001;
pub const MENU_EXIT_TEXT: PCWSTR = w!("退出");

// 图标资源 ID，和 resource.h 中的 IDI_ICON 保持一致
pub const IDI_ICON: u16 = 105;

// 窗口尺寸与位置
pub const WINDOW_WIDTH: i32 = 320;
pub const WINDOW_HEIGHT: i32 = 100;
pub const WINDOW_X: i32 = 10;
pub const WINDOW_Y: i32 = 10;

// 窗口圆角与外框区域
pub const WINDOW_CORNER: i32 = 10;
// 显示面板与窗口边缘的间距
pub const PANEL_MARGIN: i32 = 10;
// 显示面板的圆角
pub const PANEL_CORNER: i32 = 10;

// 颜色
pub const PANEL_COLOR: COLORREF = rgb(220, 220, 220);
pub const SEG_ON_COLOR: COLORREF = rgb(0, 0, 0);
pub const SEG_OFF_COLOR: COLORREF = rgb(200, 200, 200);

// 定时器
pub const DATE_TIMER_ID: usize = 1;
pub const ANIMATION_TIMER_ID: usize = 2;
pub const ANIMATION_INTERVAL_MS: u32 = 10;

// 整点前后显示提醒的半径：秒
pub const SHOW_RADIUS_SEC: i32 = 30;

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
