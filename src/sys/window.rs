//! 窗口类注册、窗口创建与窗口操作。

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    COLOR_WINDOW, CreateRoundRectRgn, DeleteObject, HBRUSH, InvalidateRect, SetWindowRgn,
    UpdateWindow,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    GWLP_USERDATA, GetCursorPos, GetWindowLongPtrW, GetWindowRect, IDC_ARROW, IDC_SIZEALL,
    KillTimer, LWA_ALPHA, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassExW, SW_SHOWNORMAL,
    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetCursor, SetLayeredWindowAttributes, SetTimer,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, WNDCLASSEXW, WNDPROC, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{Error, PCWSTR};

use crate::consts::{
    CLASS_NAME, DATE_TIMER_ID, IDI_ICON, WINDOW_CORNER, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
    WINDOW_X, WINDOW_Y,
};

pub fn instance() -> windows::core::Result<HINSTANCE> {
    // SAFETY: 传入 None 表示获取当前模块句柄。
    let module = unsafe { GetModuleHandleW(None)? };
    Ok(HINSTANCE(module.0))
}

pub fn register_class(hinstance: HINSTANCE, wndproc: WNDPROC) -> windows::core::Result<()> {
    // SAFETY: 资源 ID 为整数 MAKEINTRESOURCE，LoadIconW 返回共享图标。
    let icon = unsafe {
        LoadIconW(
            Some(hinstance),
            PCWSTR::from_raw(IDI_ICON as usize as *const u16),
        )?
    };
    // SAFETY: IDC_ARROW 是系统光标常量。
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };

    let window_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: wndproc,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: icon,
        hCursor: cursor,
        // COLOR_WINDOW + 1 是系统背景色的约定写法。
        hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut std::ffi::c_void),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CLASS_NAME,
        hIconSm: icon,
    };

    // SAFETY: window_class 在调用期间有效。
    let atom = unsafe { RegisterClassExW(&window_class) };
    if atom == 0 {
        return Err(Error::from_win32());
    }

    Ok(())
}

pub fn create_window(hinstance: HINSTANCE) -> windows::core::Result<HWND> {
    // SAFETY: 所有句柄/字符串参数均有效；窗口过程由已注册的类提供。
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            CLASS_NAME,
            WINDOW_TITLE,
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            Some(hinstance),
            None,
        )?
    };

    // SAFETY: hwnd 有效。
    unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA)?;

        let region = CreateRoundRectRgn(
            0,
            0,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WINDOW_CORNER,
            WINDOW_CORNER,
        );
        if region.is_invalid() {
            return Err(Error::from_win32());
        }

        // SetWindowRgn 成功后区域所有权归系统；失败时自行释放。
        if SetWindowRgn(hwnd, Some(region), true) == 0 {
            let _ = DeleteObject(region.into());
            return Err(Error::from_win32());
        }

        SetWindowPos(
            hwnd,
            None,
            WINDOW_X,
            WINDOW_Y,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )?;

        if SetTimer(Some(hwnd), DATE_TIMER_ID, 1000, None) == 0 {
            return Err(Error::from_win32());
        }

        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        let _ = UpdateWindow(hwnd);
    }

    Ok(hwnd)
}

pub fn default_window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: 未处理消息交还系统默认过程。
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

pub fn invalidate(hwnd: HWND) {
    // SAFETY: hwnd 有效。
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, true);
    }
}

pub fn start_timer(hwnd: HWND, id: usize, interval_ms: u32) {
    // SAFETY: hwnd 有效；同一 ID 的定时器会被更新而不是重复创建。
    unsafe {
        SetTimer(Some(hwnd), id, interval_ms, None);
    }
}

pub fn stop_timer(hwnd: HWND, id: usize) {
    // SAFETY: hwnd 有效。
    unsafe {
        let _ = KillTimer(Some(hwnd), id);
    }
}

pub fn set_alpha(hwnd: HWND, alpha: u8) {
    // SAFETY: hwnd 有效；分层窗口不使用色键，传 0 即可。
    unsafe {
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

pub fn destroy_window(hwnd: HWND) {
    // SAFETY: hwnd 有效。
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

pub fn post_quit() {
    // SAFETY: 只向当前线程的消息队列投递 WM_QUIT，不涉及指针。
    unsafe {
        PostQuitMessage(0);
    }
}

pub fn cursor_pos() -> Option<POINT> {
    let mut cursor = POINT::default();
    // SAFETY: cursor 是有效可写指针。
    unsafe { GetCursorPos(&mut cursor) }
        .is_ok()
        .then_some(cursor)
}

pub fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    // SAFETY: rect 是有效可写指针。
    unsafe { GetWindowRect(hwnd, &mut rect) }
        .is_ok()
        .then_some(rect)
}

pub fn move_window(hwnd: HWND, x: i32, y: i32) {
    // SAFETY: hwnd 有效。
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// 捕获鼠标并切成拖动光标，与 [`release_mouse`] 配对。
pub fn capture_mouse(hwnd: HWND) {
    // SAFETY: hwnd 有效；捕获在 release_mouse 或窗口销毁时解除。
    unsafe {
        SetCapture(hwnd);
        if let Ok(cursor) = LoadCursorW(None, IDC_SIZEALL) {
            SetCursor(Some(cursor));
        }
    }
}

pub fn release_mouse() {
    // SAFETY: 与 capture_mouse 配对。
    unsafe {
        let _ = ReleaseCapture();
    }
}

/// 读取窗口的用户数据槽。返回的裸指针需要调用方自行保证有效性。
pub fn user_data(hwnd: HWND) -> *mut std::ffi::c_void {
    // SAFETY: 只读写槽位本身，不触碰指针指向的内容。
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut std::ffi::c_void }
}

/// 写入窗口的用户数据槽；指针的所有权语义由调用方维护。
pub fn set_user_data(hwnd: HWND, pointer: *mut std::ffi::c_void) {
    // SAFETY: 同 user_data。
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, pointer as isize);
    }
}
