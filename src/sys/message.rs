//! 消息循环与错误提示。

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MB_ICONERROR, MSG, MessageBoxW, TranslateMessage,
};
use windows::core::{Error, PCWSTR, w};

pub fn run_message_loop() -> windows::core::Result<()> {
    let mut message = MSG::default();

    loop {
        // SAFETY: message 是有效可写指针。
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) };

        // GetMessageW 返回 -1 表示错误；它也是“真”，不能只看 as_bool。
        if result.0 == -1 {
            return Err(Error::from_win32());
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

pub fn show_error(text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: wide 在调用期间有效且以 NUL 结尾。
    unsafe {
        MessageBoxW(None, PCWSTR(wide.as_ptr()), w!("Error"), MB_ICONERROR);
    }
}
