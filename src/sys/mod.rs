//! Win32 FFI 的唯一边界，按功能拆分子模块。
//!
//! 全部 `unsafe` 调用集中在本模块树内（另见 `app` 中窗口状态的三处安全契约），
//! 对外只暴露安全函数与 RAII 类型。`main.rs` 中的 `#![deny(unsafe_code)]`
//! 保证新的 `unsafe` 不会出现在其它模块。
//!
//! 约定：安全前提能用类型表达就做成安全函数；表达不了的一律保持 `unsafe fn`
//! 并写进文档，不要用安全签名掩盖不变量。

pub mod drawing;
pub mod gdi;
pub mod message;
pub mod time;
pub mod tray;
pub mod window;
