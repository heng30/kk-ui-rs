//! 枚举选择弹窗：草稿是标签表下标，新旧标签纵向滚动切换。
//!
//! 与整数编辑器相反，这里往下是选下一个标签——标签是列表，不是数值。

use super::{Overlay, OverlayType, Row};
use crate::gfx::Frame;
use crate::model::{ChoiceBinding, EVENT_NONE, InputAction, InputEvent};
use crate::ui::{Q12_ONE, Runtime, Status};
use crate::widgets::utils::draw_centered;

/// 取覆盖层里的枚举绑定。
fn binding(rt: &Runtime) -> &'static ChoiceBinding {
    match rt.overlay.row {
        Some(Row::Choice(binding)) => binding,
        _ => panic!("overlay without choice binding"),
    }
}

/// 标签表里的第 `index` 个标签；下标越界时为空串。
fn label(binding: &ChoiceBinding, index: i32) -> &'static str {
    binding
        .labels
        .get(index.max(0) as usize)
        .copied()
        .unwrap_or("")
}

/// 打开枚举选择弹窗。
pub(crate) fn open(rt: &mut Runtime, entry: &'static ChoiceBinding, now: u32) -> Status {
    let value = entry.value.get() as i32;
    let status = super::begin(
        rt,
        OverlayType::Choice,
        Some(Row::Choice(entry)),
        "",
        EVENT_NONE,
        now,
    );
    if status == Status::Ok {
        rt.overlay.draft = value;
        rt.overlay.original = value;
        rt.overlay.previous = value;
    }
    status
}

/// 换标签：往下选下一个，往上选上一个，夹在标签表范围内。
pub(crate) fn change(rt: &mut Runtime, event: InputEvent, now: u32) {
    let entry = binding(rt);
    let forward = event.action == InputAction::Down;
    let steps = event.steps as i32;
    let last = entry.labels.len() as i32 - 1;
    let draft = rt.overlay.draft;
    let next = if forward {
        (draft + steps).min(last)
    } else {
        (draft - steps).max(0)
    };
    super::apply_value(rt, next, forward, now);
}

/// 提交枚举结果。
pub(crate) fn commit(rt: &mut Runtime, now: u32) {
    let entry = binding(rt);
    let draft = rt.overlay.draft;
    if draft != rt.overlay.original {
        if !super::event_or_reject(rt, entry.changed_event) {
            return;
        }
        entry.value.set(draft as u16);
    }
    super::close(rt, now);
}

/// 绘制标题、标签与按钮；编辑态额外画上下箭头。
pub(crate) fn draw(rt: &mut Runtime<'_>, layout: &super::Layout) {
    let overlay = rt.overlay;
    let entry = binding(rt);
    super::draw_title(rt, layout, entry.title);
    draw_value(&overlay, entry, &mut rt.frame, layout.dx, layout.dy);
    super::draw_buttons(rt, layout, true);
    if rt.overlay.editing {
        let (dx, dy) = (layout.dx, layout.dy);
        if rt.overlay.draft < entry.labels.len() as i32 - 1 {
            rt.frame.draw_line(91 + dx, 26 + dy, 95 + dx, 22 + dy);
            rt.frame.draw_line(95 + dx, 22 + dy, 99 + dx, 26 + dy);
        }
        if rt.overlay.draft > 0 {
            rt.frame.draw_line(91 + dx, 31 + dy, 95 + dx, 35 + dy);
            rt.frame.draw_line(95 + dx, 35 + dy, 99 + dx, 31 + dy);
        }
    }
}

/// 枚举选择：新旧标签纵向滚动切换。
fn draw_value(overlay: &Overlay, entry: &ChoiceBinding, frame: &mut Frame<'_>, dx: i16, dy: i16) {
    let previous = label(entry, overlay.previous);
    let draft = label(entry, overlay.draft);
    frame.set_clip(39 + dx, 21 + dy, 46, 16);
    if overlay.value_moving {
        let offset = (16 * overlay.value_q12 as i32 / Q12_ONE as i32) as i16;
        if overlay.value_direction > 0 {
            draw_centered(frame, 39 + dx, 21 + dy - offset, 46, previous);
            draw_centered(frame, 39 + dx, 37 + dy - offset, 46, draft);
        } else {
            draw_centered(frame, 39 + dx, 21 + dy + offset, 46, previous);
            draw_centered(frame, 39 + dx, 5 + dy + offset, 46, draft);
        }
    } else {
        draw_centered(frame, 39 + dx, 21 + dy, 46, draft);
    }
}
