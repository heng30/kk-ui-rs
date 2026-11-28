//! 可复用组件：图元与弹窗。
//!
//! 一个组件一个文件，全部只依赖 [`crate::gfx::Frame`]；复合组件（弹窗）自带子目录，
//! 由子目录的 `mod.rs` 会话状态与分发，具体弹窗按类型分文件。

mod focus;
mod icons;
mod progress_bar;
mod scrollbar;
mod switch;

pub mod dialog;
pub mod pages;

pub(crate) mod chip;
pub(crate) mod marquee;
pub(crate) mod roll;
pub(crate) mod toggle;
pub(crate) mod utils;

pub use focus::draw_focus;
pub use icons::{draw_battery, draw_caret, draw_chevron, draw_spinner, draw_wifi};
pub use progress_bar::draw_progress_bar;
pub use scrollbar::Scrollbar;
pub use switch::draw_switch;

#[cfg(feature = "anim-icons")]
pub mod anim_icons;

#[cfg(feature = "anim-icons")]
pub use anim_icons::{AnimIcon, IconSize, draw_anim_icon, draw_anim_icon_at};
