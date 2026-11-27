//! 文本编辑器弹窗：预览行加四行软键盘，上下键在行内移动，确定按下键。
//!
//! 每行两端是上下箭头的换行按钮：面板只有上、下、确定三个键，行内靠上下键横着走，
//! 换行靠按确定踩这两个按钮，退格只在字符区的最后一行。字母层与数字符号层各有一套字符表，`C` 是锁定式
//! 大写，`123`/`abc` 切层。屏幕矮到放不下四行完整键时字形会被裁剪，见 [`keyboard_area`]。

use super::{OverlayType, Row};
use crate::gfx::{DrawMode, Frame};
use crate::model::{EVENT_NONE, InputAction, InputEvent, LINE_INPUT_LEN, LineInputBinding};
use crate::ui::{Runtime, Status};

/// 键盘一个键的动作。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Key {
    /// 输入这个字符。
    Char(char),
    /// 切换大写锁定。
    Shift,
    /// 切换字母层与数字符号层。
    Layer,
    /// 输入空格。
    Space,
    /// 删除末尾一个字符。
    Backspace,
    /// 丢弃改动并关闭。
    Cancel,
    /// 提交并关闭。
    Done,
    /// 跳到上一行的同一格。
    RowUp,
    /// 跳到下一行的同一格。
    RowDown,
}

/// 字母层每行字符。
const LETTER_ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
/// 数字符号层每行字符。
const SYMBOL_ROWS: [&str; 3] = ["1234567890", "-/:;()$&@", "=.,?!'\"#"];

/// 键盘行数。
const KB_ROWS: u8 = 4;
/// 功能行「123/abc」占的格子数。
const LAYER_SPAN: u8 = 2;
/// 功能行空格占的格子数。
const SPACE_SPAN: u8 = 2;
/// 功能行「esc」占的格子数。
const CANCEL_SPAN: u8 = 2;
/// 功能行「ok」占的格子数。
const DONE_SPAN: u8 = 2;
/// 功能行四个键的跨度。
const FOOTER_SPANS: [u8; 4] = [LAYER_SPAN, SPACE_SPAN, CANCEL_SPAN, DONE_SPAN];
/// 功能行中间四个键占的格子总数。
const FOOTER_SLOTS: u8 = LAYER_SPAN + SPACE_SPAN + CANCEL_SPAN + DONE_SPAN;
/// 每个换行/退格按钮占的格子数。
const EDGE_SPAN: u8 = 1;
/// 每行固定占用在中间键之外的列数：左端朝上箭头、右端朝下箭头。
const EDGE_COLUMNS: u8 = 2;

/// 预览行高度，含上下边框。
const PREVIEW_HEIGHT: i16 = 18;
/// 键区四周留白。
const KB_MARGIN: i16 = 2;

/// 取覆盖层里的文本绑定。
fn binding(rt: &Runtime) -> &'static LineInputBinding {
    match rt.overlay.row {
        Some(Row::LineInput(binding)) => binding,
        _ => panic!("overlay without text binding"),
    }
}

/// 打开文本编辑器，把绑定里的文字拷进编辑缓冲。
pub(crate) fn open(rt: &mut Runtime, entry: &'static LineInputBinding, now: u32) -> Status {
    let status = super::begin(
        rt,
        OverlayType::LineInput,
        Some(Row::LineInput(entry)),
        "",
        EVENT_NONE,
        now,
    );
    if status == Status::Ok {
        let mut buf = [0u8; LINE_INPUT_LEN];
        let len = entry.value.with(|text| {
            let len = text.len().min(LINE_INPUT_LEN);
            buf[..len].copy_from_slice(&text.as_bytes()[..len]);
            len
        });
        rt.overlay.buf = buf;
        rt.overlay.buf_len = len as u8;
        rt.overlay.kb_row = 0;
        // 第 0 格是换行按钮，光标从第一个字符键开始。
        rt.overlay.kb_col = 1;
        rt.overlay.caps = false;
        rt.overlay.symbols = false;
    }
    status
}

