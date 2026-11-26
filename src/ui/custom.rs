//! 自绘页适配器与回调调度。

use super::{KkUi, Runtime};
use crate::model::{InputEvent, PageId, PageType};

/// 自绘页适配器：应用实现它来接管 `PageType::Custom` 页面。
pub trait CustomPage {
    /// 进入页面。
    fn on_enter(&mut self, ui: &mut Runtime<'_>, page: PageId) {
        let _ = (ui, page);
    }

    /// 离开页面；在切换动画结束后调用。
    fn on_leave(&mut self, ui: &mut Runtime<'_>, page: PageId) {
        let _ = (ui, page);
    }

    /// 处理输入。
    fn on_input(&mut self, ui: &mut Runtime<'_>, page: PageId, event: InputEvent) {
        let _ = (ui, page, event);
    }

    /// 逐帧推进；返回 `true` 表示需要重画。
    fn on_tick(&mut self, ui: &mut Runtime<'_>, page: PageId, now: u32) -> bool {
        let _ = (ui, page, now);
        false
    }

    /// 绘制页面；`x_offset` 为视差位移，`clip_x`/`clip_width` 是可见区域。
    fn on_draw(
        &mut self,
        ui: &mut Runtime<'_>,
        page: PageId,
        x_offset: i16,
        clip_x: i16,
        clip_width: u16,
    ) {
        let _ = (ui, page, x_offset, clip_x, clip_width);
    }
}

impl<C: CustomPage> KkUi<'_, C> {
    /// 把待发的进入/离开事件送进自绘页回调。
    pub(super) fn flush_custom(&mut self) {
        while let Some(page) = self.rt.enter_custom.take() {
            let Self { rt, custom } = self;
            rt.in_callback = true;
            custom.on_enter(rt, page);
            rt.in_callback = false;
        }
        while let Some(page) = self.rt.leave_custom.take() {
            let Self { rt, custom } = self;
            rt.in_callback = true;
            custom.on_leave(rt, page);
            rt.in_callback = false;
        }
    }

    /// 逐帧推进当前自绘页；回调返回 `true` 时要求重画。
    pub(super) fn custom_tick(&mut self, now: u32) {
        if self.rt.current_page_type() != PageType::Custom || self.rt.transition.active {
            return;
        }
        let Self { rt, custom } = self;
        let page = rt.current.page;
        rt.in_callback = true;
        let redraw = custom.on_tick(rt, page, now);
        rt.in_callback = false;
        if redraw {
            rt.dirty = true;
        }
    }
}
