//! 时长、布局与定点常量。
//!
//! 组件用到的时长也在这里：页面与弹窗只读常量，不自己定义节奏。

use super::SpringPreset;

/// 按键消抖时间。
pub(crate) const KEY_DEBOUNCE_MS: u32 = 20;
/// 长按重复的首次延迟。
pub(crate) const KEY_REPEAT_DELAY_MS: u32 = 450;
/// 确定键按住多久算「返回」：短按确认，超过阈值后松开就是返回。
pub(crate) const KEY_OK_LONG_MS: u32 = 400;
/// 最小出帧间隔。
pub(crate) const FRAME_INTERVAL_MS: u32 = 20;
/// 首页切换动画时长。
pub(crate) const HOME_MS: u32 = 300;
/// 菜单滚动动画时长。
pub(crate) const MENU_MS: u32 = 200;
/// 页面切换动画时长。
pub(crate) const PAGE_MS: u32 = 220;
/// 信息页滚动动画时长。
pub(crate) const INFO_SCROLL_MS: u32 = 180;
/// 弹窗打开动画时长。
pub(crate) const DIALOG_OPEN_MS: u32 = 180;
/// 弹窗关闭动画时长。
pub(crate) const DIALOG_CLOSE_MS: u32 = 150;
/// 焦点框补间时长。
pub(crate) const FOCUS_MS: u32 = 160;
/// 整数换值动画时长。
pub(crate) const INT_VALUE_MS: u32 = 100;
/// 开关换值动画时长。
pub(crate) const BOOL_VALUE_MS: u32 = 120;
/// Toast 滑入滑出时长。
pub(crate) const TOAST_MS: u32 = 120;
/// Q12 定点数的 1.0。
pub(crate) const Q12_ONE: u16 = 4096;
/// 菜单标题栏高度：1 px 顶部留白 + 16 px 字形 + 1 px 底部留白。
pub(crate) const MENU_HEADER_HEIGHT: i16 = 18;
/// 信息页标题栏高度：同菜单，容纳 16 px 字形。
pub(crate) const INFO_HEADER_HEIGHT: i16 = 18;
/// 列表行高：一个 16×16 字形的占位高度。
pub(crate) const ROW_HEIGHT: i16 = 16;
/// 列表条目之间的垂直空隙（像素）。
pub(crate) const ROW_GAP: i16 = 2;
/// 列表行距：行高加空隙，菜单与信息页的滚动/命中/滚动条都按它算。
pub(crate) const ROW_PITCH: i16 = ROW_HEIGHT + ROW_GAP;
/// 页面切换时旧页面的视差位移。
pub(crate) const PAGE_PARALLAX: i16 = 32;
/// 页哈希最多跟踪的页数；超过这个高度的画布退回「整帧提交」。
///
/// shortcut: 上限 32 页（256 像素高），用在更高的屏上要改成动态存储。
pub(crate) const PAGE_HASH_MAX: usize = 32;

/// 弹簧积分的固定子步长（毫秒）。
pub(crate) const SPRING_SUBSTEP_MS: i32 = 8;
/// 一帧最多积分的子步数。
pub(crate) const SPRING_MAX_SUBSTEPS: i32 = 4;
/// 轻快：响应 0.26 s、阻尼比 0.5，分段选择框滑动用。
pub(crate) const SPRING_CHIP: SpringPreset = SpringPreset::new(19132, 792);
/// 弹跳：响应 0.45 s、阻尼比 0.5，游标框与位方片翻转用。
pub(crate) const SPRING_BOUNCY: SpringPreset = SpringPreset::new(6385, 458);
/// 跑马灯滚到一端后停留的时长（毫秒）。
pub(crate) const MARQUEE_HOLD_MS: u32 = 900;
/// 跑马灯匀速移动的速度（像素每秒）。
pub(crate) const MARQUEE_SPEED_PX_S: i32 = 24;
