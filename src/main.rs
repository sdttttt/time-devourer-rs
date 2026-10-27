#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod consts;
mod fade;
mod gdi;
mod paint;
mod schedule;
mod segments;
mod time;
mod tray;
mod window;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MB_ICONERROR, MSG, MessageBoxW, TranslateMessage,
};
use windows::core::{PCWSTR, w};

fn main() {
    install_panic_hook();

    if let Err(error) = run() {
        show_error(&format!("启动失败：{error}"));
        std::process::exit(1);
    }
}

fn run() -> windows::core::Result<()> {
    let hinstance = window::instance()?;
    window::register_class(hinstance, Some(app::wndproc))?;
    let _hwnd = window::create_window(hinstance)?;
    message_loop()
}

fn message_loop() -> windows::core::Result<()> {
    let mut message = MSG::default();

    loop {
        // SAFETY: message 是有效可写指针。
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) };

        // GetMessageW 返回 -1 表示错误；它也是“真”，不能只看 as_bool。
        if result.0 == -1 {
            return Err(windows::core::Error::from_win32());
        }
        if result.0 == 0 {
            break;
        }

        // SAFETY: message 由 GetMessageW 填充。
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn show_error(text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: wide 在调用期间有效且以 NUL 结尾。
    unsafe {
        MessageBoxW(None, PCWSTR(wide.as_ptr()), w!("Error"), MB_ICONERROR);
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        show_error(&format!("Panic: {info}"));
    }));
}
