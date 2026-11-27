//! 滚动条：一条轨道加一根按比例定位的滑块。
//!
//! 菜单页与信息页原先各写一份几乎相同的几何计算，这里收成一处。所有量都是像素，
//! 滚动位置与内容高度由调用方从自己的 Q8 状态换算好再传进来。

use crate::gfx::Frame;

/// 一根滚动条的几何参数。
#[derive(Clone, Copy)]
pub struct Scrollbar {
    /// 滑块左边缘。
    pub x: i16,
    /// 轨道顶端。
    pub y: i16,
    /// 轨道高度（像素）。
    pub track: i16,
    /// 已滚动距离（像素）。
    pub scroll: i32,
    /// 内容总高度（像素）。
    pub content: i32,
    /// 视口高度（像素）。
    pub viewport: i16,
    /// 滑块宽度（像素）。
    pub thickness: u16,
    /// 滑块最短高度（像素）。
    pub min_thumb: i16,
    /// 是否画出贯穿轨道的细线。
    pub line: bool,
}

impl Scrollbar {
    /// 内容装得下时什么也不画；其余情况按比例画出滑块。
    pub fn draw(&self, frame: &mut Frame<'_>) {
        let track = self.track as i32;
        let content = self.content.max(0);
        if self.track <= 0 || self.thickness == 0 || content <= self.viewport as i32 {
            return;
        }
        if self.line {
            frame.draw_vline(
                self.x + self.thickness as i16 - 1,
                self.y,
                self.track as u16,
            );
        }
        let maximum = content - self.viewport as i32;
        let thumb = (track * self.viewport as i32 / content).max(self.min_thumb as i32);
        let thumb = thumb.min(track);
        let offset = if maximum <= 0 {
            0
        } else {
            ((track - thumb) * self.scroll.max(0).min(maximum) / maximum).clamp(0, track - thumb)
        };
        frame.draw_box(self.x, self.y + offset as i16, self.thickness, thumb as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 画完之后的亮像素数。
    fn lit(bar: Scrollbar) -> usize {
        let mut buffer = [0u8; 128];
        let mut frame = Frame::new(&mut buffer[..Frame::buffer_len(32, 32)], 32, 32);
        bar.draw(&mut frame);
        frame
            .buffer()
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    /// 同一条滚动条，滚动到不同位置的滑块位置。
    fn thumb_y(scroll: i32) -> i16 {
        let mut buffer = [0u8; 128];
        let mut frame = Frame::new(&mut buffer[..Frame::buffer_len(32, 32)], 32, 32);
        let bar = Scrollbar {
            x: 28,
            y: 0,
            track: 32,
            scroll,
            content: 64,
            viewport: 32,
            thickness: 3,
            min_thumb: 4,
            line: true,
        };
        bar.draw(&mut frame);
        (0..32)
            .find(|&row| frame.pixel(28, row))
            .expect("the thumb must be visible")
    }

    #[test]
    fn a_short_list_draws_nothing() {
        assert_eq!(
            lit(Scrollbar {
                x: 28,
                y: 0,
                track: 32,
                scroll: 0,
                content: 20,
                viewport: 32,
                thickness: 3,
                min_thumb: 4,
                line: true,
            }),
            0
        );
    }

    #[test]
    fn the_thumb_travels_from_top_to_bottom() {
        assert_eq!(thumb_y(0), 0);
        assert_eq!(thumb_y(32), 16, "thumb of half the track stops at half");
    }

    #[test]
    fn scrolling_past_the_end_keeps_the_thumb_inside() {
        assert_eq!(thumb_y(9999), 16);
    }
}
