//! 自绘页适配器与回调调度。

use super::{KkUi, Runtime};
use crate::model::{CustomPage, InputEvent, PageRoute, PageType};

/// 自绘页适配器：应用实现它，并用 `PageRoute::Custom` 挂进 [`App::routes`](crate::model::App)
/// 接管自绘页。
///
/// 回调只拿到 `&self`，可变状态请放在 `Cell`/`Shared` 这类内部可变性容器里。
pub trait CustomPageHandler {
    /// 进入页面。
    fn on_enter(&self, ui: &mut Runtime<'_>, page: &'static PageRoute) {
        let _ = (ui, page);
    }

    /// 离开页面；在切换动画结束后调用。
    fn on_leave(&self, ui: &mut Runtime<'_>, page: &'static PageRoute) {
        let _ = (ui, page);
    }

    /// 处理输入。
    fn on_input(&self, ui: &mut Runtime<'_>, page: &'static PageRoute, event: InputEvent) {
        let _ = (ui, page, event);
    }

    /// 逐帧推进；返回 `true` 表示需要重画。
    fn on_tick(&self, ui: &mut Runtime<'_>, page: &'static PageRoute, now: u32) -> bool {
        let _ = (ui, page, now);
        false
    }

    /// 绘制页面；`x_offset` 为视差位移，`clip_x`/`clip_width` 是可见区域。
    fn on_draw(
        &self,
        ui: &mut Runtime<'_>,
        page: &'static PageRoute,
        x_offset: i16,
        clip_x: i16,
        clip_width: u16,
    ) {
        let _ = (ui, page, x_offset, clip_x, clip_width);
    }
}

impl KkUi<'_> {
    /// 取自绘页句柄对应的实现。
    pub(super) fn custom_page(page: &'static PageRoute) -> Option<CustomPage> {
        match page {
            PageRoute::Custom(handler) => Some(*handler),
            _ => None,
        }
    }

    /// 把待发的进入/离开事件送进自绘页回调；先离开旧页，再进入新页。
    pub(super) fn flush_custom(&mut self) {
        while let Some(page) = self.rt.leave_custom.take() {
            if let Some(desc) = Self::custom_page(page) {
                self.rt.in_callback = true;
                desc.on_leave(&mut self.rt, page);
                self.rt.in_callback = false;
            }
        }
        while let Some(page) = self.rt.enter_custom.take() {
            if let Some(desc) = Self::custom_page(page) {
                self.rt.in_callback = true;
                desc.on_enter(&mut self.rt, page);
                self.rt.in_callback = false;
            }
        }
    }

    /// 逐帧推进当前自绘页；回调返回 `true` 时要求重画。
    pub(super) fn custom_tick(&mut self, now: u32) {
        if self.rt.current_page_type() != PageType::Custom || self.rt.transition.active {
            return;
        }
        let Some(page) = self.rt.current.page else {
            return;
        };
        let Some(desc) = Self::custom_page(page) else {
            return;
        };
        self.rt.in_callback = true;
        let redraw = desc.on_tick(&mut self.rt, page, now);
        self.rt.in_callback = false;
        if redraw {
            self.rt.dirty = true;
        }
    }
}
