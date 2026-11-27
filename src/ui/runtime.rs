//! 运行时本体：[`Runtime`] 的内存布局与页面/画布基础操作。
//!
//! 输入、导航、延迟请求等行为分散在 `input`、`nav`、`deferred` 等文件，各自给
//! [`Runtime`] 加 `impl` 块。

use super::input::KeyState;
use super::nav::{self, PageTransition, TransitionKind};
use super::{
    BitsAnimation, ChipAnimation, Deferred, DeferredType, EVENT_QUEUE_LENGTH, ErrorInfo,
    HomeAnimation, ListAnimation, MarqueeState, MenuRoll, PAGE_HASH_MAX, PageState, Status,
};
use crate::gfx::Frame;
use crate::model::{
    App, BitsBinding, EVENT_NONE, EventId, IntBinding, ItemState, LineInputBinding, PageRoute,
    PageType,
};
use crate::widgets::dialog::{Overlay, OverlayType, Row, Toast};
use crate::widgets::pages;

/// UI 运行时状态；自绘页回调也通过它访问画布与 UI 接口。
pub struct Runtime<'a> {
    /// 应用描述表。
    pub(crate) app: &'static App,
    /// 画布。
    pub(crate) frame: Frame<'a>,
    /// 当前页面状态。
    pub(crate) current: PageState,
    /// 导航栈，栈顶是上一页。
    pub(super) stack: [PageState; nav::NAV_DEPTH],
    /// 页面切换动画。
    pub(super) transition: PageTransition,
    /// 之后导航使用的转场效果。
    pub(super) transition_kind: TransitionKind,
    /// 首页横向动画。
    pub(crate) home_anim: HomeAnimation,
    /// 列表滚动与焦点动画。
    pub(crate) list_anim: ListAnimation,
    /// 分段选择行的滑动框。
    pub(crate) chip: ChipAnimation,
    /// 位掩码行的游标与翻转反馈。
    pub(crate) bits: BitsAnimation,
    /// 跑马灯。
    pub(crate) marquee: MarqueeState,
    /// 菜单右列取值的滚动数字。
    pub(crate) menu_roll: MenuRoll,
    /// 上一次刷新到这次刷新的间隔（毫秒），弹簧按它积分。
    pub(crate) dt_ms: i32,
    /// 覆盖层状态。
    pub(crate) overlay: Overlay,
    /// Toast 状态。
    pub(crate) toast: Toast,
    /// 待处理的延迟请求。
    pub(super) deferred: Deferred,
    /// 三个按键的消抖状态，下标对应按键位。
    pub(super) keys: [KeyState; 3],
    /// 事件环形队列。
    pub(super) events: [EventId; EVENT_QUEUE_LENGTH],
    /// 队列头部下标。
    pub(super) event_head: u8,
    /// 队列内事件数。
    pub(super) event_count: u8,
    /// 最近一次错误。
    pub(super) error: ErrorInfo,
    /// 最近一次 [`KkUi::update`] 的时戳。
    pub(crate) last_update: u32,
    /// 最近一次有按键或编码器动作的时戳。
    pub(crate) last_input: u32,
    /// 变暗与息屏的两个阀值（毫秒），为 0 表示对应阶段不启用。
    pub(crate) dim_after_ms: u32,
    pub(crate) sleep_after_ms: u32,
    /// 下一帧最早可出帧的时刻。
    pub(super) next_frame: u32,
    /// 导航栈内页面数。
    pub(super) stack_count: u8,
    /// 是否已成功初始化。
    pub(crate) initialized: bool,
    /// 按键状态是否已用首帧输入对齐。
    pub(super) keys_initialized: bool,
    /// 是否有待重画的内容。
    pub(crate) dirty: bool,
    /// 是否有一帧新画面待上层提交。
    pub(crate) frame_ready: bool,
    /// [`Runtime::poll_error`] 是否尚未被取走。
    pub(super) error_valid: bool,
    /// 拒绝操作的边框闪烁计数。
    pub(crate) reject_flash: u8,
    /// 页面切换期间需要吞掉的按键位。
    pub(super) blocked_keys: u8,
    /// 是否正在执行自绘页回调。
    pub(crate) in_callback: bool,
    /// 待通知的自绘页进入/离开事件，由 [`KkUi`] 在安全时机送进回调。
    pub(super) enter_custom: Option<&'static PageRoute>,
    pub(super) leave_custom: Option<&'static PageRoute>,
    /// 上一帧每页的哈希，用于只提交变化的页。
    pub(super) page_hashes: [u32; PAGE_HASH_MAX],
    /// 上一次哈希是否有效；初始化后第一次计算时置为假，强制全刷。
    pub(super) page_hashes_valid: bool,
    /// 最近一次计算出的变化页位掩码，`take_dirty_pages` 取走后清零。
    pub(super) dirty_pages: u32,
}

