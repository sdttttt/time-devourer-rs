#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 除 sys 与 app 中窗口状态的三处契约外，其它模块不允许出现 unsafe。
#![deny(unsafe_code)]

// wndproc 必须是 unsafe extern "system"，AppState 指针的解引用与回收也在此。
#[allow(unsafe_code)]
mod app;
mod consts;
mod fade;
mod paint;
mod schedule;
mod segments;
#[allow(unsafe_code)]
mod sys;
mod time;

use windows::core::Result;

fn main() {
    install_panic_hook();

    if let Err(error) = run() {
        sys::message::show_error(&format!("启动失败：{error}"));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let hinstance = sys::window::instance()?;
    sys::window::register_class(hinstance, Some(app::wndproc))?;
    let _hwnd = sys::window::create_window(hinstance)?;
    sys::message::run_message_loop()
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        sys::message::show_error(&format!("Panic: {info}"));
    }));
}
