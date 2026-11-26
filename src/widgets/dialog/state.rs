//! 弹窗与贴底提示条的会话状态。
//!
//! 这些状态由 [`crate::ui::Runtime`] 持有，读写都在 `widgets::dialog` 里。

use crate::model::{
    BitsBinding, BoolBinding, ChoiceBinding, ConfirmDesc, EVENT_NONE, EventId, IntBinding,
    LINE_INPUT_LEN, LineInputBinding,
};
use crate::ui::Q12_ONE;

/// 覆盖层类型。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverlayType {
    /// 无覆盖层。
    None,
    /// 整数编辑器。
    Int,
    /// 开关编辑器。
    Bool,
    /// 枚举选择弹窗。
    Choice,
    /// 位掩码弹窗。
    Bits,
    /// 文本编辑器。
    LineInput,
    /// 确认框。
    Confirm,
    /// 消息框。
    Message,
    /// 加载弹窗。
    Loading,
}

/// 弹窗焦点。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogFocus {
    /// 取值区域。
    Value,
    /// 确定按钮。
    Confirm,
    /// 取消按钮。
    Cancel,
}

/// 弹窗操作的表项句柄。
#[derive(Clone, Copy)]
pub(crate) enum Row {
    /// 整数编辑器绑定。
    Int(&'static IntBinding),
    /// 开关编辑器绑定。
    Bool(&'static BoolBinding),
    /// 枚举选择绑定。
    Choice(&'static ChoiceBinding),
    /// 位掩码绑定。
    Bits(&'static BitsBinding),
    /// 文本绑定。
    LineInput(&'static LineInputBinding),
    /// 确认框描述。
    Confirm(&'static ConfirmDesc),
}

/// 覆盖层状态。
#[derive(Clone, Copy)]
pub(crate) struct Overlay {
    /// 当前覆盖层类型。
    pub kind: OverlayType,
    /// 编辑器或确认框操作的表项；消息框为 `None`。
    pub row: Option<Row>,
    /// 正文；确认框与消息框使用。
    pub text: &'static str,
    /// 关闭时要入队的事件。
    pub event: EventId,
    /// 编辑器草稿值。
    pub draft: i32,
    /// 打开时的原值，用于判断是否改动。
    pub original: i32,
    /// 上一个草稿值，用于上下列滚动动画。
    pub previous: i32,
    /// 当前焦点矩形，Q8 定点 `(x, y, w, h)`。
    pub focus: (i32, i32, i32, i32),
    /// 补间起点的焦点矩形。
    pub focus_from: (i32, i32, i32, i32),
    /// 焦点位置。
    pub focus_mode: DialogFocus,
    /// 开/关动画开始的时戳。
    pub phase_started: u32,
    /// 焦点补间开始的时戳。
    pub focus_started: u32,
    /// 换值动画开始的时戳。
    pub value_started: u32,
    /// 弹窗出现时对齐的行号。
    pub origin_y: i16,
    /// 开合进度，Q12。
    pub phase_q12: u16,
    /// 关闭动画起点的开合进度，Q12。
    pub phase_from_q12: u16,
    /// 换值动画进度，Q12。
    pub value_q12: u16,
    /// 换值方向，`1` 递增、`-1` 递减。
    pub value_direction: i8,
    /// 是否处于取值编辑态。
    pub editing: bool,
    /// 位掩码弹窗里游标所在的位。
    pub cursor: u8,
    /// 位掩码弹窗里第一个可见的位。
    pub top: u8,

    /// 文本编辑缓冲。
    pub buf: [u8; LINE_INPUT_LEN],
    /// 缓冲内的有效字节数。
    pub buf_len: u8,
    /// 软键盘光标所在行。
    pub kb_row: u8,
    /// 软键盘光标在该行的第几格，0 与末格是换行按钮。
    pub kb_col: u8,
    /// 软键盘是否处于大写锁定。
    pub caps: bool,
    /// 软键盘是否处于数字符号层。
    pub symbols: bool,

    /// 是否正在播关闭动画。
    pub closing: bool,
    /// 焦点框是否正在补间。
    pub focus_moving: bool,
    /// 取值是否正在换值动画。
    pub value_moving: bool,
}

impl Overlay {
    /// 没有任何覆盖层时的初始状态。
    pub(crate) const EMPTY: Self = Self {
        kind: OverlayType::None,
        row: None,
        text: "",
        event: EVENT_NONE,
        draft: 0,
        original: 0,
        previous: 0,
        focus: (0, 0, 0, 0),
        focus_from: (0, 0, 0, 0),
        focus_mode: DialogFocus::Cancel,
        phase_started: 0,
        focus_started: 0,
        value_started: 0,
        origin_y: 0,
        phase_q12: 0,
        phase_from_q12: 0,
        value_q12: Q12_ONE,
        value_direction: 0,
        editing: false,
        cursor: 0,
        top: 0,
        buf: [0; LINE_INPUT_LEN],
        buf_len: 0,
        kb_row: 0,
        kb_col: 0,
        caps: false,
        symbols: false,
        closing: false,
        focus_moving: false,
        value_moving: false,
    };
}

/// Toast 状态。
#[derive(Clone, Copy)]
pub(crate) struct Toast {
    /// 提示文本。
    pub text: &'static str,
    /// 滑入动画开始的时戳。
    pub shown_at: u32,
    /// 到期时刻，到达后开始滑出。
    pub expires_at: u32,
    /// 滑入进度，Q12。
    pub phase_q12: u16,
    /// 是否可见。
    pub visible: bool,
    /// 是否正在滑出。
    pub closing: bool,
}
