//! 自绘页导航：Custom ↔ Custom 互跳、互跳后返回、跳到当前页、转场与弹窗期间的导航。

use kk_ui::{
    App, CustomPage, CustomPageHandler, Frame, Input, KEY_OK, KkUi, PageId, PageRoute, PageType,
    Runtime, Shared, Status, Texts,
};

const PAGE_A: PageId = 1;
const PAGE_B: PageId = 2;
const WIDTH: i16 = 64;
const HEIGHT: i16 = 32;

/// 事件序列，每 3 bit 一个新事件，低位最新：1/2 是 A 进/出，3/4 是 B 进/出。
static SEQ: Shared<u32> = Shared::new(0);

/// 两个用例共用 `SEQ`，并行跑会互相污染，用锁串起来并清零。
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn exclusive() -> std::sync::MutexGuard<'static, ()> {
    let guard = LOCK.lock().unwrap_or_else(|error| error.into_inner());
    SEQ.set(0);
    guard
}

fn push(kind: u32) {
    SEQ.set((SEQ.get() << 3) | kind);
}

/// 把事件编号按发生顺序拼成期望的序列值。
fn seq(events: &[u32]) -> u32 {
    events.iter().fold(0, |acc, &kind| (acc << 3) | kind)
}

struct PageA;
struct PageB;

impl CustomPageHandler for PageA {
    fn on_enter(&self, _ui: &mut Runtime<'_>, _page: PageId) {
        push(1);
    }

    fn on_leave(&self, _ui: &mut Runtime<'_>, _page: PageId) {
        push(2);
    }
}

impl CustomPageHandler for PageB {
    fn on_enter(&self, _ui: &mut Runtime<'_>, _page: PageId) {
        push(3);
    }

    fn on_leave(&self, _ui: &mut Runtime<'_>, _page: PageId) {
        push(4);
    }
}

static HANDLER_A: PageA = PageA;
static HANDLER_B: PageB = PageB;
static PAGES: [CustomPage; 2] = [&HANDLER_A, &HANDLER_B];
static ROUTES: [PageRoute; 2] = [
    PageRoute {
        kind: PageType::Custom,
        index: 0,
    },
    PageRoute {
        kind: PageType::Custom,
        index: 1,
    },
];
static APP: App = App {
    root_page: PAGE_A,
    routes: &ROUTES,
    home_pages: &[],
    menu_pages: &[],
    info_pages: &[],
    custom_pages: &PAGES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
    texts: Texts {
        return_text: "",
        cancel_text: "",
        confirm_text: "",
        on_text: "",
        off_text: "",
        message_title: "",
    },
};

/// 一块 `'static` 帧缓冲 + 单调时钟，方便连着一串操作。
struct Rig {
    ui: KkUi<'static>,
    now: u32,
}

impl Rig {
    fn new() -> Self {
        let buffer: &'static mut [u8] =
            Box::leak(vec![0u8; Frame::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        let mut ui = KkUi::new(&APP, buffer, WIDTH, HEIGHT);
        assert_eq!(ui.init(), Status::Ok);
        Self { ui, now: 0 }
    }

    fn step(&mut self, keys: u8) {
        self.now += 5;
        assert_eq!(
            self.ui.update(
                self.now,
                Input {
                    keys,
                    encoder_delta: 0,
                }
            ),
            Status::Ok
        );
    }

    /// 按下并松开一次按键，再空转到动画跑完。
    fn press(&mut self, keys: u8) {
        for _ in 0..8 {
            self.step(keys);
        }
        for _ in 0..70 {
            self.step(0);
        }
    }

    /// 导航并把转场推到结束；`now` 不变时第二次导航才是 `Busy` 的判据。
    fn navigate_to(&mut self, page: PageId) -> Status {
        let status = self.ui.navigate_to(page, self.now);
        for _ in 0..70 {
            self.step(0);
        }
        status
    }

    fn navigate_back(&mut self) -> Status {
        let status = self.ui.navigate_back(self.now);
        for _ in 0..70 {
            self.step(0);
        }
        status
    }
}

#[test]
fn custom_pages_navigate_both_ways() {
    let _guard = exclusive();
    let mut rig = Rig::new();
    assert_eq!(SEQ.get(), seq(&[1]), "root page enters on init");

    // 跳到当前页：不转场、不压栈、不触发回调。
    assert_eq!(rig.navigate_to(PAGE_A), Status::Ok);
    assert_eq!(rig.ui.current_page(), PAGE_A);
    assert_eq!(SEQ.get(), seq(&[1]), "same page must not fire callbacks");
    assert_eq!(rig.navigate_back(), Status::NotAllowed, "stack stays empty");

    // A → B：先离开旧页再进入新页。
    assert_eq!(rig.navigate_to(PAGE_B), Status::Ok);
    assert_eq!(rig.ui.current_page(), PAGE_B);
    assert_eq!(SEQ.get(), seq(&[1, 2, 3]), "leave A then enter B");

    // 返回 A：B 离开、A 重新进入。
    assert_eq!(rig.navigate_back(), Status::Ok);
    assert_eq!(rig.ui.current_page(), PAGE_A);
    assert_eq!(SEQ.get(), seq(&[1, 2, 3, 4, 1]), "leave B then enter A");
}

#[test]
fn navigation_is_rejected_during_transition_and_dialog() {
    let _guard = exclusive();
    let mut rig = Rig::new();

    // 转场进行中再导航：Busy，且回调一个都不发。
    assert_eq!(rig.ui.navigate_to(PAGE_B, rig.now), Status::Ok);
    assert_eq!(rig.ui.navigate_to(PAGE_A, rig.now), Status::Busy);
    assert_eq!(rig.ui.navigate_back(rig.now), Status::Busy);
    assert_eq!(SEQ.get(), seq(&[1]), "rejected navigation stays silent");
    for _ in 0..70 {
        rig.step(0);
    }
    assert_eq!(rig.ui.current_page(), PAGE_B);
    assert_eq!(SEQ.get(), seq(&[1, 2, 3]));

    // 弹窗打开期间同样不许导航，也不许把按键漏给自绘页。
    assert_eq!(rig.ui.show_message("hi", 0), Status::Ok);
    assert!(rig.ui.dialog_open());
    assert_eq!(rig.ui.navigate_to(PAGE_A, rig.now), Status::Busy);
    assert_eq!(rig.ui.navigate_back(rig.now), Status::Busy);
    assert_eq!(rig.ui.current_page(), PAGE_B);
    rig.press(KEY_OK);
    assert!(!rig.ui.dialog_open(), "OK closes the message box");
    assert_eq!(
        SEQ.get(),
        seq(&[1, 2, 3]),
        "dialog input must not reach the page"
    );

    // 弹窗关掉后互跳照常。
    assert_eq!(rig.navigate_to(PAGE_A), Status::Ok);
    assert_eq!(rig.ui.current_page(), PAGE_A);
    assert_eq!(SEQ.get(), seq(&[1, 2, 3, 4, 1]));
}