/// 一行中间有几个键（不含两端的换行按钮）。
fn middle_len(symbols: bool, row: u8) -> u8 {
    let table = if symbols { &SYMBOL_ROWS } else { &LETTER_ROWS };
    match row {
        0 | 1 => table[row as usize].len() as u8,
        2 => table[2].len() as u8 + if symbols { 1 } else { 2 },
        _ => FOOTER_SPANS.len() as u8,
    }
}

/// 一行有多少格：中间键数加两端的换行按钮。
fn row_slots(symbols: bool, row: u8) -> u8 {
    middle_len(symbols, row) + EDGE_COLUMNS
}

/// 第 `row` 行第 `col` 格的键。
fn key_at(symbols: bool, row: u8, col: u8) -> Key {
    let middle = middle_len(symbols, row);
    if col == 0 {
        return Key::RowUp;
    }
    if col > middle {
        return Key::RowDown;
    }
    let index = col - 1;
    let table = if symbols { &SYMBOL_ROWS } else { &LETTER_ROWS };
    match row {
        0 | 1 => table[row as usize]
            .chars()
            .nth(index as usize)
            .map_or(Key::Done, Key::Char),
        2 if symbols => {
            if index as usize >= table[2].len() {
                Key::Backspace
            } else {
                Key::Char(table[2].chars().nth(index as usize).unwrap())
            }
        }
        2 => match index {
            0 => Key::Shift,
            index if index as usize > table[2].len() => Key::Backspace,
            _ => Key::Char(table[2].chars().nth(index as usize - 1).unwrap()),
        },
        _ => match index {
            0 => Key::Layer,
            1 => Key::Space,
            2 => Key::Cancel,
            _ => Key::Done,
        },
    }
}

/// 键区矩形 `(x, y, width, height)`。
///
/// shortcut: 行高按屏幕平均分，16 px 字形在 96 px 以下高的屏上会被裁剪，接矮屏时
/// 应改为更小的字库或压缩到三行键。
fn keyboard_area(frame: &Frame<'_>) -> (i16, i16, i16, i16) {
    let left = KB_MARGIN;
    let top = PREVIEW_HEIGHT + KB_MARGIN;
    let width = frame.width() - KB_MARGIN * 2;
    // 预览占掉顶部，剩下的四行平分。
    let height = (frame.height() - top - KB_MARGIN).max(KB_ROWS as i16);
    (left, top, width, height)
}

/// 一个键的矩形 `(x, y, width, height)`。
fn key_rect(rt: &Runtime, symbols: bool, row: u8, col: u8) -> (i16, i16, i16, i16) {
    let (left, top, width, height) = keyboard_area(&rt.frame);
    let row_height = height / KB_ROWS as i16;
    let y = top + row as i16 * row_height;
    if row == 3 {
        // 功能行两端各留一格换行按钮，中间按跨度分配。
        let slot = width / (EDGE_SPAN * 2 + FOOTER_SLOTS) as i16;
        if col == 0 {
            return (left, y, slot, row_height);
        }
        if col > FOOTER_SPANS.len() as u8 {
            return (
                left + slot * (EDGE_SPAN + FOOTER_SLOTS) as i16,
                y,
                slot,
                row_height,
            );
        }
        let mut x = left + slot * EDGE_SPAN as i16;
        for span in FOOTER_SPANS.iter().take(col as usize - 1) {
            x += slot * *span as i16;
        }
        let span = FOOTER_SPANS[col as usize - 1];
        (x, y, slot * span as i16, row_height)
    } else {
        let slots = row_slots(symbols, row).max(1) as i16;
        let slot = width / slots;
        (left + col as i16 * slot, y, slot, row_height)
    }
}

