//! 小图标与指示器：尖角、电池、Wi-Fi、转圈。
//!
//! 全部按整数坐标绘制，尺寸随参数缩放，和 [`Frame`] 的其它图元一样受裁剪窗口约束。

use crate::gfx::Frame;

/// 圆周方向上 16 个等分点的单位偏移，放大 256 倍（Q8）。
const DIRECTIONS: [(i16, i16); 16] = [
    (0, -256),
    (98, -236),
    (181, -181),
    (236, -98),
    (256, 0),
    (236, 98),
    (181, 181),
    (98, 236),
    (0, 256),
    (-98, 236),
    (-181, 181),
    (-236, 98),
    (-256, 0),
    (-236, -98),
    (-181, -181),
    (-98, -236),
];

/// 画一个尖角（chevron），`x` 是尖角所在列的左边缘，`cy` 是垂直中心。
///
/// `right` 为真时尖角朝右、尾巴在左；`size` 是半高，1 就退化成两个像素。
pub fn draw_chevron(frame: &mut Frame<'_>, x: i16, cy: i16, right: bool, size: i16) {
    if size <= 0 {
        return;
    }
    for step in 0..=size {
        let dx = if right { step } else { size - step };
        frame.draw_pixel(x + dx, cy - size + step);
        frame.draw_pixel(x + dx, cy + size - step);
    }
}

/// 上下方向的尖角（caret）：`x` 是尖角的左边缘，`cy` 是垂直中心，`up` 为真时朝上。
///
/// 两种朝向都以 `cy` 为中心、上下各伸 `size / 2` 像素；`size` 是半宽。
pub fn draw_caret(frame: &mut Frame<'_>, x: i16, cy: i16, up: bool, size: i16) {
    if size <= 0 {
        return;
    }
    for half in 0..=size {
        let y = if up {
            cy + size / 2 - size + half
        } else {
            cy + size / 2 - half
        };
        frame.draw_pixel(x + size - half, y);
        frame.draw_pixel(x + size + half, y);
    }
}

/// 电池图标：`x`/`y` 是左上角，`level_q12` 是 0..4096 的电量。
pub fn draw_battery(frame: &mut Frame<'_>, x: i16, y: i16, level_q12: u16) {
    frame.draw_frame(x, y, 11, 6);
    frame.draw_vline(x + 11, y + 2, 2);
    let filled = (7 * level_q12.min(4096) as i32 / 4096) as u16;
    if filled != 0 {
        frame.draw_box(x + 2, y + 2, filled, 2);
    }
}

/// Wi-Fi 图标：`x` 是左边缘，`cy` 是天线中心，`on` 为假时画一道斜杠表示断开。
pub fn draw_wifi(frame: &mut Frame<'_>, x: i16, cy: i16, on: bool) {
    frame.draw_pixel(x + 3, cy + 2);
    if !on {
        frame.draw_line(x + 1, cy - 2, x + 5, cy + 2);
        return;
    }
    frame.draw_hline(x + 2, cy, 3);
    frame.draw_pixel(x + 1, cy + 1);
    frame.draw_pixel(x + 5, cy + 1);
    frame.draw_hline(x + 1, cy - 2, 5);
    frame.draw_pixel(x, cy - 1);
    frame.draw_pixel(x + 6, cy - 1);
}

/// 转圈指示器：`phase` 每加一就前进一格，当前格画大一号。
///
/// 共 8 个点分布在半径为 `radius` 的圆周上，单色屏上靠「大点」标出朝向。
pub fn draw_spinner(frame: &mut Frame<'_>, cx: i16, cy: i16, radius: i16, phase: u8) {
    if radius <= 0 {
        return;
    }
    let leader = phase as usize % 8;
    for step in 0..8usize {
        let (sx, sy) = DIRECTIONS[step * 2];
        let px = cx + (sx as i32 * radius as i32 / 256) as i16;
        let py = cy + (sy as i32 * radius as i32 / 256) as i16;
        frame.draw_disc(px, py, if step == leader { 2 } else { 1 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 画到画布上一块区域里的像素数。
    fn lit(width: i16, height: i16, draw: impl Fn(&mut Frame<'_>)) -> usize {
        let mut buffer = [0u8; 128];
        let mut frame = Frame::new(
            &mut buffer[..Frame::buffer_len(width, height)],
            width,
            height,
        );
        draw(&mut frame);
        frame
            .buffer()
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    #[test]
    fn a_chevron_grows_with_its_size() {
        let small = lit(32, 32, |frame| draw_chevron(frame, 4, 16, true, 3));
        let large = lit(32, 32, |frame| draw_chevron(frame, 4, 16, true, 6));
        assert!(large > small, "{large} > {small}");
    }

    #[test]
    fn a_caret_grows_with_its_size() {
        let small = lit(32, 32, |frame| draw_caret(frame, 4, 16, true, 3));
        let large = lit(32, 32, |frame| draw_caret(frame, 4, 16, true, 6));
        assert!(large > small, "{large} > {small}");
    }

    /// 画出尖角，返回亮像素所在的最上、最下行。
    fn caret_rows(up: bool) -> (i16, i16) {
        let mut buffer = [0u8; 128];
        let mut frame = Frame::new(&mut buffer[..Frame::buffer_len(32, 32)], 32, 32);
        draw_caret(&mut frame, 4, 16, up, 4);
        let lit =
            |x: i16, y: i16| frame.buffer()[(y as usize / 8) * 32 + x as usize] >> (y % 8) & 1 == 1;
        let mut rows = (i16::MAX, i16::MIN);
        for y in 0..32i16 {
            if (0..32i16).any(|x| lit(x, y)) {
                rows.0 = rows.0.min(y);
                rows.1 = rows.1.max(y);
            }
        }
        rows
    }

    #[test]
    fn carets_point_apart_and_stay_centred() {
        // 以 `cy` 为中心上下对称，不偏上也不偏下。
        assert_eq!(caret_rows(true), (14, 18));
        assert_eq!(caret_rows(false), (14, 18));
    }

    #[test]
    fn a_full_battery_fills_more_than_an_empty_one() {
        let empty = lit(32, 16, |frame| draw_battery(frame, 4, 4, 0));
        let full = lit(32, 16, |frame| draw_battery(frame, 4, 4, 4096));
        assert!(full > empty, "full {full} must beat empty {empty}");
    }

    #[test]
    fn a_disconnected_wifi_draws_less_than_a_connected_one() {
        let off = lit(32, 16, |frame| draw_wifi(frame, 4, 8, false));
        let on = lit(32, 16, |frame| draw_wifi(frame, 4, 8, true));
        assert!(on > off, "on {on} must beat off {off}");
    }

    #[test]
    fn the_spinner_moves_its_leader_around() {
        let mut phases = [[0u8; 128]; 8];
        for (phase, slot) in phases.iter_mut().enumerate() {
            let mut frame = Frame::new(&mut slot[..128], 32, 32);
            draw_spinner(&mut frame, 16, 16, 10, phase as u8);
        }
        for index in 0..8usize {
            assert_ne!(
                phases[index],
                phases[(index + 1) % 8],
                "phase {index} and its successor must differ"
            );
        }
    }
}
