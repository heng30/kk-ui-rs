//! 位掩码弹窗：一列带名字的方片，上下选位、确定翻转、确定键提交。
//!
//! 列表走到尽头就把焦点交给按钮：在第一位按上、在最后一位按下或按确定都会交棒
//! （末位按确定会先翻转再交棒），所以一路按确定也能提交出去。

use super::{DialogFocus, OverlayType, Row};
use crate::model::{BitsBinding, EVENT_NONE, InputAction, InputEvent};
use crate::ui::{Runtime, Status};
use crate::widgets::toggle;

/// 取覆盖层里的位掩码绑定。
fn binding(rt: &Runtime) -> &'static BitsBinding {
    match rt.overlay.row {
        Some(Row::Bits(binding)) => binding,
        _ => panic!("overlay without bits binding"),
    }
}

/// 当前显示的行数。
fn visible(rt: &Runtime) -> i16 {
    super::bits_visible(rt)
}

/// 打开位掩码弹窗。
pub(crate) fn open(rt: &mut Runtime, entry: &'static BitsBinding, now: u32) -> Status {
    let mask = entry.value.get();
    let status = super::begin(
        rt,
        OverlayType::Bits,
        Some(Row::Bits(entry)),
        "",
        EVENT_NONE,
        now,
    );
    if status == Status::Ok {
        rt.overlay.draft = mask as i32;
        rt.overlay.original = mask as i32;
        rt.bits.pop_bit = u8::MAX;
        rt.bits.pop.jump(256);
        super::set_focus(rt, DialogFocus::Value, now);
    }
    status
}

/// 列表态输入：方向键移游标，确定翻转当前位。
pub(crate) fn edit(rt: &mut Runtime, event: InputEvent, now: u32) {
    let count = binding(rt).labels.len() as i16;
    let cursor = rt.overlay.cursor as i16;
    let steps = event.steps.min(count as u16) as i16;
    match event.action {
        InputAction::Ok => {
            flip(rt);
            if cursor + 1 >= count {
                leave(rt, DialogFocus::Confirm, now);
            }
        }
        InputAction::Up if cursor == 0 => leave(rt, DialogFocus::Cancel, now),
        InputAction::Down if cursor + 1 >= count => leave(rt, DialogFocus::Confirm, now),
        InputAction::Up => move_cursor(rt, -steps, now),
        InputAction::Down => move_cursor(rt, steps, now),
    }
}

/// 把焦点交给按钮。
fn leave(rt: &mut Runtime, focus: DialogFocus, now: u32) {
    rt.overlay.editing = false;
    super::set_focus(rt, focus, now);
}

/// 移动游标，并把它滚进可见范围。
fn move_cursor(rt: &mut Runtime, delta: i16, now: u32) {
    let last = binding(rt).labels.len() as i16 - 1;
    rt.overlay.cursor = (rt.overlay.cursor as i16 + delta).clamp(0, last) as u8;
    let rows = visible(rt) as u8;
    if rt.overlay.cursor < rt.overlay.top {
        rt.overlay.top = rt.overlay.cursor;
    } else if rt.overlay.cursor >= rt.overlay.top + rows {
        rt.overlay.top = rt.overlay.cursor + 1 - rows;
    }
    super::set_focus(rt, DialogFocus::Value, now);
}

/// 翻转游标位，方片从中心弹开。
fn flip(rt: &mut Runtime) {
    let bit = rt.overlay.cursor;
    rt.overlay.draft ^= 1 << bit;
    rt.bits.pop_bit = bit;
    rt.bits.pop.jump(0);
    rt.bits.pop.set(256);
    rt.dirty = true;
}

/// 提交草稿。
pub(crate) fn commit(rt: &mut Runtime, now: u32) {
    let entry = binding(rt);
    let draft = rt.overlay.draft;
    if draft != rt.overlay.original {
        if !super::event_or_reject(rt, entry.changed_event) {
            return;
        }
        entry.value.set(draft as u32);
    }
    super::close(rt, now);
}

/// 绘制标题、位列表与按钮。
pub(crate) fn draw(rt: &mut Runtime<'_>, layout: &super::Layout) {
    let entry = binding(rt);
    super::draw_title(rt, layout, entry.title);
    let rows = visible(rt);
    let (dx, dy) = (layout.dx, layout.dy);
    rt.frame.set_clip(15 + dx, 21 + dy, 94, (rows * 16) as u16);
    for row in 0..rows {
        let index = rt.overlay.top as i16 + row;
        let y = 21 + dy + row * 16;
        let on = rt.overlay.draft & (1 << index) != 0;
        let pop = if rt.bits.pop_bit as i16 == index {
            rt.bits.pop.value_q8()
        } else {
            256
        };
        toggle::draw_mark(&mut rt.frame, 18 + dx, y + 3, on, pop);
        rt.frame.draw_text(31 + dx, y, entry.labels[index as usize]);
    }
    layout.apply_clip(&mut rt.frame);
    super::draw_buttons(rt, layout, true);
}
