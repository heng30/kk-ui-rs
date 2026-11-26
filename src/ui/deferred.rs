//! 弹窗/Toast 的打开入口与延迟请求。
//!
//! 公开入口在自绘页回调期间不能直接开弹窗，只能登记请求，等回调退出后统一执行。

use super::{Runtime, Status};
use crate::model::{
    BitsBinding, BoolBinding, ChoiceBinding, ConfirmDesc, EVENT_NONE, EventId, IntBinding,
    LineInputBinding, PageType,
};
use crate::widgets::dialog;
use crate::widgets::dialog::Row;

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
    /// 打开枚举选择弹窗。
    Choice,
    /// 打开位掩码弹窗。
    Bits,
    /// 打开文本编辑器。
    LineInput,
    /// 打开确认框。
    Confirm,
    /// 打开消息框。
    Message,
    /// 打开加载弹窗。
    Loading,
    /// 关闭加载弹窗。
    HideLoading,
    /// 显示 Toast。
    Toast,
}

/// 一次延迟请求。
#[derive(Clone, Copy)]
pub(crate) struct Deferred {
    /// 请求类型。
    pub kind: DeferredType,
    /// 编辑器或确认框操作的表项；其它请求为 `None`。
    pub row: Option<Row>,
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
        row: Option<Row>,
        text: &'static str,
        event: EventId,
        duration: u32,
    ) -> Status {
        if self.deferred.kind != DeferredType::None {
            return Status::Busy;
        }
        self.deferred = Deferred {
            kind,
            row,
            event,
            text,
            duration,
        };
        Status::Ok
    }

    /// 打开整数编辑器。
    pub fn open_int_editor(&mut self, binding: &'static IntBinding) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::Int,
                Some(Row::Int(binding)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_int(self, binding, now)
    }

    /// 打开开关编辑器。
    pub fn open_bool_editor(&mut self, binding: &'static BoolBinding) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::Bool,
                Some(Row::Bool(binding)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_bool(self, binding, now)
    }

    /// 打开枚举选择弹窗。
    pub fn open_choice_editor(&mut self, binding: &'static ChoiceBinding) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::Choice,
                Some(Row::Choice(binding)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_choice(self, binding, now)
    }

    /// 打开位掩码弹窗。
    pub fn open_bits_editor(&mut self, binding: &'static BitsBinding) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::Bits,
                Some(Row::Bits(binding)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_bits(self, binding, now)
    }

    /// 打开文本编辑器。
    pub fn open_line_input(&mut self, binding: &'static LineInputBinding) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::LineInput,
                Some(Row::LineInput(binding)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_line_input(self, binding, now)
    }

    /// 打开确认框。
    pub fn open_confirm(&mut self, desc: &'static ConfirmDesc) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(
                DeferredType::Confirm,
                Some(Row::Confirm(desc)),
                "",
                EVENT_NONE,
                0,
            );
        }
        let now = self.last_update;
        dialog::open_confirm(self, desc, now)
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
            return self.defer(DeferredType::Message, None, text, event, 0);
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
            return self.defer(DeferredType::Toast, None, text, EVENT_NONE, duration_ms);
        }
        let now = self.last_update;
        dialog::toast_open(self, text, duration_ms, now)
    }

    /// 打开加载弹窗；已经有覆盖层时返回 `Busy`。
    pub fn show_loading(&mut self, text: &'static str) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.in_callback {
            return self.defer(DeferredType::Loading, None, text, EVENT_NONE, 0);
        }
        let now = self.last_update;
        dialog::open_loading(self, text, now)
    }

    /// 关闭加载弹窗；没有加载弹窗时返回 `NotAllowed`。
    pub fn hide_loading(&mut self) -> Status {
        if !self.initialized {
            return Status::NotInitialized;
        }
        if !dialog::loading_open(self) {
            return Status::NotAllowed;
        }
        if self.in_callback {
            return self.defer(DeferredType::HideLoading, None, "", EVENT_NONE, 0);
        }
        let now = self.last_update;
        dialog::close(self, now);
        Status::Ok
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
            return self.defer(DeferredType::Close, None, "", EVENT_NONE, 0);
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
            return self.defer(DeferredType::Finish, None, "", event, 0);
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
                if let Some(Row::Int(binding)) = request.row {
                    dialog::open_int(self, binding, now);
                }
            }
            DeferredType::Bool => {
                if let Some(Row::Bool(binding)) = request.row {
                    dialog::open_bool(self, binding, now);
                }
            }
            DeferredType::Confirm => {
                if let Some(Row::Confirm(desc)) = request.row {
                    dialog::open_confirm(self, desc, now);
                }
            }
            DeferredType::Choice => {
                if let Some(Row::Choice(binding)) = request.row {
                    dialog::open_choice(self, binding, now);
                }
            }
            DeferredType::Bits => {
                if let Some(Row::Bits(binding)) = request.row {
                    dialog::open_bits(self, binding, now);
                }
            }
            DeferredType::LineInput => {
                if let Some(Row::LineInput(binding)) = request.row {
                    dialog::open_line_input(self, binding, now);
                }
            }
            DeferredType::Message => {
                dialog::open_message(self, request.text, request.event, now);
            }
            DeferredType::Loading => {
                dialog::open_loading(self, request.text, now);
            }
            DeferredType::HideLoading => {
                if dialog::loading_open(self) {
                    dialog::close(self, now);
                }
            }
            DeferredType::Toast => {
                dialog::toast_open(self, request.text, request.duration, now);
            }
        }
    }
}
