//! 页面与补间的会话状态。

use super::Spring;
use crate::model::PageRoute;
use crate::widgets::roll::Roll;

/// 单页的滚动与焦点状态。
#[derive(Clone, Copy, Default)]
pub(crate) struct PageState {
    /// 页面句柄；尚未初始化时为 `None`。
    pub page: Option<&'static PageRoute>,
    /// 选中条目下标，等于条目数时表示虚拟返回项。
    pub selected: u16,
    /// 首个可见行的序号。
    pub top: u16,
    /// 滚动量，Q8 定点（1 行 = 16 * 256）。
    pub scroll_q8: i32,
    /// 焦点行相对首个可见行的偏移，Q8 定点。
    pub focus_q8: i32,
}

impl PageState {
    /// 页面句柄；调用方保证页面已就绪。
    pub(crate) fn route(&self) -> &'static PageRoute {
        self.page.expect("page state without route")
    }
}

/// 列表滚动与焦点补间的起点。
#[derive(Clone, Copy, Default)]
pub(crate) struct ListAnimation {
    /// 补间起点的滚动量，Q8。
    pub(crate) scroll_from_q8: i32,
    /// 补间起点的焦点偏移，Q8。
    pub(crate) focus_from_q8: i32,
    /// 动画开始的时戳。
    pub(crate) started: u32,
    /// 动画总时长（毫秒）。
    pub(crate) duration: u32,
    /// 是否正在播放。
    pub(crate) active: bool,
}

/// 首页横向补间的起点。
#[derive(Clone, Copy, Default)]
pub(crate) struct HomeAnimation {
    /// 补间起点的图标横向偏移，Q8。
    pub(crate) offset_from_q8: i32,
    /// 补间起点的标签框宽度，Q8。
    pub(crate) label_from_q8: i32,
    /// 补间终点的标签框宽度，Q8。
    pub(crate) label_to_q8: i32,
    /// 动画开始的时戳。
    pub(crate) started: u32,
    /// 是否正在播放。
    pub(crate) active: bool,
}

/// 分段选择行的滑动框状态。
#[derive(Clone, Copy)]
pub(crate) struct ChipAnimation {
    /// 对应的菜单条目下标；换行时用它判断要不要瞬移。
    pub(crate) row: u16,
    /// 选中框左边的位置，Q8 像素。
    pub(crate) pill: Spring,
}

impl ChipAnimation {
    /// 没有任何一行选中时的状态。
    pub(crate) const EMPTY: Self = Self {
        row: u16::MAX,
        pill: Spring::REST,
    };
}

/// 位掩码弹窗里刚翻转那一格的弹开反馈。
#[derive(Clone, Copy)]
pub(crate) struct BitsAnimation {
    /// 填充进度，0 到 1 是 Q8 的 0 到 256。
    pub(crate) pop: Spring,
    /// 刚翻转的位；`u8::MAX` 表示没有。
    pub(crate) pop_bit: u8,
}

impl BitsAnimation {
    /// 静止状态。
    pub(crate) const EMPTY: Self = Self {
        pop: Spring::at(256),
        pop_bit: u8::MAX,
    };
}

/// 跑马灯状态。
///
/// 偏移是匀速的，所以不存位置：每次用它按阶段起点现算，掉帧也不会追不上。
#[derive(Clone, Copy)]
pub(crate) struct MarqueeState {
    /// 正在滚动的条目下标。
    pub(crate) row: u16,
    /// 当前阶段，见 `widgets::marquee`。
    pub(crate) phase: u8,
    /// 当前阶段的起始时戳。
    pub(crate) started: u32,
}

impl MarqueeState {
    /// 静止状态。
    pub(crate) const EMPTY: Self = Self {
        row: u16::MAX,
        phase: 0,
        started: 0,
    };
}

/// 菜单右列取值的滚动数字：只滚当前焦点行。
///
/// shortcut: 非焦点行的取值变化直接跳字，取值只在焦点行编辑，够用；
/// 要支持背景行同时滚动，得改成按行存的数组。
#[derive(Clone, Copy)]
pub(crate) struct MenuRoll {
    /// 这份状态对应的条目下标。
    pub(crate) row: u16,
    /// 滚动状态。
    pub(crate) roll: Roll,
}

impl MenuRoll {
    /// 静止状态。
    pub(crate) const EMPTY: Self = Self {
        row: u16::MAX,
        roll: Roll::EMPTY,
    };
}
