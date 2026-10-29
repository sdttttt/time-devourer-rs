use windows::Win32::Graphics::Gdi::{HDC, PS_SOLID};

use crate::consts::{PANEL_COLOR, SEG_OFF_COLOR, SEG_ON_COLOR};
use crate::sys::drawing::fill_rect;
use crate::sys::gdi::OwnedGdi;

/// 数码管段的宽度。
pub const SEG_WIDTH: i32 = 4;
/// 数码管段的长度。
pub const SEG_LENGTH: i32 = 25;
/// 数字之间的间距。
pub const DIGIT_SPACING: i32 = 10;
/// 每个数字占用的横向步进。
pub const DIGIT_STEP: i32 = SEG_LENGTH + 2 * SEG_WIDTH + DIGIT_SPACING; // 43

/// 绘制的起始坐标。
pub const PAINT_START_X: i32 = 20;
pub const PAINT_START_Y: i32 = 20;

/// 冒号方块大小与偏移。
pub const COLON_SIZE: i32 = 5;
pub const COLON_OFFSET_Y: i32 = 8;
pub const COLON_STEP: i32 = COLON_SIZE + DIGIT_SPACING; // 15

// 七段位掩码，位序与 SEGMENT_RECTS 一致。
pub const TOP: u8 = 1 << 0;
pub const LEFT_UP: u8 = 1 << 1;
pub const RIGHT_UP: u8 = 1 << 2;
pub const MIDDLE: u8 = 1 << 3;
pub const LEFT_DOWN: u8 = 1 << 4;
pub const RIGHT_DOWN: u8 = 1 << 5;
pub const BOTTOM: u8 = 1 << 6;

pub const DIGIT_MASKS: [u8; 10] = [
    TOP | LEFT_UP | RIGHT_UP | LEFT_DOWN | RIGHT_DOWN | BOTTOM, // 0: 0x77
    RIGHT_UP | RIGHT_DOWN,                                      // 1: 0x24
    TOP | RIGHT_UP | MIDDLE | LEFT_DOWN | BOTTOM,               // 2: 0x5D
    TOP | RIGHT_UP | MIDDLE | RIGHT_DOWN | BOTTOM,              // 3: 0x6D
    LEFT_UP | RIGHT_UP | MIDDLE | RIGHT_DOWN,                   // 4: 0x2E
    TOP | LEFT_UP | MIDDLE | RIGHT_DOWN | BOTTOM,               // 5: 0x6B
    TOP | LEFT_UP | MIDDLE | LEFT_DOWN | RIGHT_DOWN | BOTTOM,   // 6: 0x7B
    TOP | RIGHT_UP | RIGHT_DOWN,                                // 7: 0x25
    TOP | LEFT_UP | RIGHT_UP | MIDDLE | LEFT_DOWN | RIGHT_DOWN | BOTTOM, // 8: 0x7F
    TOP | LEFT_UP | RIGHT_UP | MIDDLE | RIGHT_DOWN | BOTTOM,    // 9: 0x6F
];

/// 七段相对数字左上角的矩形：(x, y, w, h)，索引即位序。
const SEGMENT_RECTS: [(i32, i32, i32, i32); 7] = [
    (SEG_WIDTH, 0, SEG_LENGTH, SEG_WIDTH), // TOP
    (0, SEG_WIDTH, SEG_WIDTH, SEG_LENGTH), // LEFT_UP
    (SEG_WIDTH + SEG_LENGTH, SEG_WIDTH, SEG_WIDTH, SEG_LENGTH), // RIGHT_UP
    (SEG_WIDTH, SEG_LENGTH + SEG_WIDTH, SEG_LENGTH, SEG_WIDTH), // MIDDLE
    (0, SEG_LENGTH + 2 * SEG_WIDTH, SEG_WIDTH, SEG_LENGTH), // LEFT_DOWN
    (
        SEG_WIDTH + SEG_LENGTH,
        SEG_LENGTH + 2 * SEG_WIDTH,
        SEG_WIDTH,
        SEG_LENGTH,
    ), // RIGHT_DOWN
    (
        SEG_WIDTH,
        2 * (SEG_LENGTH + SEG_WIDTH),
        SEG_LENGTH,
        SEG_WIDTH,
    ), // BOTTOM
];

