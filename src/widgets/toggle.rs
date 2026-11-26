//! 位掩码方片：描边加内填，置位画实心，刚翻转的一格从中心弹开。

use crate::gfx::Frame;

/// 方片边长。
const BOX: i16 = 9;
/// 方片之间的水平空隙。
const GAP: i16 = 3;
/// 实心方块的边长（方片内缩 2 px）。
const FILL: i16 = 5;

/// 一组方片的左边。
pub(crate) fn layout(right: i16, count: u8) -> i16 {
    let count = count.max(1) as i16;
    right - (BOX * count + GAP * (count - 1))
}

/// 第 `index` 个方片的左边。
pub(crate) fn bit_x(left: i16, index: u8) -> i16 {
    left + index as i16 * (BOX + GAP)
}

/// 一个方片，`(x, y)` 是它的左上角；`pop_q8` 是刚翻转那一格的填充进度
/// （Q8 的 0 到 256，静止时为 256）。
pub(crate) fn draw_mark(frame: &mut Frame<'_>, x: i16, y: i16, on: bool, pop_q8: i32) {
    frame.draw_rframe(x, y, BOX as u16, BOX as u16, 2);
    let progress = pop_q8.clamp(0, 384) as i16;
    let grown = FILL * progress / 256;
    let fill = if on {
        grown.min(FILL)
    } else {
        (FILL - grown).max(0)
    };
    if fill > 0 {
        let inset = (BOX - fill) / 2;
        frame.draw_box(x + inset, y + inset, fill as u16, fill as u16);
    }
}

/// 一行方片，右对齐；菜单行里当预览用。
pub(crate) fn draw(frame: &mut Frame<'_>, mask: u32, count: u8, right: i16, y: i16, height: u16) {
    let left = layout(right, count);
    let top = y + (height as i16 - BOX) / 2;
    for index in 0..count {
        draw_mark(
            frame,
            bit_x(left, index),
            top,
            mask & (1 << index) != 0,
            256,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 画一个方片，返回整块画布（16×12）。
    fn render(on: bool, pop_q8: i32) -> [u8; 32] {
        let mut buffer = [0u8; 32];
        let mut frame = Frame::new(&mut buffer, 16, 12);
        draw_mark(&mut frame, 2, 1, on, pop_q8);
        buffer
    }

    /// 一块画布里点亮的像素数。
    fn lit(buffer: &[u8; 32]) -> u32 {
        buffer.iter().map(|byte| byte.count_ones()).sum()
    }

    #[test]
    fn a_set_bit_fills_more_than_a_clear_one() {
        let clear = render(false, 256);
        let on = render(true, 256);
        assert_ne!(clear, on, "a set bit must look different");
        assert!(lit(&on) > lit(&clear), "a set bit must draw its fill");
    }

    #[test]
    fn the_pop_animation_grows_the_fill_from_the_centre() {
        assert_eq!(lit(&render(true, 0)), lit(&render(false, 256)));
        assert!(lit(&render(true, 256)) > lit(&render(true, 0)));
        assert!(lit(&render(false, 256)) < lit(&render(false, 0)));
    }

    #[test]
    fn a_row_packs_the_bits_from_the_right() {
        let mut buffer = [0u8; 64];
        let mut frame = Frame::new(&mut buffer, 32, 12);
        draw(&mut frame, 0b10, 2, 32, 0, 12);
        assert_eq!(layout(32, 2), 32 - (BOX * 2 + GAP));
        assert_eq!(bit_x(layout(32, 2), 1), 32 - BOX);
        assert!(buffer.iter().any(|byte| *byte != 0), "the row must draw");
    }
}
