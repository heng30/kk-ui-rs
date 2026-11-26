//! 分段单选行：右列 N 段文字，选中段套一个会滑动的异或圆角框。

use crate::gfx::{DrawMode, Frame};
use crate::ui::round_q8;

/// 每段之间的水平空隙。
const CHIP_GAP: i16 = 2;
/// 段内左右各留的空白。
const CHIP_PADDING: i16 = 2;

/// 分段行的几何：组左边与每段宽度；每段等宽，滑动框才能只用一个位置表示。
pub(crate) fn layout(frame: &Frame<'_>, levels: &[&str], right: i16) -> (i16, i16) {
    let mut widest = 0i16;
    for level in levels {
        widest = widest.max(frame.text_width(level) as i16);
    }
    let segment = widest + CHIP_PADDING * 2;
    let count = levels.len() as i16;
    let width = segment * count + CHIP_GAP * (count - 1).max(0);
    (right - width, segment)
}

/// 第 `index` 段的左边。
pub(crate) fn segment_x(left: i16, segment: i16, index: u8) -> i16 {
    left + index as i16 * (segment + CHIP_GAP)
}

/// 画分段行，返回组的左边；`pill_q8` 是选中框左边的 Q8 像素，滑动动画由调用方推进。
pub(crate) fn draw(
    frame: &mut Frame<'_>,
    levels: &[&str],
    right: i16,
    y: i16,
    height: u16,
    pill_q8: i32,
) -> i16 {
    let (left, segment) = layout(frame, levels, right);
    for (index, level) in levels.iter().enumerate() {
        let x = segment_x(left, segment, index as u8);
        let text = frame.text_width(level) as i16;
        frame.draw_text(x + (segment - text) / 2, y, level);
    }
    frame.set_draw_mode(DrawMode::Xor);
    frame.draw_rbox(round_q8(pill_q8), y, segment as u16, height, 3);
    frame.set_draw_mode(DrawMode::Set);
    left
}
