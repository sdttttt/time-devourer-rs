use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, HMENU, LoadIconW, MF_STRING,
    PostMessageW, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_NULL,
};
use windows::core::PCWSTR;

use crate::consts::{IDI_ICON, MENU_EXIT_ID, MENU_EXIT_TEXT, TRAY_ICON_ID, TRAY_TIP, WM_TRAY_ICON};

pub struct TrayIcon {
    data: NOTIFYICONDATAW,
    added: bool,
}

impl TrayIcon {
    pub fn add(hwnd: HWND, hinstance: HINSTANCE) -> Option<Self> {
        // SAFETY: hinstance 有效；105 是图标资源 ID。
        let icon = unsafe {
            LoadIconW(
                Some(hinstance),
                PCWSTR::from_raw(IDI_ICON as usize as *const u16),
            )
        }
        .ok()?;

        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ICON_ID,
            uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE,
            uCallbackMessage: WM_TRAY_ICON,
            hIcon: icon,
            ..Default::default()
        };
        write_tip(&mut data.szTip, TRAY_TIP);

        // SAFETY: data 已按 NOTIFYICONDATAW 规则初始化。
        let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &data) };
        ok.as_bool().then_some(Self { data, added: true })
    }

    pub fn remove(&mut self) {
        if self.added {
            // SAFETY: data 是先前 NIM_ADD 成功的同一份数据。
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &self.data);
            }
            self.added = false;
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove();
    }
}

fn write_tip(buffer: &mut [u16; 128], text: &str) {
    let mut index = 0;
    for unit in text.encode_utf16() {
        if index + 1 >= buffer.len() {
            break;
        }
        buffer[index] = unit;
        index += 1;
    }
    buffer[index] = 0;
}

struct Menu(HMENU);

impl Drop for Menu {
    fn drop(&mut self) {
        // SAFETY: 菜单在本次调用中创建，尚未销毁。
        unsafe {
            let _ = DestroyMenu(self.0);
        }
    }
}

/// 显示托盘右键菜单，返回选中的命令 ID（0 表示未选择）。
///
/// 使用 `TPM_RETURNCMD`：菜单不派发 `WM_COMMAND`，避免在模态循环中
/// 重入 wndproc 导致 `AppState` 别名/悬空。
pub fn show_menu(hwnd: HWND) -> usize {
    // SAFETY: 无参数失败路径由 Result 处理。
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return 0;
    };
    let menu = Menu(menu);

    unsafe {
        let _ = AppendMenuW(menu.0, MF_STRING, MENU_EXIT_ID, MENU_EXIT_TEXT);

        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);

        let command = TrackPopupMenu(
            menu.0,
            TPM_RETURNCMD | TPM_BOTTOMALIGN | TPM_LEFTALIGN | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            Some(0),
            hwnd,
            None,
        );

        // MSDN 建议：菜单关闭后向窗口投递 WM_NULL，确保菜单正确消失。
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));

        command.0 as usize
    }
}
