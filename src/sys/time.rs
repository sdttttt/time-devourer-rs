//! 本地时间。

use windows::Win32::System::SystemInformation::GetLocalTime;

/// 本地时间的时、分、秒。
pub fn local_hms() -> (u8, u8, u8) {
    // SAFETY: GetLocalTime 无参数、无失败路径，直接返回系统分解好的本地时间。
    let st = unsafe { GetLocalTime() };
    (st.wHour as u8, st.wMinute as u8, st.wSecond as u8)
}
