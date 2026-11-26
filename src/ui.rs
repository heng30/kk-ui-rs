//! KK_UI 运行时与对外接口：页面栈、输入消抖、动画调度、事件队列与延迟请求。
//!
//! [`Runtime`] 的状态在 `runtime`，一项能力一个文件：`status` 结果与错误、`timing`
//! 常量、`anim` 缓动、`state` 页面状态、`input` 按键、`nav` 导航、`deferred` 弹窗入口
//! 与延迟请求、`events` 事件队列、`validate` 描述表校验、`custom` 自绘页回调、`scene`
//! 整屏绘制。弹窗与 Toast 的会话状态在 [`crate::widgets::dialog`]。

mod anim;
mod custom;
mod deferred;
mod events;
mod input;
mod nav;
mod runtime;
mod scene;
mod state;
mod status;
mod timing;
mod validate;

use crate::gfx::Frame;
use crate::model::{App, EventId, Input, ItemState, PageId};
use crate::widgets::{dialog, pages};

pub(crate) use anim::{ease_q12, lerp_q12, round_q8};
pub use custom::CustomPage;
pub(crate) use deferred::{Deferred, DeferredType};
pub(crate) use events::EVENT_QUEUE_LENGTH;
pub use runtime::Runtime;
pub(crate) use state::{HomeAnimation, ListAnimation, PageState};
pub use status::{ErrorInfo, Status};
pub(crate) use timing::*;

/// UI 本体：运行时 + 自绘页适配器。
pub struct KkUi<'a, C: CustomPage> {
    /// 运行时状态。
    pub(crate) rt: Runtime<'a>,
    /// 应用提供的自绘页实现。
    pub(crate) custom: C,
}

impl<'a, C: CustomPage> KkUi<'a, C> {
    /// 绑定应用描述表、自绘页实现与一块 `width * ceil(height / 8)` 字节的帧缓冲。
    pub fn new(
        app: &'static App,
        custom: C,
        buffer: &'a mut [u8],
        width: i16,
        height: i16,
    ) -> Self {
        Self {
            rt: Runtime::new(app, buffer, width, height),
            custom,
        }
    }

    /// 校验配置，回到根页面并清空导航栈、覆盖层与动画。
    pub fn init(&mut self) -> Status {
        let status = self.rt.init();
        self.flush_custom();
        status
    }

    /// 推进一帧：动画、输入、延迟请求、重画。
    pub fn update(&mut self, now: u32, input: Input) -> Status {
        if !self.rt.initialized {
            return Status::NotInitialized;
        }
        if !self.rt.keys_initialized {
            self.rt.next_frame = now;
        }
        self.rt.last_update = now;
        self.rt.animate_navigation(now);
        pages::home_animate(&mut self.rt, now);
        pages::menu_animate(&mut self.rt, now);
        pages::info_animate(&mut self.rt, now);
        dialog::animate(&mut self.rt, now);
        self.flush_custom();
        self.process_input(now, input);
        self.rt.apply_deferred(now);
        self.flush_custom();
        self.custom_tick(now);
        self.rt.apply_deferred(now);
        self.flush_custom();

        if self.rt.dirty && (now.wrapping_sub(self.rt.next_frame) as i32) >= 0 {
            let missed = now.wrapping_sub(self.rt.next_frame) / FRAME_INTERVAL_MS;
            self.rt.next_frame = self
                .rt
                .next_frame
                .wrapping_add((missed + 1).wrapping_mul(FRAME_INTERVAL_MS));
            self.rt.dirty = false;
            self.draw_scene();
            self.rt.frame_ready = true;
        }
        Status::Ok
    }

    /// 当前画布。
    pub fn frame(&self) -> &Frame<'_> {
        self.rt.frame()
    }

    /// 当前页面编号。
    pub fn current_page(&self) -> PageId {
        self.rt.current_page()
    }

    /// 取走「有新帧待提交」的标记。
    pub fn take_frame_ready(&mut self) -> bool {
        let ready = self.rt.frame_ready;
        self.rt.frame_ready = false;
        ready
    }

    /// 取走一个应用事件。
    pub fn poll_event(&mut self) -> Option<EventId> {
        self.rt.poll_event()
    }

    /// 取走最近一次错误。
    pub fn poll_error(&mut self) -> Option<ErrorInfo> {
        self.rt.poll_error()
    }

    /// 设置菜单条目状态。
    pub fn set_menu_item_state(
        &mut self,
        page_id: PageId,
        item_index: u16,
        state: ItemState,
    ) -> Status {
        self.rt.set_menu_item_state(page_id, item_index, state)
    }

    /// 读取整数编辑器草稿。
    pub fn get_int_editor_draft(&self, binding_index: u16) -> Option<i32> {
        self.rt.get_int_editor_draft(binding_index)
    }

    /// 显示工作是否已经静止。
    pub fn is_display_idle(&self) -> bool {
        self.rt.is_display_idle()
    }

    /// 当前页面的选中条目下标；菜单页等于条目数时表示虚拟返回项。
    pub fn selected_index(&self) -> u16 {
        self.rt.selected_index()
    }

    /// 是否有覆盖层（编辑器、确认框、消息框）正占用输入。
    pub fn dialog_open(&self) -> bool {
        self.rt.dialog_open()
    }

    /// 标记需要重画。
    pub fn invalidate(&mut self) {
        self.rt.invalidate();
    }

    /// 打开整数编辑器。
    pub fn open_int_editor(&mut self, binding_index: u16) -> Status {
        self.rt.open_int_editor(binding_index)
    }

    /// 打开开关编辑器。
    pub fn open_bool_editor(&mut self, binding_index: u16) -> Status {
        self.rt.open_bool_editor(binding_index)
    }

    /// 打开确认框。
    pub fn open_confirm(&mut self, confirm_index: u16) -> Status {
        self.rt.open_confirm(confirm_index)
    }

    /// 打开消息框。
    pub fn show_message(&mut self, text: &'static str, event: EventId) -> Status {
        self.rt.show_message(text, event)
    }

    /// 显示 Toast。
    pub fn show_toast(&mut self, text: &'static str, duration_ms: u32) -> Status {
        self.rt.show_toast(text, duration_ms)
    }
}
