//! 动图演示页：8 个 32×32 的 icons8 动图，4 个一排，按 24 fps 循环。

use kk_ui::anim_icons::{
    ACTIVITY_32, GEARS_32, GIFT_HEART_32, HEART_32, HOURGLASS_32, LOADING_BAR_32, ROCKET_32,
    TRASH_32, frame_index,
};
use kk_ui::{
    AnimIcon, CustomPageHandler, Frame, InputAction, InputEvent, PageRoute, Runtime, Shared,
    draw_anim_icon_at,
};

use super::tables::APP_NOW;

/// 页面上的 8 个动图，按绘制顺序排列。
static ICONS: [&AnimIcon; 8] = [
    &ACTIVITY_32,
    &LOADING_BAR_32,
    &ROCKET_32,
    &HOURGLASS_32,
    &HEART_32,
    &TRASH_32,
    &GEARS_32,
    &GIFT_HEART_32,
];

/// 每排的图标数。
const PER_ROW: usize = 4;
/// 图标左边距与横向步进（像素）。
const LEFT: i16 = 4;
const STEP_X: i16 = 38;
/// 第一排的上边缘与行距（像素）。
const TOP: i16 = 22;
const STEP_Y: i16 = 50;

/// 动图页的帧间状态。
#[derive(Clone, Copy, Default)]
struct AnimState {
    /// 上次画到的帧序号。
    frame: usize,
}

/// 动图演示页。
pub(super) struct Anims {
    /// 帧间状态；回调只拿 `&self`，所以放在共享单元里。
    state: Shared<AnimState>,
}

impl Anims {
    /// 初始状态。
    pub const fn new() -> Self {
        Self {
            state: Shared::new(AnimState { frame: 0 }),
        }
    }
}

impl Default for Anims {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPageHandler for Anims {
    fn on_input(&self, ui: &mut Runtime, _page: &'static PageRoute, event: InputEvent) {
        if matches!(event.action, InputAction::Ok | InputAction::Back) {
            ui.custom_request_close();
        }
    }

    fn on_tick(&self, _ui: &mut Runtime, _page: &'static PageRoute, now: u32) -> bool {
        let frame = frame_index(now);
        if frame == self.state.get().frame {
            return false;
        }
        self.state.set(AnimState { frame });
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
        let now = APP_NOW.get();
        let frame: &mut Frame<'_> = ui.frame_mut();
        let height = frame.height();
        frame.set_clip(clip_x, 0, clip_width, height as u16);
        frame.draw_text(4 + x_offset, 1, "动图");
        for (index, anim) in ICONS.iter().enumerate() {
            let x = LEFT + (index % PER_ROW) as i16 * STEP_X + x_offset;
            let y = TOP + (index / PER_ROW) as i16 * STEP_Y;
            draw_anim_icon_at(frame, x, y, anim, now);
        }
        frame.reset_clip();
    }
}
