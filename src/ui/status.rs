//! 操作结果与错误记录。

use super::Runtime;
use crate::model::PageRoute;

/// 操作结果。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// 成功。
    Ok,
    /// 参数非法。
    InvalidArgument,
    /// 尚未初始化。
    NotInitialized,
    /// 应用描述表配置错误。
    ConfigError,
    /// 当前状态不允许该操作。
    Busy,
    /// 事件队列已满。
    QueueFull,
    /// 导航栈已满。
    NavigationFull,
    /// 当前目标不支持该能力。
    Unsupported,
    /// 当前页面不允许该操作。
    NotAllowed,
}

impl Status {
    /// 稳定短名，便于日志输出。
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::InvalidArgument => "invalid argument",
            Status::NotInitialized => "not initialized",
            Status::ConfigError => "config error",
            Status::Busy => "busy",
            Status::QueueFull => "queue full",
            Status::NavigationFull => "navigation full",
            Status::Unsupported => "unsupported",
            Status::NotAllowed => "not allowed",
        }
    }
}

/// 被记录的错误信息。
#[derive(Clone, Copy)]
pub struct ErrorInfo {
    /// 错误码。
    pub code: Status,
    /// 出错时所在页面；尚未初始化时为 `None`。
    pub page: Option<&'static PageRoute>,
    /// 相关条目下标。
    pub index: u16,
}

impl<'a> Runtime<'a> {
    /// 记录一条错误，供应用轮询。
    pub(crate) fn record_error(
        &mut self,
        code: Status,
        page: Option<&'static PageRoute>,
        index: u16,
    ) {
        self.error = ErrorInfo { code, page, index };
        self.error_valid = true;
    }

    /// 记录一条配置错误并返回 `ConfigError`。
    pub(super) fn config_error(&mut self, page: Option<&'static PageRoute>, index: u16) -> Status {
        self.record_error(Status::ConfigError, page, index);
        Status::ConfigError
    }
}