impl<'a> Runtime<'a> {
    /// 绑定一份应用描述表与帧缓冲。
    pub(crate) fn new(app: &'static App, buffer: &'a mut [u8], width: i16, height: i16) -> Self {
        Self {
            app,
            frame: Frame::new(buffer, width, height),
            current: PageState::default(),
            stack: [PageState::default(); nav::NAV_DEPTH],
            transition: PageTransition::default(),
            transition_kind: TransitionKind::Slide,
            home_anim: HomeAnimation::default(),
            list_anim: ListAnimation::default(),
            chip: ChipAnimation::EMPTY,
            bits: BitsAnimation::EMPTY,
            marquee: MarqueeState::EMPTY,
            menu_roll: MenuRoll::EMPTY,
            dt_ms: 0,
            overlay: Overlay::EMPTY,
            toast: Toast {
                text: "",
                shown_at: 0,
                expires_at: 0,
                phase_q12: 0,
                visible: false,
                closing: false,
            },
            deferred: Deferred {
                kind: DeferredType::None,
                row: None,
                event: EVENT_NONE,
                text: "",
                duration: 0,
            },
            keys: [KeyState::default(); 3],
            events: [EVENT_NONE; EVENT_QUEUE_LENGTH],
            event_head: 0,
            event_count: 0,
            error: ErrorInfo {
                code: Status::Ok,
                page: None,
                index: 0,
            },
            last_update: 0,
            last_input: 0,
            dim_after_ms: 0,
            sleep_after_ms: 0,
            next_frame: 0,
            stack_count: 0,
            initialized: false,
            keys_initialized: false,
            dirty: false,
            frame_ready: false,
            in_callback: false,
            error_valid: false,
            reject_flash: 0,
            blocked_keys: 0,
            enter_custom: None,
            leave_custom: None,
            page_hashes: [0; PAGE_HASH_MAX],
            page_hashes_valid: false,
            dirty_pages: 0,
        }
    }

