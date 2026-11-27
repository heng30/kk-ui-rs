//! 图标自绘页：电池、无线、箭头与转圈四个绘制助手，转圈随时间转。

use kk_ui::{
    CustomPageHandler, Frame, InputAction, InputEvent, PageRoute, Runtime, Shared, draw_battery,
    draw_chevron, draw_spinner, draw_wifi,
};

use super::tables::APP_NOW;

/// 转圈每格停留的毫秒数。
const SPIN_STEP_MS: u32 = 120;

/// 图标页的帧间状态。
#[derive(Clone, Copy)]
struct IconsState {
    /// 转圈走到第几格。
    phase: u8,
    /// 进入页面的时间戳，毫秒。
    entered_at: u32,
}

impl IconsState {
    /// 初始状态。
    const fn new() -> Self {
        Self {
            phase: 0,
            entered_at: 0,
        }
    }
}

/// 图标自绘页。
pub(super) struct Icons {
    /// 帧间状态；回调只拿 `&self`，所以放在共享单元里。
    state: Shared<IconsState>,
}

impl Icons {
    /// 初始状态。
    pub const fn new() -> Self {
        Self {
            state: Shared::new(IconsState::new()),
        }
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPageHandler for Icons {
    fn on_enter(&self, _ui: &mut Runtime, _page: &'static PageRoute) {
        let mut state = IconsState::new();
        state.entered_at = APP_NOW.get();
        self.state.set(state);
    }

    fn on_input(&self, ui: &mut Runtime, _page: &'static PageRoute, event: InputEvent) {
        if matches!(event.action, InputAction::Ok | InputAction::Back) {
            ui.custom_request_close();
        }
    }

    fn on_tick(&self, _ui: &mut Runtime, _page: &'static PageRoute, now: u32) -> bool {
        let mut state = self.state.get();
        let phase = (now.wrapping_sub(state.entered_at) / SPIN_STEP_MS % 8) as u8;
        if phase == state.phase {
            return false;
        }
        state.phase = phase;
        self.state.set(state);
        true
    }

    fn on_draw(
        &self,
        ui: &mut Runtime,
        _page: &'static PageRoute,
        x_offset: i16,
        clip_x: i16,
        clip_width: u16,
    ) {
        let state = self.state.get();
        let frame: &mut Frame<'_> = ui.frame_mut();
        let (width, height) = (frame.width(), frame.height());
        frame.set_clip(clip_x, 0, clip_width, height as u16);
        frame.draw_text(4 + x_offset, 1, "图标");

        frame.draw_text(4 + x_offset, 22, "电池");
        for (index, level) in [0u16, 2048, 4096].into_iter().enumerate() {
            draw_battery(frame, 50 + index as i16 * 22 + x_offset, 27, level);
        }

        frame.draw_text(4 + x_offset, 46, "无线");
        draw_wifi(frame, 50 + x_offset, 52, true);
        draw_wifi(frame, 72 + x_offset, 52, false);

        frame.draw_text(4 + x_offset, 70, "箭头");
        draw_chevron(frame, 52 + x_offset, 78, false, 5);
        draw_chevron(frame, 72 + x_offset, 78, true, 5);

        frame.draw_text(4 + x_offset, 94, "转圈");
        draw_spinner(frame, width - 24 + x_offset, 102, 8, state.phase);
        frame.reset_clip();
    }
}
