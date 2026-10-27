use windows::Win32::System::SystemInformation::GetLocalTime;

/// 本地时间，只保留时钟显示需要的字段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl ClockTime {
    pub fn now() -> Self {
        // SAFETY: GetLocalTime 无参数、无失败路径。
        let st = unsafe { GetLocalTime() };
        Self {
            hour: st.wHour as u8,
            minute: st.wMinute as u8,
            second: st.wSecond as u8,
        }
    }

    /// 格式化为 `HH:MM:SS` 的 8 个 ASCII 字节，避免每次重绘分配。
    pub fn format_hms(self) -> [u8; 8] {
        let mut buf = [b'0'; 8];
        buf[2] = b':';
        buf[5] = b':';

        buf[0] = b'0' + self.hour / 10;
        buf[1] = b'0' + self.hour % 10;
        buf[3] = b'0' + self.minute / 10;
        buf[4] = b'0' + self.minute % 10;
        buf[6] = b'0' + self.second / 10;
        buf[7] = b'0' + self.second % 10;

        buf
    }
}

/// 距最近整点的有符号秒数。
///
/// 正数表示距下一个整点，负数表示距上一个整点：
/// - `:00:00` → 0
/// - `:59:30` → +30
/// - `:00:30` → -30
/// - `:30:00` → -1800
pub fn seconds_from_nearest_hour(t: ClockTime) -> i32 {
    let secs_in_hour = t.minute as i32 * 60 + t.second as i32;
    if secs_in_hour <= 1800 {
        -secs_in_hour
    } else {
        3600 - secs_in_hour
    }
}

#[cfg(test)]
mod tests {
    use super::{ClockTime, seconds_from_nearest_hour};

    fn t(hour: u8, minute: u8, second: u8) -> ClockTime {
        ClockTime {
            hour,
            minute,
            second,
        }
    }

    #[test]
    fn formats_with_leading_zeroes() {
        assert_eq!(&t(0, 0, 0).format_hms(), b"00:00:00");
        assert_eq!(&t(9, 5, 3).format_hms(), b"09:05:03");
        assert_eq!(&t(23, 59, 59).format_hms(), b"23:59:59");
    }

    #[test]
    fn distance_to_nearest_hour() {
        assert_eq!(seconds_from_nearest_hour(t(11, 59, 29)), 31);
        assert_eq!(seconds_from_nearest_hour(t(11, 59, 30)), 30);
        assert_eq!(seconds_from_nearest_hour(t(12, 0, 0)), 0);
        assert_eq!(seconds_from_nearest_hour(t(12, 0, 30)), -30);
        assert_eq!(seconds_from_nearest_hour(t(12, 0, 31)), -31);
        assert_eq!(seconds_from_nearest_hour(t(11, 30, 0)), -1800);
        assert_eq!(seconds_from_nearest_hour(t(11, 30, 1)), 1799);
    }
}
