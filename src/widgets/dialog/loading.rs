//! 加载弹窗：转圈指示器加一行文案，纯展示，只由应用调用 `hide_loading` 关闭。

use super::OverlayType;
use crate::model::EVENT_NONE;
use crate::ui::{Runtime, Status};
use crate::widgets::utils;

/// 指示器的 8 个点相对圆心的偏移，半径 10，自顶部起顺时针。
const SPINNER: [(i8, i8); 8] = [
    (0, -10),
    (7, -7),
    (10, 0),
    (7, 7),
    (0, 10),
    (-7, 7),
    (-10, 0),
    (-7, -7),
];

/// 指示器在每个点上停留的毫秒数，8 点转一圈约 1 秒。
const STEP_MS: u32 = 120;

/// 打开加载弹窗。
pub(crate) fn open(rt: &mut Runtime, text: &'static str, now: u32) -> Status {
    super::begin(rt, OverlayType::Loading, None, text, EVENT_NONE, now)
}

/// 加载弹窗是否正打开。
pub(crate) fn is_open(rt: &Runtime) -> bool {
    rt.overlay.kind == OverlayType::Loading
}

/// 绘制指示器与文案。
pub(crate) fn draw(rt: &mut Runtime<'_>, layout: &super::Layout) {
    let width = super::dialog_width(rt);
    let cx = layout.dx + width / 2;
    let cy = layout.dy + 17;
    let step = rt
        .last_update
        .wrapping_sub(rt.overlay.phase_started)
        .wrapping_div(STEP_MS)
        % SPINNER.len() as u32;
    for (index, (x, y)) in SPINNER.iter().enumerate() {
        let (x, y) = (cx + *x as i16, cy + *y as i16);
        // 当前点画大一号，单色屏上才分得出指示器在转。
        let radius = if index as u32 == step { 2 } else { 1 };
        rt.frame.draw_disc(x, y, radius);
    }
    utils::draw_centered(
        &mut rt.frame,
        layout.dx,
        layout.dy + 32,
        width as u16,
        rt.overlay.text,
    );
}
