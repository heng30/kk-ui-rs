//! 导航栈与页面切换动画。

use super::{PAGE_MS, PageState, Runtime, Status};
use crate::model::{PageId, PageType};
use crate::widgets::{dialog, pages};

/// 导航栈深度上限。
pub(super) const NAV_DEPTH: usize = 8;

/// 页面切换动画。
#[derive(Clone, Copy, Default)]
pub(super) struct PageTransition {
    /// 正在退出页面的状态快照。
    pub(super) outgoing: PageState,
    /// 动画开始的时戳。
    pub(super) started: u32,
    /// 是否正在播放。
    pub(super) active: bool,
    /// 是返回还是进入，决定两页的滑动方向。
    pub(super) reverse: bool,
}

impl<'a> Runtime<'a> {
    /// 启动页面切换动画，并吞掉切换期间仍按住的按键。
    fn start_transition(&mut self, outgoing: PageState, reverse: bool, now: u32) {
        let mut stable = 0u8;
        for (index, key) in self.keys.iter().enumerate() {
            if key.stable {
                stable |= 1 << index;
            }
        }
        self.blocked_keys |= stable;
        let leave_custom =
            self.route(outgoing.page).map(|route| route.kind) == Some(PageType::Custom);
        self.transition = PageTransition {
            outgoing,
            started: now,
            active: true,
            reverse,
        };
        if leave_custom {
            self.leave_custom = Some(outgoing.page);
        }
        self.dirty = true;
    }

    /// 进入页面。
    pub fn navigate_to(&mut self, page: PageId, now: u32) -> Status {
        let Some(route) = self.route(page) else {
            return Status::InvalidArgument;
        };
        if self.transition.active || dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.current.page == page {
            return Status::Ok;
        }
        if self.stack_count as usize >= NAV_DEPTH {
            self.record_error(Status::NavigationFull, self.current.page, page);
            self.reject_flash = 2;
            self.dirty = true;
            return Status::NavigationFull;
        }
        let outgoing = self.current;
        self.stack[self.stack_count as usize] = outgoing;
        self.stack_count += 1;
        let state = self.reset_page_state(page);
        self.current = state;
        self.home_anim.active = false;
        self.list_anim.active = false;
        self.start_transition(outgoing, false, now);
        if route.kind == PageType::Custom {
            self.enter_custom = Some(page);
        }
        Status::Ok
    }

    /// 返回上一页。
    pub fn navigate_back(&mut self, now: u32) -> Status {
        if self.transition.active || dialog::dialog_active(self) {
            return Status::Busy;
        }
        if self.stack_count == 0 {
            return Status::NotAllowed;
        }
        let outgoing = self.current;
        self.stack_count -= 1;
        self.current = self.stack[self.stack_count as usize];
        if self.current_page_type() == PageType::Menu {
            let mut current = self.current;
            pages::menu_repair_focus(self, &mut current);
            self.current = current;
        }
        self.home_anim.active = false;
        self.list_anim.active = false;
        self.start_transition(outgoing, true, now);
        if self.current_page_type() == PageType::Custom {
            self.enter_custom = Some(self.current.page);
        }
        Status::Ok
    }

    /// 页面切换动画推进。
    pub(crate) fn animate_navigation(&mut self, now: u32) {
        if !self.transition.active {
            return;
        }
        self.dirty = true;
        if now.wrapping_sub(self.transition.started) >= PAGE_MS {
            self.transition.active = false;
        }
    }
}
