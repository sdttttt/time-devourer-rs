/// 每帧 alpha 的步进，和原 C++ 实现一致。
pub const FADE_STEP: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeTick {
    pub alpha: u8,
    pub animating: bool,
}

/// 纯 alpha 动画状态机：0 ↔ 255，每帧 ±5。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fade {
    alpha: u8,
    target_visible: bool,
    animating: bool,
}

impl Fade {
    pub const fn new() -> Self {
        Self {
            alpha: 0,
            target_visible: false,
            animating: false,
        }
    }

    #[cfg(test)]
    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    /// 请求淡入/淡出。
    ///
    /// 返回 true 表示调用方需要启动动画定时器；目标未变时返回 false，
    /// 避免每秒重复 SetTimer 重置动画节奏。
    pub fn request(&mut self, visible: bool) -> bool {
        if self.target_visible == visible {
            return false;
        }

        self.target_visible = visible;

        let at_target = if visible {
            self.alpha == 255
        } else {
            self.alpha == 0
        };
        if at_target {
            self.animating = false;
            return false;
        }

        let need_timer = !self.animating;
        self.animating = true;
        need_timer
    }

    /// 托盘左键：立即显示到全不透明，然后淡出。
    pub fn flash_from_full(&mut self) {
        self.alpha = 255;
        self.target_visible = false;
        self.animating = true;
    }

    /// 前进一帧。
    pub fn tick(&mut self) -> FadeTick {
        if self.animating {
            if self.target_visible {
                self.alpha = self.alpha.saturating_add(FADE_STEP);
                if self.alpha == 255 {
                    self.animating = false;
                }
            } else {
                self.alpha = self.alpha.saturating_sub(FADE_STEP);
                if self.alpha == 0 {
                    self.animating = false;
                }
            }
        }

        FadeTick {
            alpha: self.alpha,
            animating: self.animating,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Fade;

    fn tick_n(fade: &mut Fade, n: usize) -> super::FadeTick {
        let mut last = fade.tick();
        for _ in 1..n {
            last = fade.tick();
        }
        last
    }

    #[test]
    fn starts_hidden_and_clear() {
        let fade = Fade::new();
        assert_eq!(fade.alpha(), 0);
        assert!(!fade.animating);
    }

    #[test]
    fn fades_in_over_51_ticks() {
        let mut fade = Fade::new();
        assert!(fade.request(true));
        let at_50 = tick_n(&mut fade, 50);
        assert_eq!(at_50.alpha, 250);
        assert!(at_50.animating);

        let at_51 = fade.tick();
        assert_eq!(at_51.alpha, 255);
        assert!(!at_51.animating);
    }

    #[test]
    fn fades_out_over_51_ticks() {
        let mut fade = Fade::new();
        fade.request(true);
        tick_n(&mut fade, 51);
        assert_eq!(fade.alpha(), 255);

        assert!(fade.request(false));
        let at_51 = tick_n(&mut fade, 51);
        assert_eq!(at_51.alpha, 0);
        assert!(!at_51.animating);
    }

    #[test]
    fn same_target_does_not_restart() {
        let mut fade = Fade::new();
        assert!(fade.request(true));
        assert!(!fade.request(true));

        tick_n(&mut fade, 51);
        assert!(!fade.request(true));
    }

    #[test]
    fn reverses_mid_animation_without_new_timer() {
        let mut fade = Fade::new();
        assert!(fade.request(true));
        tick_n(&mut fade, 10);
        assert_eq!(fade.alpha(), 50);

        assert!(!fade.request(false));
        let after = tick_n(&mut fade, 10);
        assert_eq!(after.alpha, 0);
        assert!(!after.animating);
    }

    #[test]
    fn flash_starts_from_full_and_fades_out() {
        let mut fade = Fade::new();
        fade.flash_from_full();
        assert_eq!(fade.alpha(), 255);
        assert!(fade.animating);

        let after = tick_n(&mut fade, 51);
        assert_eq!(after.alpha, 0);
        assert!(!after.animating);
    }

    #[test]
    fn alpha_never_overflows() {
        let mut fade = Fade::new();
        fade.request(true);
        tick_n(&mut fade, 100);
        assert_eq!(fade.alpha(), 255);

        fade.request(false);
        tick_n(&mut fade, 100);
        assert_eq!(fade.alpha(), 0);
    }
}