    /// 当前画布。
    pub fn frame(&self) -> &Frame<'_> {
        &self.frame
    }

    /// 重算每页哈希，把变化的页并进 `dirty_pages`。
    ///
    /// 页数超过 [`PAGE_HASH_MAX`] 时放弃跟踪，调用方拿到全 1 掩码自己整帧提交。
    pub(super) fn refresh_page_hashes(&mut self) {
        let pages = self.frame.pages();
        if pages > PAGE_HASH_MAX {
            self.page_hashes_valid = false;
            return;
        }
        let mut mask = 0u32;
        for page in 0..pages {
            let hash = self.frame.page_hash(page);
            if !self.page_hashes_valid || self.page_hashes[page] != hash {
                mask |= 1 << page;
            }
            self.page_hashes[page] = hash;
        }
        self.page_hashes_valid = true;
        self.dirty_pages |= mask;
    }

    /// 取走变化页掩码，`bit n` 对应第 `n` 页（每页 8 行）。
    pub(super) fn take_dirty_pages(&mut self) -> u32 {
        if self.frame.pages() > PAGE_HASH_MAX {
            return u32::MAX;
        }
        core::mem::take(&mut self.dirty_pages)
    }

    /// 当前画布（可变），供自绘页绘制。
    pub fn frame_mut(&mut self) -> &mut Frame<'a> {
        &mut self.frame
    }

    /// 当前页面句柄；尚未初始化时为 `None`。
    pub fn current_page(&self) -> Option<&'static PageRoute> {
        self.current.page
    }

    /// 当前页面类型。
    pub(crate) fn current_page_type(&self) -> PageType {
        self.current
            .page
            .map(PageRoute::kind)
            .unwrap_or(PageType::Home)
    }

    /// 回到根页面并清空导航栈、覆盖层、Toast 与动画，等于「重新开始」。
    pub(crate) fn init(&mut self) -> Status {
        if let Err((page, index)) = Self::validate_app(self.app) {
            return self.config_error(page, index);
        }
        let state = self.reset_page_state(self.app.root_page);
        self.current = state;
        self.stack_count = 0;
        self.transition.active = false;
        self.home_anim.active = false;
        self.list_anim.active = false;
        self.chip = ChipAnimation::EMPTY;
        self.bits = BitsAnimation::EMPTY;
        self.marquee = MarqueeState::EMPTY;
        self.menu_roll = MenuRoll::EMPTY;
        self.overlay = Overlay::EMPTY;
        self.toast.visible = false;
        self.toast.closing = false;
        self.blocked_keys = 0;
        self.reject_flash = 0;
        self.page_hashes_valid = false;
        self.dirty_pages = 0;
        self.initialized = true;
        self.dirty = true;
        if self.current_page_type() == PageType::Custom {
            self.enter_custom = self.current.page;
        }
        Status::Ok
    }

    /// 新建一个页面状态，菜单页会先把焦点收缩到合法条目。
    pub(crate) fn reset_page_state(&self, page: &'static PageRoute) -> PageState {
        let mut state = PageState {
            page: Some(page),
            ..PageState::default()
        };
        if page.kind() == PageType::Menu {
            pages::menu_repair_focus(self, &mut state);
        }
        state
    }

    /// 标记需要重画。
    pub fn invalidate(&mut self) {
        if self.initialized {
            self.dirty = true;
        }
    }

    /// 显示工作是否已经静止：可以安全地做一次阻塞式提交。
    pub fn is_display_idle(&self) -> bool {
        self.initialized && !self.dirty && !self.frame_ready
    }

    /// 当前页面的选中条目下标；菜单页等于条目数时表示虚拟返回项。
    pub fn selected_index(&self) -> u16 {
        self.current.selected
    }

    /// 是否有覆盖层（编辑器、确认框、消息框、加载弹窗）正占用输入。
    pub fn dialog_open(&self) -> bool {
        self.overlay.kind != OverlayType::None
    }

    /// 读取整数编辑器草稿；编辑器未打开时返回 `None`。
    pub fn get_int_editor_draft(&self, binding: &'static IntBinding) -> Option<i32> {
        if !self.initialized || self.overlay.kind != OverlayType::Int || self.overlay.closing {
            return None;
        }
        match self.overlay.row {
            Some(Row::Int(current)) if core::ptr::eq(current, binding) => Some(self.overlay.draft),
            _ => None,
        }
    }

    /// 读取位掩码弹窗草稿；弹窗未打开时返回 `None`。
    pub fn get_bits_editor_draft(&self, binding: &'static BitsBinding) -> Option<u32> {
        if !self.initialized || self.overlay.kind != OverlayType::Bits || self.overlay.closing {
            return None;
        }
        match self.overlay.row {
            Some(Row::Bits(current)) if core::ptr::eq(current, binding) => {
                Some(self.overlay.draft as u32)
            }
            _ => None,
        }
    }

    /// 文本编辑器是否正打开在给定绑定上。
    pub fn line_input_open(&self, binding: &'static LineInputBinding) -> bool {
        if !self.initialized || self.overlay.kind != OverlayType::LineInput || self.overlay.closing
        {
            return false;
        }
        matches!(self.overlay.row, Some(Row::LineInput(current)) if core::ptr::eq(current, binding))
    }

    /// 设置菜单条目状态。
    pub fn set_menu_item_state(
        &mut self,
        page_route: &'static PageRoute,
        item_index: u16,
        state: ItemState,
    ) -> Status {
        pages::menu_set_state(self, page_route, item_index, state)
    }
}
