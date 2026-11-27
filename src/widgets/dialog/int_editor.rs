//! 整数编辑器弹窗：草稿值按步长增减，取值逐位滚动。

use super::{OverlayType, Row};
use crate::model::{EVENT_NONE, InputAction, InputEvent, IntBinding};
use crate::ui::{Runtime, Status};
use crate::widgets::utils::{BUFFER_LEN, format_int};

/// 取覆盖层里的整数绑定。
fn binding(rt: &Runtime) -> &'static IntBinding {
    match rt.overlay.row {
        Some(Row::Int(binding)) => binding,
        _ => panic!("overlay without int binding"),
    }
}

/// 打开整数编辑器。
pub(crate) fn open(rt: &mut Runtime, entry: &'static IntBinding, now: u32) -> Status {
    let value = entry.value.get();
    let status = super::begin(
        rt,
        OverlayType::Int,
        Some(Row::Int(entry)),
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

/// 按步长调整整数草稿。
fn adjust(binding: &IntBinding, value: i32, increase: bool, steps: u16) -> i32 {
    let amount = binding.step as i64 * steps as i64;
    let next = value as i64 + if increase { amount } else { -amount };
    next.clamp(binding.minimum as i64, binding.maximum as i64) as i32
}

/// 改值：方向决定增减，结果夹在绑定的取值范围内。
pub(crate) fn change(rt: &mut Runtime, event: InputEvent, now: u32) {
    let increase = event.action == InputAction::Up;
    let draft = rt.overlay.draft;
    let next = adjust(binding(rt), draft, increase, event.steps);
    super::apply_value(rt, next, increase, now);
}

/// 提交编辑器结果。
pub(crate) fn commit(rt: &mut Runtime, now: u32) {
    let entry = binding(rt);
    let draft = rt.overlay.draft;
    if draft != rt.overlay.original {
        if !super::event_or_reject(rt, entry.changed_event) {
            return;
        }
        entry.value.set(draft);
    }
    super::close(rt, now);
}

/// 绘制标题、取值与按钮；编辑态额外画上下箭头。
pub(crate) fn draw(rt: &mut Runtime<'_>, layout: &super::Layout) {
    let entry = binding(rt);
    super::draw_title(rt, layout, entry.title);
    draw_value(rt, entry.unit, layout);
    super::draw_buttons(rt, layout, true);
    if rt.overlay.editing {
        let (dx, dy) = (layout.dx, layout.dy);
        if rt.overlay.draft < entry.maximum {
            rt.frame.draw_line(91 + dx, 26 + dy, 95 + dx, 22 + dy);
            rt.frame.draw_line(95 + dx, 22 + dy, 99 + dx, 26 + dy);
        }
        if rt.overlay.draft > entry.minimum {
            rt.frame.draw_line(91 + dx, 31 + dy, 95 + dx, 35 + dy);
            rt.frame.draw_line(95 + dx, 35 + dy, 99 + dx, 31 + dy);
        }
    }
}

/// 整数编辑器：取值逐位滚动，单位跟着数字一起滚。
fn draw_value(rt: &mut Runtime<'_>, unit: &str, layout: &super::Layout) {
    let mut buffer = [0u8; BUFFER_LEN];
    let text = format_int(rt.overlay.draft, unit, &mut buffer);
    let dir = if rt.overlay.value_direction < 0 {
        -1
    } else {
        1
    };
    rt.overlay.roll.set(text, dir, rt.last_update);
    let width = rt.overlay.roll.width() as i16;
    let x = 39 + layout.dx + (46 - width) / 2;
    rt.frame.set_clip(39 + layout.dx, 21 + layout.dy, 46, 16);
    rt.overlay
        .roll
        .draw(&mut rt.frame, x, 21 + layout.dy, rt.last_update);
}
