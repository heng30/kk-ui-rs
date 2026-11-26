//! KK_UI 的 Rust 移植：单色菜单 UI 运行时。
//!
//! 结构对应上游 C 版：首页图标选择器、可嵌套菜单、只读信息页、应用自绘页
//! （[`CustomPageHandler`]）、整数/开关编辑器、确认框、消息框与 Toast。
//! 画布与 KK_OLED 一致是页式 1 bit 帧缓冲，尺寸由调用方给定（[`Frame::new`]），
//! 由应用负责把它放大送到真实屏幕。
//!
//! 相比 C 版没有 `KK_UI_ENABLE_*` 编译开关：所有页面与弹窗都编译，未用到的部分由
//! release 的 `lto = "fat"` + gc-sections 从最终镜像里剥掉，不必手写 cfg。也没移植
//! 异步 IT/DMA 刷新路径：渲染结果直接落在 [`Frame`]，提交由应用阻塞完成。

#![no_std]

// 模拟器跑在宿主机的 std 上，只有 `sim` 特性才把它拉进来。
#[cfg(feature = "sim")]
extern crate std;

mod font;
mod gfx;
mod model;
mod ui;
mod widgets;

#[cfg(feature = "cjk-fonts")]
pub mod cjk;
#[cfg(feature = "sim")]
pub mod sim;

pub use font::{FULL_WIDTH, GLYPH_LEN, GLYPH_SIZE, Glyphs, set_font};
pub use gfx::{DrawMode, Frame};
pub use model::{
    App, BitsBinding, BoolBinding, ChipBinding, ChoiceBinding, ConfirmDesc, CustomPage, EVENT_NONE,
    EventId, HomeItem, HomePage, InfoPage, InfoRow, Input, InputAction, InputEvent, InputSource,
    IntBinding, ItemState, KEY_DOWN, KEY_OK, KEY_UP, LINE_INPUT_LEN, LineInput, LineInputBinding,
    MAX_BITS, MenuAction, MenuItem, MenuPage, PageRoute, PageType, Shared, Text, Texts,
};
pub use ui::{CustomPageHandler, ErrorInfo, KkUi, Runtime, Status};
pub use widgets::{draw_focus, draw_progress_bar, draw_switch, utils::format_int};

/// 按声明顺序为事件生成从 1 开始的编号常量。
///
/// 只能写在模块作用域：生成的是 `pub const`。
///
/// ```ignore
/// kk_ui::event_ids! {
///     /// 事件：亮度改变。
///     EVENT_BRIGHTNESS_CHANGED,
///     EVENT_SWITCH_CHANGED,
/// }
/// ```
#[macro_export]
macro_rules! event_ids {
    ($($(#[$attr:meta])* $name:ident),* $(,)?) => {
        $crate::event_ids!(@step 0u16; $($(#[$attr])* $name),*);
    };
    (@step $next:expr;) => {};
    (@step $next:expr; $(#[$attr:meta])* $name:ident $(, $(#[$rest_attr:meta])* $rest:ident)* $(,)?) => {
        $(#[$attr])* pub const $name: $crate::EventId = $next + 1;
        $crate::event_ids!(@step $next + 1u16; $($(#[$rest_attr])* $rest),*);
    };
}

/// 批量声明 [`Shared`] 静态单元，写法是 `名字: 类型 = 初值`。
///
/// 只能写在模块作用域：生成的是 `pub static`。
///
/// ```ignore
/// kk_ui::shared! {
///     BRIGHTNESS: i32 = 80,
///     DEMO_SWITCH: bool = true,
/// }
/// ```
#[macro_export]
macro_rules! shared {
    ($($(#[$attr:meta])* $name:ident : $ty:ty = $value:expr),* $(,)?) => {
        $(
            $(#[$attr])*
            pub static $name: $crate::Shared<$ty> = $crate::Shared::new($value);
        )*
    };
}

/// 批量声明 [`Text`] 静态文案，写法是 `名字 = "字面量"`。
///
/// ```ignore
/// kk_ui::texts! {
///     UPTIME = "0s",
///     DEVICE_INFO = "KK_UI",
/// }
/// ```
#[macro_export]
macro_rules! texts {
    ($($(#[$attr:meta])* $name:ident = $text:literal),* $(,)?) => {
        $(
            $(#[$attr])*
            pub static $name: $crate::Text = $crate::Text::literal($text);
        )*
    };
}

/// 批量声明菜单条目状态表，写法是 `名字 = [初值, ...]`，每字节压 4 个 2 bit 状态。
///
/// 只能写在模块作用域：生成的是定长 `pub static` 数组。`Shared` 带内部可变性，
/// 不能靠借用临时切片来声明，所以长度由宏自己数出来。
///
/// ```ignore
/// kk_ui::menu_states! {
///     SETTINGS_STATES = [0, 0],
/// }
/// ```
#[macro_export]
macro_rules! menu_states {
    ($($(#[$attr:meta])* $name:ident = [$($value:expr),* $(,)?]),* $(,)?) => {
        $(
            $(#[$attr])*
            pub static $name: [$crate::Shared<u8>; $crate::menu_states!(@len $($value),*)] =
                [$($crate::Shared::new($value)),*];
        )*
    };
    (@len) => { 0usize };
    (@len $head:expr $(, $tail:expr)*) => { 1usize + $crate::menu_states!(@len $($tail),*) };
}

/// 批量声明图标静态表，写法是 `名字 = "资源路径"`。
///
/// `include_bytes!` 的路径相对**调用该宏的文件**解析。
///
/// ```ignore
/// kk_ui::icons! {
///     ICON_SETTINGS = "assets/icon0.bin",
/// }
/// ```
#[macro_export]
macro_rules! icons {
    ($($(#[$attr:meta])* $name:ident = $path:literal),* $(,)?) => {
        $(
            $(#[$attr])*
            pub static $name: &[u8] = include_bytes!($path);
        )*
    };
}
