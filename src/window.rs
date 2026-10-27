use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND};
use windows::Win32::Graphics::Gdi::{
    COLOR_WINDOW, CreateRoundRectRgn, DeleteObject, HBRUSH, SetWindowRgn, UpdateWindow,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, IDC_ARROW, LWA_ALPHA, LoadCursorW,
    LoadIconW, RegisterClassExW, SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    SetLayeredWindowAttributes, SetTimer, SetWindowPos, ShowWindow, WNDCLASSEXW, WNDPROC,
    WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
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
        hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut core::ffi::c_void),
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
