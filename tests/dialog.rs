//! 弹窗交互：结束编辑后焦点要真的落到确认键，确认框能用方向键切换焦点。
//!
//! 这两条曾经是坏的：`set_focus` 只补间了焦点框，没记住新的焦点模式，
//! 于是编辑器按「确定」永远回到编辑态、确认框永远停在取消上。

use kk_ui::{
    App, BoolBinding, ConfirmDesc, Frame, Glyphs, HomePage, Input, IntBinding, KEY_DOWN, KEY_OK,
    KkUi, MenuAction, MenuItem, MenuPage, PageRoute, Shared, Texts, set_font,
};

kk_ui::event_ids! {
    EVENT_INT,
    EVENT_BOOL,
    EVENT_CONFIRMED,
    EVENT_CANCELLED,
}

#[test]
fn event_ids_are_sequential() {
    assert_eq!(
        (EVENT_INT, EVENT_BOOL, EVENT_CONFIRMED, EVENT_CANCELLED),
        (1, 2, 3, 4)
    );
}
/// 画布尺寸。
const WIDTH: i16 = 128;
const HEIGHT: i16 = 64;

/// 空字库：每个字符都缺字，文字绘制为空但仍按全角步进占位。
static GLYPHS: Glyphs = Glyphs {
    bitmaps: &[],
    widths: &[],
    codepoints: &[],
};
static ICON: [u8; 8] = [0; 8];
static INT_VALUE: Shared<i32> = Shared::new(10);
static BOOL_VALUE: Shared<bool> = Shared::new(false);

static HOME_ITEMS: [kk_ui::HomeItem; 1] = [kk_ui::HomeItem {
    label: "Menu",
    icon: kk_ui::HomeIcon::Static(&ICON),
    target: &ROUTE_MENU,
}];

static MENU_ITEMS: [MenuItem; 3] = [
    MenuItem {
        label: "Level",
        action: MenuAction::Int(&INT_LEVEL),
    },
    MenuItem {
        label: "Switch",
        action: MenuAction::Bool(&BOOL_SWITCH),
    },
    MenuItem {
        label: "Reset",
        action: MenuAction::Confirm(&CONFIRM_RESET),
    },
];

/// 测试用页面：首页 → 菜单。
static ROUTE_HOME: PageRoute = PageRoute::Home(&HomePage { items: &HOME_ITEMS });
static ROUTE_MENU: PageRoute = PageRoute::Menu(&MenuPage {
    title: "Menu",
    items: &MENU_ITEMS,
    states: None,
});
static ROUTES: [&PageRoute; 2] = [&ROUTE_HOME, &ROUTE_MENU];
static INT_LEVEL: IntBinding = IntBinding {
    title: "Level",
    value: &INT_VALUE,
    minimum: 0,
    maximum: 100,
    step: 1,
    unit: "",
    changed_event: EVENT_INT,
};
static BOOL_SWITCH: BoolBinding = BoolBinding {
    title: "Switch",
    value: &BOOL_VALUE,
    changed_event: EVENT_BOOL,
};
static CONFIRM_RESET: ConfirmDesc = ConfirmDesc {
    text: "Reset?",
    confirmed_event: EVENT_CONFIRMED,
    cancelled_event: EVENT_CANCELLED,
};
static INT_BINDINGS: [&IntBinding; 1] = [&INT_LEVEL];
static BOOL_BINDINGS: [&BoolBinding; 1] = [&BOOL_SWITCH];
static CONFIRMS: [&ConfirmDesc; 1] = [&CONFIRM_RESET];

static APP: App = App {
    root_page: &ROUTE_HOME,
    routes: &ROUTES,
    int_bindings: &INT_BINDINGS,
    bool_bindings: &BOOL_BINDINGS,
    confirm_descs: &CONFIRMS,
    line_inputs: &[],
    texts: Texts {
        return_text: "Back",
        cancel_text: "Cancel",
        confirm_text: "OK",
        on_text: "On",
        off_text: "Off",
        message_title: "Message",
    },
};

/// 一台只有按键的模拟面板。
struct Panel {
    ui: KkUi<'static>,
    now: u32,
}