/// 处理输入：上下键在本行内移动光标，确定按下光标下的键。
pub(crate) fn input(rt: &mut Runtime, event: InputEvent, now: u32) {
    let steps = event.steps.min(127) as i16;
    match event.action {
        // 长按确定在键盘上就是按一下键，返回只有「esc」和面板的返回键。
        InputAction::Ok | InputAction::Back => press(rt, now),
        InputAction::Up => move_cursor(rt, -steps),
        InputAction::Down => move_cursor(rt, steps),
    }
}

/// 在本行的格子之间循环移动光标。
fn move_cursor(rt: &mut Runtime, delta: i16) {
    let slots = row_slots(rt.overlay.symbols, rt.overlay.kb_row).max(1) as i16;
    rt.overlay.kb_col = (rt.overlay.kb_col as i16 + delta).rem_euclid(slots) as u8;
    rt.dirty = true;
}

/// 跳到相邻行，尽量停在原来的格子上。
fn change_row(rt: &mut Runtime, delta: i8) {
    let symbols = rt.overlay.symbols;
    let row = (rt.overlay.kb_row as i8 + delta).rem_euclid(KB_ROWS as i8) as u8;
    let slots = row_slots(symbols, row).max(1);
    rt.overlay.kb_row = row;
    rt.overlay.kb_col = rt.overlay.kb_col.min(slots - 1);
    rt.dirty = true;
}

/// 按下光标下的键。
fn press(rt: &mut Runtime, now: u32) {
    let symbols = rt.overlay.symbols;
    let key = key_at(symbols, rt.overlay.kb_row, rt.overlay.kb_col);
    match key {
        Key::Char(c) => {
            let c = if rt.overlay.caps {
                c.to_ascii_uppercase()
            } else {
                c
            };
            push(rt, c);
        }
        Key::Space => push(rt, ' '),
        Key::Backspace => {
            let len = rt.overlay.buf_len as usize;
            if len != 0 {
                let mut end = len - 1;
                while end > 0 && rt.overlay.buf[end] & 0xC0 == 0x80 {
                    end -= 1;
                }
                for byte in &mut rt.overlay.buf[end..len] {
                    *byte = 0;
                }
                rt.overlay.buf_len = end as u8;
                rt.dirty = true;
            }
        }
        Key::Shift => {
            rt.overlay.caps = !rt.overlay.caps;
            rt.dirty = true;
        }
        Key::Layer => {
            rt.overlay.symbols = !rt.overlay.symbols;
            let slots = row_slots(rt.overlay.symbols, rt.overlay.kb_row).max(1);
            rt.overlay.kb_col = rt.overlay.kb_col.min(slots - 1);
            rt.dirty = true;
        }
        Key::RowUp => change_row(rt, -1),
        Key::RowDown => change_row(rt, 1),
        Key::Cancel => super::close(rt, now),
        Key::Done => commit(rt, now),
    }
}

/// 在编辑缓冲末尾追加一个字符。
fn push(rt: &mut Runtime, c: char) {
    let mut encoded = [0u8; 4];
    let encoded = c.encode_utf8(&mut encoded).as_bytes();
    let len = rt.overlay.buf_len as usize;
    if len + encoded.len() > LINE_INPUT_LEN {
        return;
    }
    rt.overlay.buf[len..len + encoded.len()].copy_from_slice(encoded);
    rt.overlay.buf_len = (len + encoded.len()) as u8;
    rt.dirty = true;
}

/// 提交编辑缓冲：有改动时入队事件并写回绑定，然后关闭。
fn commit(rt: &mut Runtime, now: u32) {
    let entry = binding(rt);
    let buf = rt.overlay.buf;
    let text = core::str::from_utf8(&buf[..rt.overlay.buf_len as usize]).unwrap_or("");
    if entry.value.with(|old| old != text) {
        if !super::event_or_reject(rt, entry.changed_event) {
            return;
        }
        entry.value.set(text);
    }
    super::close(rt, now);
}

