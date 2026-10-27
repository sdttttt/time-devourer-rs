use crate::consts::SHOW_RADIUS_SEC;
use crate::fade::{Fade, FadeTick};
use crate::time::{ClockTime, seconds_from_nearest_hour};

/// 可见性策略 + 淡入淡出状态机。
pub struct Schedule {
    fade: Fade,
    /// 启动时如果已经处于“整点后 30 秒”内，则保持隐藏，直到该窗口过去。
    ///
    /// 原 C++ 的 StepCountDown 以“距下一整点”为终点，启动时不会补显刚过去的整点；
    /// 这里保留同样的启动行为。
    suppress_current_window: bool,
}

impl Schedule {
    pub fn new(now: ClockTime) -> Self {
        let delta = seconds_from_nearest_hour(now);
        Self {
            fade: Fade::new(),
            suppress_current_window: delta < 0 && delta.abs() <= SHOW_RADIUS_SEC,
        }
    }

    /// 每个 1 秒定时器调用一次。
    ///
    /// 返回 true 表示调用方需要启动（或重新启动）10ms 动画定时器。
    pub fn update(&mut self, now: ClockTime, force_visible: bool) -> bool {
        let delta = seconds_from_nearest_hour(now);

        // 一旦离开“整点后 30 秒”的区间，就不再抑制。
        if delta < -SHOW_RADIUS_SEC {
            self.suppress_current_window = false;
        }

        let desired =
            force_visible || (!self.suppress_current_window && delta.abs() <= SHOW_RADIUS_SEC);

        self.fade.request(desired)
    }

    pub fn animation_tick(&mut self) -> FadeTick {
        self.fade.tick()
    }

    /// 托盘左键：闪现当前时间后淡出。
    pub fn flash(&mut self) {
        self.fade.flash_from_full();
    }

    #[cfg(test)]
    pub fn alpha(&self) -> u8 {
        self.fade.alpha()
    }
}

#[cfg(test)]
mod tests {
    use super::Schedule;
    use crate::time::ClockTime;

    fn t(hour: u8, minute: u8, second: u8) -> ClockTime {
        ClockTime {
            hour,
            minute,
            second,
        }
    }

    #[test]
    fn window_is_visible_around_the_hour() {
        let mut schedule = Schedule::new(t(11, 59, 29));
        assert!(!schedule.update(t(11, 59, 29), false));
        assert_eq!(schedule.alpha(), 0);

        assert!(schedule.update(t(11, 59, 30), false));
        for _ in 0..51 {
            schedule.animation_tick();
        }
        assert_eq!(schedule.alpha(), 255);

        assert!(!schedule.update(t(12, 0, 30), false));
        assert_eq!(schedule.alpha(), 255);

        assert!(schedule.update(t(12, 0, 31), false));
        for _ in 0..51 {
            schedule.animation_tick();
        }
        assert_eq!(schedule.alpha(), 0);
    }

    #[test]
    fn startup_after_the_hour_does_not_replay_the_window() {
        let mut schedule = Schedule::new(t(12, 0, 20));

        assert!(!schedule.update(t(12, 0, 25), false));
        assert!(!schedule.update(t(12, 0, 30), false));
        assert!(!schedule.update(t(12, 0, 31), false));
        assert_eq!(schedule.alpha(), 0);

        assert!(!schedule.update(t(12, 59, 29), false));
        assert!(schedule.update(t(12, 59, 30), false));
    }

    #[test]
    fn startup_before_the_hour_shows_immediately_in_window() {
        let mut schedule = Schedule::new(t(11, 59, 40));
        assert!(schedule.update(t(11, 59, 41), false));
    }

    #[test]
    fn force_visible_overrides_policy() {
        let mut schedule = Schedule::new(t(12, 15, 0));
        assert!(schedule.update(t(12, 15, 1), true));
    }

    #[test]
    fn repeated_updates_do_not_restart_animation() {
        let mut schedule = Schedule::new(t(11, 59, 29));
        assert!(schedule.update(t(11, 59, 30), false));
        assert!(!schedule.update(t(11, 59, 31), false));
    }
}