impl Panel {
    fn new() -> Self {
        // 测试不开 cjk-fonts，默认无字体；装上空字库让布局走真实代码路径。
        set_font(&GLYPHS);
        let buffer: &'static mut [u8] =
            Box::leak(vec![0u8; Frame::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        let mut ui = KkUi::new(&APP, buffer, WIDTH, HEIGHT);
        assert_eq!(ui.init(), kk_ui::Status::Ok);
        INT_VALUE.set(10);
        BOOL_VALUE.set(false);
        let mut panel = Self { ui, now: 0 };
        panel.step(0);
        panel
    }

    fn step(&mut self, keys: u8) {
        self.now += 5;
        self.ui.update(
            self.now,
            Input {
                keys,
                encoder_delta: 0,
            },
        );
    }

    /// 按下并松开一次按键（按住 40 ms，超过 20 ms 消抖），然后空转到动画跑完。
    fn press(&mut self, keys: u8) {
        for _ in 0..8 {
            self.step(keys);
        }
        for _ in 0..70 {
            self.step(0);
        }
    }
}

#[test]
fn int_editor_commits_after_ending_edit() {
    let mut panel = Panel::new();
    panel.press(KEY_OK); // 首页 → 菜单
    panel.press(KEY_OK); // 打开整数编辑器
    assert_eq!(panel.ui.get_int_editor_draft(&INT_LEVEL), Some(10));
    panel.press(KEY_DOWN); // 值减一
    assert_eq!(panel.ui.get_int_editor_draft(&INT_LEVEL), Some(9));
    panel.press(KEY_OK); // 结束编辑，焦点应落到确认键
    panel.press(KEY_OK); // 提交并关闭
    assert_eq!(
        panel.ui.get_int_editor_draft(&INT_LEVEL),
        None,
        "editor must close"
    );
    assert_eq!(INT_VALUE.get(), 9, "committed value");
    assert_eq!(panel.ui.poll_event(), Some(EVENT_INT));
}

#[test]
fn confirm_dialog_follows_focus() {
    let mut panel = Panel::new();
    panel.press(KEY_OK); // 首页 → 菜单
    panel.press(KEY_DOWN); // 选中开关
    panel.press(KEY_DOWN); // 选中确认框
    panel.press(KEY_OK); // 打开确认框，初始焦点在取消
    panel.press(KEY_DOWN); // 焦点切到确认
    panel.press(KEY_OK);
    assert_eq!(panel.ui.poll_event(), Some(EVENT_CONFIRMED));
}

#[test]
fn confirm_dialog_cancels_from_default_focus() {
    let mut panel = Panel::new();
    panel.press(KEY_OK);
    panel.press(KEY_DOWN);
    panel.press(KEY_DOWN);
    panel.press(KEY_OK);
    panel.press(KEY_OK);
    assert_eq!(panel.ui.poll_event(), Some(EVENT_CANCELLED));
}

#[test]
fn bool_editor_commits_and_toggles() {
    let mut panel = Panel::new();
    panel.press(KEY_OK);
    panel.press(KEY_DOWN); // 选中开关
    panel.press(KEY_OK); // 打开开关编辑器
    panel.press(KEY_DOWN); // 翻转
    panel.press(KEY_OK); // 结束编辑
    panel.press(KEY_OK); // 提交
    assert!(BOOL_VALUE.get());
    assert_eq!(panel.ui.poll_event(), Some(EVENT_BOOL));
}

#[test]
fn loading_dialog_ignores_keys_and_closes_by_api() {
    let mut panel = Panel::new();
    assert_eq!(panel.ui.hide_loading(), kk_ui::Status::NotAllowed);
    assert_eq!(panel.ui.show_loading("Loading"), kk_ui::Status::Ok);
    panel.step(0);
    panel.press(KEY_OK);
    panel.press(KEY_DOWN);
    assert!(panel.ui.dialog_open(), "loading must swallow keys");
    let before = panel.ui.frame().buffer().to_vec();
    for _ in 0..30 {
        panel.step(0);
    }
    assert_ne!(
        before,
        panel.ui.frame().buffer(),
        "spinner must advance on its own"
    );
    assert_eq!(panel.ui.hide_loading(), kk_ui::Status::Ok);
    for _ in 0..70 {
        panel.step(0);
    }
    assert!(!panel.ui.dialog_open(), "loading must finish closing");
}

#[test]
fn loading_dialog_rejects_a_second_overlay() {
    let mut panel = Panel::new();
    assert_eq!(panel.ui.show_loading("Loading"), kk_ui::Status::Ok);
    assert_eq!(panel.ui.show_loading("Again"), kk_ui::Status::Busy);
    assert_eq!(panel.ui.show_message("Hi", EVENT_INT), kk_ui::Status::Busy);
}

#[test]
fn int_editor_keeps_redrawing_until_the_digits_settle() {
    let mut panel = Panel::new();
    panel.press(KEY_OK); // 首页 → 菜单
    panel.press(KEY_OK); // 打开整数编辑器
    for _ in 0..8 {
        panel.step(KEY_DOWN); // 按住，值减一
    }
    panel.ui.take_frame_ready();
    // 数值补间只有 100 ms，逐位滚要 300 ms：中间不能断帧，否则数字卡在半路。
    for _ in 0..50 {
        panel.step(0); // 之后 250 ms 不再输入
    }
    assert!(
        panel.ui.take_frame_ready(),
        "frames must keep coming while the digits roll"
    );
}