/// 绘制预览行与键盘。
pub(crate) fn draw(rt: &mut Runtime<'_>, layout: &super::Layout) {
    let dx = super::dialog_left(rt);
    let dy = super::dialog_top(rt);
    draw_preview(rt, dx, dy);
    let outer = rt.frame.clip();
    for row in 0..KB_ROWS {
        let (_, y, _, height) = key_rect(rt, rt.overlay.symbols, row, 0);
        rt.frame
            .set_clip(outer.0, y, (outer.2 - outer.0) as u16, height.max(1) as u16);
        for col in 0..row_slots(rt.overlay.symbols, row) {
            draw_key(rt, row, col);
        }
    }
    rt.frame.set_clip(
        outer.0,
        outer.1,
        (outer.2 - outer.0) as u16,
        (outer.3 - outer.1) as u16,
    );
    let (x, y, width, height) =
        key_rect(rt, rt.overlay.symbols, rt.overlay.kb_row, rt.overlay.kb_col);
    rt.frame.set_draw_mode(DrawMode::Xor);
    rt.frame
        .draw_rbox(x, y, width.max(1) as u16, height.max(1) as u16, 2);
    rt.frame.set_draw_mode(DrawMode::Set);
    let _ = layout;
}

/// 预览行：带圆角边框的当前文字，末尾跟一个光标，超宽时改为右对齐。
fn draw_preview(rt: &mut Runtime<'_>, dx: i16, dy: i16) {
    let width = rt.frame.width() - 4;
    rt.frame.draw_rframe(dx + 2, dy + 2, width as u16, 16, 2);
    let mut buf = [0u8; LINE_INPUT_LEN + 1];
    let len = rt.overlay.buf_len as usize;
    buf[..len].copy_from_slice(&rt.overlay.buf[..len]);
    buf[len] = b'_';
    let text = core::str::from_utf8(&buf[..len + 1]).unwrap_or("");
    let available = width - 4;
    let text_width = rt.frame.text_width(text) as i16;
    let x = if text_width <= available {
        dx + 4
    } else {
        dx + 2 + available - text_width
    };
    let clip = rt.frame.clip();
    rt.frame.set_clip(dx + 3, dy + 3, (width - 2) as u16, 14);
    rt.frame.draw_text(x, dy + 3, text);
    rt.frame.set_clip(
        clip.0,
        clip.1,
        (clip.2 - clip.0) as u16,
        (clip.3 - clip.1) as u16,
    );
}

/// 画一个键：字符居中，大写锁定时字母改大写，`C` 加下划线表示锁定中。
fn draw_key(rt: &mut Runtime<'_>, row: u8, col: u8) {
    let (x, y, width, height) = key_rect(rt, rt.overlay.symbols, row, col);
    let key = key_at(rt.overlay.symbols, row, col);
    let mut buf = [0u8; 4];
    let text = match key {
        Key::Char(c) => {
            let c = if rt.overlay.caps {
                c.to_ascii_uppercase()
            } else {
                c
            };
            c.encode_utf8(&mut buf)
        }
        Key::Shift => "C",
        Key::Layer if rt.overlay.symbols => "abc",
        Key::Layer => "123",
        Key::Space => "sp",
        Key::Backspace => "<",
        Key::Cancel => "esc",
        Key::Done => "ok",
        Key::RowUp | Key::RowDown => {
            let (cx, cy) = (x + width / 2, y + height / 2);
            crate::widgets::draw_caret(&mut rt.frame, cx - 3, cy, key == Key::RowUp, 3);
            return;
        }
    };
    let text_width = rt.frame.text_width(text) as i16;
    let tx = x + (width - text_width) / 2;
    let ty = y + (height - crate::GLYPH_SIZE as i16) / 2;
    rt.frame.draw_text(tx, ty, text);
    if key == Key::Shift && rt.overlay.caps {
        rt.frame.draw_hline(tx, ty + 15, text_width as u16);
    }
}
