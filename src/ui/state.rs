//! 页面与补间的会话状态。

use crate::model::PageId;

/// 单页的滚动与焦点状态。
#[derive(Clone, Copy, Default)]
pub(crate) struct PageState {
    /// 页面编号。
    pub page: PageId,
    /// 选中条目下标，等于条目数时表示虚拟返回项。
    pub selected: u16,
    /// 首个可见行的序号。
    pub top: u16,
    /// 滚动量，Q8 定点（1 行 = 16 * 256）。
    pub scroll_q8: i32,
    /// 焦点行相对首个可见行的偏移，Q8 定点。
    pub focus_q8: i32,
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
