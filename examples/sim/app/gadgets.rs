//! 控件自绘页：进度条、开关、滚动条与焦点框四个纯绘制助手，取值随时间来回走。

use kk_ui::{
    CustomPageHandler, Frame, InputAction, InputEvent, PageRoute, Runtime, Scrollbar, Shared,
    draw_focus, draw_progress_bar, draw_switch,
};

use super::tables::APP_NOW;

/// 每格停留的毫秒数。
const STEP_MS: u32 = 120;
/// 一个来回的格数。
const STEPS: u32 = 200;
/// 滚动条的内容高度（像素）。
const SCROLL_CONTENT: i32 = 200;

/// 控件页的帧间状态。
#[derive(Clone, Copy)]
struct GadgetsState {
    /// 当前进度，0..=100。
    phase: u16,
    /// 进入页面的时间戳，毫秒。
    entered_at: u32,
}

impl GadgetsState {
    /// 初始状态。
    const fn new() -> Self {
        Self {
            phase: 0,
            entered_at: 0,
        }
    }
}

/// 控件自绘页。
pub(super) struct Gadgets {
    /// 帧间状态；回调只拿 `&self`，所以放在共享单元里。
    state: Shared<GadgetsState>,
}

impl Gadgets {
    /// 初始状态。
    pub const fn new() -> Self {
        Self {
            state: Shared::new(GadgetsState::new()),
        }
    }
}

impl Default for Gadgets {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPageHandler for Gadgets {
    fn on_enter(&self, _ui: &mut Runtime, _page: &'static PageRoute) {
        let mut state = GadgetsState::new();
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
        let phase = gadgets_phase(now.wrapping_sub(state.entered_at));
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
        let phase = self.state.get().phase;
        let frame: &mut Frame<'_> = ui.frame_mut();
        let (width, height) = (frame.width(), frame.height());
        frame.set_clip(clip_x, 0, clip_width, height as u16);
        frame.draw_text(4 + x_offset, 1, "控件");

        let bar_width = (width - 52).max(5) as u16;
        frame.draw_text(4 + x_offset, 22, "进度");
        draw_progress_bar(frame, 44 + x_offset, 24, bar_width, 10, phase, 100);

        let knob = (phase as u32 * 4096 / 100) as u16;
        frame.draw_text(4 + x_offset, 46, "开关");
        draw_switch(frame, 44 + x_offset, 46, 26, 12, true, knob);
        draw_switch(frame, 76 + x_offset, 46, 26, 12, false, 4096 - knob);

        let track = (height - 100).clamp(8, 24);
        frame.draw_text(4 + x_offset, 70, "滚动");
        frame.draw_frame(44 + x_offset, 72, bar_width, track as u16);
        Scrollbar {
            x: 44 + x_offset + bar_width as i16 - 5,
            y: 72,
            track,
            scroll: phase as i32 * (SCROLL_CONTENT - track as i32) / 100,
            content: SCROLL_CONTENT,
            viewport: track,
            thickness: 4,
            min_thumb: 6,
            line: true,
        }
        .draw(frame);

        frame.draw_text(4 + x_offset, height - 20, "焦点");
        draw_focus(frame, 44 + x_offset, height - 20, bar_width, 16, 4);
        frame.reset_clip();
    }
}

/// 按进入页面后的毫秒数算出一个来回的进度，0..=100。
fn gadgets_phase(elapsed: u32) -> u16 {
    let step = (elapsed / STEP_MS) % STEPS;
    if step <= STEPS / 2 {
        (step * 100 / (STEPS / 2)) as u16
    } else {
        ((STEPS - step) * 100 / (STEPS / 2)) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_ping_pongs_between_bounds() {
        assert_eq!(gadgets_phase(0), 0);
        assert_eq!(gadgets_phase(STEP_MS * 100), 100);
        assert_eq!(gadgets_phase(STEP_MS * 200), 0);
        assert!(gadgets_phase(STEP_MS * 150) < 100);
    }
}
