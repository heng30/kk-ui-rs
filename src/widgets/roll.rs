//! 滚动数字：逐位垂直滚动，多位同时变化时从右往左错峰。
//!
//! 状态按「每个字符从哪来、滚到哪、什么时候起跑」保存，位置每次按时间现算，
//! 所以掉帧也不会越滚越偏。

use crate::font::{GLYPH_SIZE, char_width, draw_char};
use crate::gfx::Frame;
use crate::ui::{Curve, curve_q12};

/// 最多同时滚动的字符数。
pub(crate) const ROLL_MAX: usize = 8;
/// 单个字符滚完的时长（毫秒）。
const ROLL_MS: u32 = 300;
/// 相邻两位之间的错峰（毫秒）。
const STAGGER_MS: u32 = 30;
/// 上下两行字形之间的步进。
const STEP: i16 = GLYPH_SIZE as i16 + 2;

/// 一个字符位的滚动状态。
#[derive(Clone, Copy)]
struct Cell {
    /// 原来的字符。
    from: char,
    /// 目标字符。
    to: char,
    /// 滚动方向，正数表示新字符从下方进来。
    dir: i8,
    /// 这一位开始滚动的时刻。
    started: u32,
}

impl Cell {
    /// 空位：内容一致，不参与动画。
    const IDLE: Self = Self {
        from: '\0',
        to: '\0',
        dir: 1,
        started: 0,
    };

    /// 距离自己起跑还有多久（毫秒）；已起跑或没有动画时为 0。
    fn elapsed(&self, now: u32) -> i32 {
        (now.wrapping_sub(self.started) as i32).max(0)
    }

    /// 是否还在滚动。
    fn busy(&self, now: u32) -> bool {
        self.from != self.to && self.elapsed(now) < ROLL_MS as i32
    }
}

/// 一行会逐位滚动的文字。
#[derive(Clone, Copy)]
pub(crate) struct Roll {
    /// 各位的状态。
    cells: [Cell; ROLL_MAX],
    /// 有效字符数。
    len: u8,
}

impl Roll {
    /// 空状态。
    pub(crate) const EMPTY: Self = Self {
        cells: [Cell::IDLE; ROLL_MAX],
        len: 0,
    };

    /// 直接跳到新文本，不播动画。
    pub(crate) fn reset(&mut self, text: &str, now: u32) {
        self.len = 0;
        for c in text.chars().take(ROLL_MAX) {
            self.cells[self.len as usize] = Cell {
                from: c,
                to: c,
                dir: 1,
                started: now,
            };
            self.len += 1;
        }
        for slot in self.cells.iter_mut().skip(self.len as usize) {
            *slot = Cell::IDLE;
        }
    }

    /// 滚到新文本；字符数变了就直接跳过去。
    pub(crate) fn set(&mut self, text: &str, dir: i8, now: u32) {
        let mut chars = ['\0'; ROLL_MAX];
        let mut count = 0usize;
        for c in text.chars() {
            if count == ROLL_MAX {
                break;
            }
            chars[count] = c;
            count += 1;
        }
        if count != self.len as usize {
            self.reset(text, now);
            return;
        }
        let mut rank = 0u32;
        for index in (0..count).rev() {
            let c = chars[index];
            let cell = &mut self.cells[index];
            if cell.to == c {
                continue;
            }
            cell.from = cell.to;
            cell.to = c;
            cell.dir = if dir < 0 { -1 } else { 1 };
            cell.started = now.wrapping_add(rank * STAGGER_MS);
            rank += 1;
        }
    }

    /// 这一行是否还有字符在滚动。
    pub(crate) fn busy(&self, now: u32) -> bool {
        self.cells[..self.len as usize]
            .iter()
            .any(|cell| cell.busy(now))
    }

    /// 目标文本的像素宽度。
    pub(crate) fn width(&self) -> u16 {
        self.cells[..self.len as usize]
            .iter()
            .map(|cell| char_width(cell.to))
            .sum()
    }

    /// 从左端 `x` 开始绘制，返回目标文本的右端；滚动中的旧字会被裁在行内。
    pub(crate) fn draw(&self, frame: &mut Frame<'_>, x: i16, y: i16, now: u32) -> i16 {
        let saved = frame.clip();
        // 纵向只留一行字形高度，横向交给调用方的窗口。
        let left = (x - 1).max(saved.0);
        let top = y.max(saved.1);
        let right = saved.2;
        let bottom = (y + GLYPH_SIZE as i16).min(saved.3);
        if left < right && top < bottom {
            frame.set_clip(left, top, (right - left) as u16, (bottom - top) as u16);
        }
        let mut cx = x;
        for cell in &self.cells[..self.len as usize] {
            if !cell.busy(now) {
                draw_char(frame, cx, y, cell.to);
            } else {
                let p = curve_q12(Curve::Overshoot, cell.elapsed(now) as u32, ROLL_MS) as i32;
                let offset = (p * STEP as i32 * cell.dir as i32 / 4096) as i16;
                draw_char(frame, cx, y - offset, cell.from);
                draw_char(frame, cx, y + STEP * cell.dir as i16 - offset, cell.to);
            }
            cx += char_width(cell.to) as i16;
        }
        frame.set_clip(
            saved.0,
            saved.1,
            (saved.2 - saved.0).max(0) as u16,
            (saved.3 - saved.1).max(0) as u16,
        );
        cx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resetting_a_line_leaves_nothing_to_animate() {
        let mut roll = Roll::EMPTY;
        roll.reset("42", 0);
        assert!(!roll.busy(0));
        assert_eq!(roll.len, 2);
    }

    #[test]
    fn a_changed_digit_starts_rolling() {
        let mut roll = Roll::EMPTY;
        roll.reset("42", 0);
        roll.set("43", 1, 100);
        assert!(roll.busy(100), "the last digit must be moving");
        assert!(!roll.busy(100 + ROLL_MS), "and settle within one roll");
    }

    #[test]
    fn several_digits_are_staggered_from_the_right() {
        let mut roll = Roll::EMPTY;
        roll.reset("999", 0);
        roll.set("000", 1, 1000);
        // 最右边先起跑，最左边要等两位的错峰。
        assert_eq!(roll.cells[2].started, 1000);
        assert_eq!(roll.cells[1].started, 1000 + STAGGER_MS);
        assert_eq!(roll.cells[0].started, 1000 + 2 * STAGGER_MS);
        assert!(roll.busy(1000), "rightmost rolls first");
        assert!(
            roll.busy(1000 + 2 * STAGGER_MS),
            "leftmost is still waiting for its slot"
        );
        assert!(
            !roll.busy(1000 + 2 * STAGGER_MS + ROLL_MS),
            "everything settles after the last slot"
        );
    }

    #[test]
    fn a_different_length_jumps_instead_of_rolling() {
        let mut roll = Roll::EMPTY;
        roll.reset("42", 0);
        roll.set("100", 1, 0);
        assert!(!roll.busy(0), "length changes must not animate");
        assert_eq!(roll.len, 3);
    }
}