/// 一次绘制共用的 GDI 对象，避免每个段都创建/删除 brush/pen。
pub struct Palette {
    pub panel_brush: OwnedGdi,
    pub panel_pen: OwnedGdi,
    on_brush: OwnedGdi,
    on_pen: OwnedGdi,
    off_brush: OwnedGdi,
    off_pen: OwnedGdi,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            panel_brush: OwnedGdi::brush(PANEL_COLOR),
            panel_pen: OwnedGdi::pen(PS_SOLID, 0, PANEL_COLOR),
            on_brush: OwnedGdi::brush(SEG_ON_COLOR),
            on_pen: OwnedGdi::pen(PS_SOLID, 0, SEG_ON_COLOR),
            off_brush: OwnedGdi::brush(SEG_OFF_COLOR),
            off_pen: OwnedGdi::pen(PS_SOLID, 0, SEG_OFF_COLOR),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

pub fn is_lit(digit: u8, bit: u8) -> bool {
    DIGIT_MASKS[digit as usize] & bit != 0
}

/// 绘制 `HH:MM:SS`。
pub fn draw_clock(hdc: HDC, time: &[u8; 8], palette: &Palette) {
    let mut x = PAINT_START_X;
    let y = PAINT_START_Y;

    for &byte in time {
        match byte {
            b'0'..=b'9' => {
                draw_digit(hdc, x, y, byte - b'0', palette);
                x += DIGIT_STEP;
            }
            b':' => {
                draw_colon(hdc, x, y, palette);
                x += COLON_STEP;
            }
            _ => {}
        }
    }
}

fn draw_digit(hdc: HDC, x: i32, y: i32, digit: u8, palette: &Palette) {
    // 先画全部 7 段（未点亮灰），再画点亮段（黑），与原 DrawBkDigit 完全一致。
    {
        let _brush = palette.off_brush.select(hdc);
        let _pen = palette.off_pen.select(hdc);
        for &(dx, dy, w, h) in SEGMENT_RECTS.iter() {
            fill_rect(hdc, x + dx, y + dy, x + dx + w, y + dy + h);
        }
    }

    {
        let _brush = palette.on_brush.select(hdc);
        let _pen = palette.on_pen.select(hdc);
        for (bit, &(dx, dy, w, h)) in SEGMENT_RECTS.iter().enumerate() {
            if is_lit(digit, 1 << bit) {
                fill_rect(hdc, x + dx, y + dy, x + dx + w, y + dy + h);
            }
        }
    }
}

fn draw_colon(hdc: HDC, x: i32, y: i32, palette: &Palette) {
    let colon_y = y + (SEG_LENGTH * 2 + SEG_WIDTH * 3) / 2;

    let _brush = palette.on_brush.select(hdc);
    let _pen = palette.on_pen.select(hdc);

    fill_rect(
        hdc,
        x,
        colon_y - COLON_SIZE - COLON_OFFSET_Y,
        x + COLON_SIZE,
        colon_y - COLON_OFFSET_Y,
    );
    fill_rect(
        hdc,
        x,
        colon_y + COLON_OFFSET_Y,
        x + COLON_SIZE,
        colon_y + COLON_SIZE + COLON_OFFSET_Y,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_masks_match_expected_patterns() {
        assert_eq!(DIGIT_MASKS[0], 0x77);
        assert_eq!(DIGIT_MASKS[1], 0x24);
        assert_eq!(DIGIT_MASKS[2], 0x5D);
        assert_eq!(DIGIT_MASKS[3], 0x6D);
        assert_eq!(DIGIT_MASKS[4], 0x2E);
        assert_eq!(DIGIT_MASKS[5], 0x6B);
        assert_eq!(DIGIT_MASKS[6], 0x7B);
        assert_eq!(DIGIT_MASKS[7], 0x25);
        assert_eq!(DIGIT_MASKS[8], 0x7F);
        assert_eq!(DIGIT_MASKS[9], 0x6F);
    }

    #[test]
    fn digit_8_is_all_segments_and_1_is_right_side_only() {
        assert_eq!(DIGIT_MASKS[8], 0x7F);
        assert!(!is_lit(0, MIDDLE));
        assert!(is_lit(1, RIGHT_UP));
        assert!(is_lit(1, RIGHT_DOWN));
        assert!(!is_lit(1, TOP));
        assert!(!is_lit(1, LEFT_UP));
    }

    #[test]
    fn clock_fits_inside_panel() {
        // 六个数字 + 两个冒号后的右边缘：20 + 5*43 + 2*15 + 33 = 298
        let right_edge =
            PAINT_START_X + 5 * DIGIT_STEP + 2 * COLON_STEP + SEG_LENGTH + 2 * SEG_WIDTH;
        assert_eq!(right_edge, 298);
        assert!(right_edge <= 310); // 面板右边缘 = 320 - 10

        let bottom_edge = PAINT_START_Y + 2 * (SEG_LENGTH + SEG_WIDTH) + SEG_WIDTH;
        assert_eq!(bottom_edge, 82);
        assert!(bottom_edge <= 90); // 面板下边缘 = 100 - 10
    }
}
