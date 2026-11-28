//! 移植自 KK_UI 标准 Demo 的应用层：页面表、业务状态与自绘页。
//!
//! 只依赖 `kk-ui-rs` 与 `core`，可以在主机上原样编入渲染测试工程。

mod anims;
mod autoplay;
mod gadgets;
mod handlers;
mod icons;
mod math;
mod tables;
mod wave;

pub use autoplay::Autoplay;
pub use handlers::{process_events, reset, update};
pub use tables::APP;
