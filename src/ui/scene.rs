//! 整屏场景绘制：页面切换、拒绝闪烁、弹窗与 Toast。

use super::{KkUi, PAGE_MS, PAGE_PARALLAX, PageState, Q12_ONE, ease_q12};
use crate::gfx::DrawMode;
use crate::model::PageType;
use crate::widgets::{dialog, pages};

impl KkUi<'_> {
    /// 按页面类型绘制一页；`x_offset` 为视差位移，`clip_x`/`clip_width` 是可见区域。
    fn draw_page(
        &mut self,
        state: PageState,
        is_current: bool,
        x_offset: i16,
        clip_x: i16,
        clip_width: u16,
    ) {
        let Some(route) = self.rt.route(state.page) else {
            return;
        };
        match route.kind {
            PageType::Home => pages::home_draw(
                &mut self.rt,
                state,
                is_current,
                x_offset,
                clip_x,
                clip_width,
            ),
            PageType::Menu => pages::menu_draw(
                &mut self.rt,
                state,
                is_current,
                x_offset,
                clip_x,
                clip_width,
            ),
            PageType::Info => pages::info_draw(
                &mut self.rt,
                state,
                is_current,
                x_offset,
                clip_x,
                clip_width,
            ),
            PageType::Custom => {
                if let Some(desc) = self.custom_page(state.page) {
                    self.rt.in_callback = true;
                    desc.on_draw(&mut self.rt, state.page, x_offset, clip_x, clip_width);
                    self.rt.in_callback = false;
                }
            }
        }
    }

    /// 清屏并重画整个场景：页面、切换动画、拒绝闪烁、弹窗与 Toast。
    pub(super) fn draw_scene(&mut self) {
        self.rt.frame.clear();
        self.rt.frame.reset_clip();
        self.rt.frame.set_draw_mode(DrawMode::Set);
        let now = self.rt.last_update;
        let width = self.rt.frame.width();
        let height = self.rt.frame.height();

        if self.rt.transition.active {
            let progress = ease_q12(now.wrapping_sub(self.rt.transition.started), PAGE_MS);
            let outgoing = self.rt.transition.outgoing;
            let current = self.rt.current;
            let boundary;
            if !self.rt.transition.reverse {
                let outgoing_x =
                    -((PAGE_PARALLAX as i32 * progress as i32) / Q12_ONE as i32) as i16;
                let current_x =
                    (width as i32 * (Q12_ONE as i32 - progress as i32) / Q12_ONE as i32) as i16;
                boundary = current_x;
                if boundary > 0 {
                    self.draw_page(outgoing, false, outgoing_x, 0, boundary as u16);
                }
                if boundary < width {
                    self.draw_page(
                        current,
                        true,
                        current_x,
                        boundary,
                        (width - boundary) as u16,
                    );
                }
            } else {
                let outgoing_x = (PAGE_PARALLAX as i32 * progress as i32 / Q12_ONE as i32) as i16;
                boundary = (width as i32 * progress as i32 / Q12_ONE as i32) as i16;
                let current_x = boundary - width;
                if boundary > 0 {
                    self.draw_page(current, true, current_x, 0, boundary as u16);
                }
                if boundary < width {
                    self.draw_page(
                        outgoing,
                        false,
                        outgoing_x,
                        boundary,
                        (width - boundary) as u16,
                    );
                }
            }
            if (0..width).contains(&boundary) {
                self.rt.frame.reset_clip();
                self.rt.frame.draw_vline(boundary, 0, height as u16);
            }
        } else {
            self.draw_page(self.rt.current, true, 0, 0, width as u16);
        }

        if self.rt.reject_flash > 1 {
            self.rt.frame.set_draw_mode(DrawMode::Xor);
            self.rt
                .frame
                .draw_rframe(1, 1, (width - 2) as u16, (height - 2) as u16, 3);
            self.rt.frame.set_draw_mode(DrawMode::Set);
            self.rt.reject_flash = 1;
            self.rt.dirty = true;
        } else if self.rt.reject_flash != 0 {
            self.rt.reject_flash = 0;
        }
        dialog::draw(&mut self.rt);
        dialog::draw_toast(&mut self.rt);
        self.rt.frame.reset_clip();
        self.rt.frame.set_draw_mode(DrawMode::Set);
    }
}
