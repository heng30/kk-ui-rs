//! 弹窗/Toast 的打开入口与延迟请求。
//!
//! 公开入口在自绘页回调期间不能直接开弹窗，只能登记请求，等回调退出后统一执行。

use super::{Runtime, Status};
use crate::model::{EVENT_NONE, EventId, PageType};
use crate::widgets::dialog;

/// 延迟请求类型。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeferredType {
    /// 无请求。
    None,
    /// 自绘页请求关闭。
    Close,
    /// 自绘页结束并上报事件。
    Finish,
    /// 打开整数编辑器。
    Int,
    /// 打开开关编辑器。
    Bool,
    /// 打开确认框。
    Confirm,
    /// 打开消息框。
    Message,
    /// 显示 Toast。
    Toast,
}

/// 一次延迟请求。
#[derive(Clone, Copy)]
pub(crate) struct Deferred {
    /// 请求类型。
    pub kind: DeferredType,
    /// 绑定表或确认框表的下标。
    pub reference: u16,
    /// 消息框或结束时上报的事件。
    pub event: EventId,
    /// 消息框或 Toast 的文本。
    pub text: &'static str,
    /// Toast 时长（毫秒）。
    pub duration: u32,
}

impl<'a> Runtime<'a> {
    /// 登记一次延迟请求；已有请求未处理时返回 `Busy`。
    fn defer(
        &mut self,
        kind: DeferredType,
        reference: u16,
        text: &'static str,
        event: EventId,
        duration: u32,
    ) -> Status {
        if self.deferred.kind != DeferredType::None {
            return Status::Busy;
        }
        self.deferred = Deferred {
            kind,
            reference,
            event,
            text,
            duration,
        };
        Status::Ok
    }

    /// 打开整数编辑器。
    pub fn open_int_editor(&mut self, binding_index: u16) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if binding_index as usize >= self.app.int_bindings.len() {
            return Status::InvalidArgument;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Int, binding_index, "", EVENT_NONE, 0);
        }
        let now = self.last_update;
        dialog::open_int(self, binding_index, now)
    }

    /// 打开开关编辑器。
    pub fn open_bool_editor(&mut self, binding_index: u16) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if binding_index as usize >= self.app.bool_bindings.len() {
            return Status::InvalidArgument;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Bool, binding_index, "", EVENT_NONE, 0);
        }
        let now = self.last_update;
        dialog::open_bool(self, binding_index, now)
    }

    /// 打开确认框。
    pub fn open_confirm(&mut self, confirm_index: u16) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if confirm_index as usize >= self.app.confirm_descs.len() {
            return Status::InvalidArgument;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Confirm, confirm_index, "", EVENT_NONE, 0);
        }
        let now = self.last_update;
        dialog::open_confirm(self, confirm_index, now)
    }

    /// 打开消息框。
    pub fn show_message(&mut self, text: &'static str, event: EventId) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Message, 0, text, event, 0);
        }
        let now = self.last_update;
        dialog::open_message(self, text, event, now)
    }

    /// 显示 Toast；`duration_ms` 为 0 时使用默认时长。
    pub fn show_toast(&mut self, text: &'static str, duration_ms: u32) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Toast, 0, text, EVENT_NONE, duration_ms);
        }
        let now = self.last_update;
        dialog::toast_open(self, text, duration_ms, now)
    }

    /// 自绘页请求关闭自己。
    pub fn custom_request_close(&mut self) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if self.current_page_type() != PageType::Custom {
            return Status::NotAllowed;
        }
        if self.transition.active || dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.stack_count == 0 {
            return Status::NotAllowed;
        }
        if self.in_callback {
            return self.defer(DeferredType::Close, 0, "", EVENT_NONE, 0);
        }
        let now = self.last_update;
        self.navigate_back(now)
    }

    /// 自绘页结束时上报事件并返回上一页。
    pub fn custom_finish(&mut self, event: EventId) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if event == EVENT_NONE {
            return Status::InvalidArgument;
        }
        if self.current_page_type() != PageType::Custom {
            return Status::NotAllowed;
        }
        if self.transition.active || dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.stack_count == 0 {
            return Status::NotAllowed;
        }
        if self.in_callback {
            return self.defer(DeferredType::Finish, 0, "", event, 0);
        }
        if !self.queue_event(event) {
            return Status::QueueFull;
        }
        let now = self.last_update;
        self.navigate_back(now)
    }

    /// 处理延迟请求。
    pub(crate) fn apply_deferred(&mut self, now: u32) {
        let request = self.deferred;
        self.deferred.kind = DeferredType::None;
        match request.kind {
            DeferredType::None => {}
            DeferredType::Close => {
                if self.current_page_type() == PageType::Custom
                    && self.stack_count != 0
                    && !self.transition.active
                    && !dialog::dialog_active(self)
                {
                    self.navigate_back(now);
                }
            }
            DeferredType::Finish => {
                if self.current_page_type() != PageType::Custom
                    || self.stack_count == 0
                    || self.transition.active
                    || dialog::dialog_active(self)
                {
                    self.record_error(Status::NotAllowed, self.current.page, request.event);
                } else if self.queue_event(request.event) {
                    self.navigate_back(now);
                } else {
                    self.record_error(Status::QueueFull, self.current.page, request.event);
                    self.reject_flash = 2;
                    self.dirty = true;
                }
            }
            DeferredType::Int => {
                dialog::open_int(self, request.reference, now);
            }
            DeferredType::Bool => {
                dialog::open_bool(self, request.reference, now);
            }
            DeferredType::Confirm => {
                dialog::open_confirm(self, request.reference, now);
            }
            DeferredType::Message => {
                dialog::open_message(self, request.text, request.event, now);
            }
            DeferredType::Toast => {
                dialog::toast_open(self, request.text, request.duration, now);
            }
        }
    }
}
