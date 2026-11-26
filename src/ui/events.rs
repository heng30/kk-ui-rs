//! 应用事件环形队列。

use super::{ErrorInfo, Runtime};
use crate::model::{EVENT_NONE, EventId};

/// 事件队列长度。
pub(crate) const EVENT_QUEUE_LENGTH: usize = 4;

impl Runtime<'_> {
    /// 事件入队；返回 `false` 表示队列已满。
    pub fn queue_event(&mut self, event: EventId) -> bool {
        if event == EVENT_NONE {
            return true;
        }
        if self.event_count as usize >= EVENT_QUEUE_LENGTH {
            return false;
        }
        let tail = (self.event_head as usize + self.event_count as usize) % EVENT_QUEUE_LENGTH;
        self.events[tail] = event;
        self.event_count += 1;
        true
    }

    /// 取出一个事件。
    pub fn poll_event(&mut self) -> Option<EventId> {
        if self.event_count == 0 {
            return None;
        }
        let event = self.events[self.event_head as usize];
        self.event_head = (self.event_head + 1) % EVENT_QUEUE_LENGTH as u8;
        self.event_count -= 1;
        Some(event)
    }

    /// 取出最近一次错误。
    pub fn poll_error(&mut self) -> Option<ErrorInfo> {
        if !self.error_valid {
            return None;
        }
        self.error_valid = false;
        Some(self.error)
    }
}
