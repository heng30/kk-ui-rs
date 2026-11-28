//! 单行输入框：行内移动、两端换行按钮、大写锁定、退格与取消。

use kk_ui::{
    App, Frame, Glyphs, HomeItem, HomePage, Input, KEY_DOWN, KEY_OK, KEY_UP, KkUi, LineInput,
    LineInputBinding, MenuAction, MenuItem, MenuPage, PageRoute, Texts, set_font,
};

kk_ui::event_ids! {
    EVENT_NAME,
}

/// 画布尺寸。
const WIDTH: i16 = 160;
const HEIGHT: i16 = 120;

/// 空字库：每个字符都缺字，文字绘制为空但仍按全角步进占位。
static GLYPHS: Glyphs = Glyphs {
    bitmaps: &[],
    widths: &[],
    codepoints: &[],
};
static ICON: [u8; 8] = [0; 8];

static NAME: LineInput = LineInput::literal("");
static LINE_INPUT: LineInputBinding = LineInputBinding {
    title: "Name",
    value: &NAME,
    changed_event: EVENT_NAME,
};
static LINE_INPUTS: [&LineInputBinding; 1] = [&LINE_INPUT];

static MENU_ITEMS: [MenuItem; 1] = [MenuItem {
    label: "Name",
    action: MenuAction::LineInput(&LINE_INPUT),
}];

static ROUTE_MENU: PageRoute = PageRoute::Menu(&MenuPage {
    title: "Menu",
    items: &MENU_ITEMS,
    states: None,
});

static HOME_ITEMS: [HomeItem; 1] = [HomeItem {
    label: "Menu",
    icon: kk_ui::HomeIcon::Static(&ICON),
    target: &ROUTE_MENU,
}];

static ROUTE_HOME: PageRoute = PageRoute::Home(&HomePage { items: &HOME_ITEMS });
static ROUTES: [&PageRoute; 2] = [&ROUTE_HOME, &ROUTE_MENU];

static APP: App = App {
    root_page: &ROUTE_HOME,
    routes: &ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
    line_inputs: &LINE_INPUTS,
    texts: Texts {
        return_text: "Back",
        cancel_text: "X",
        confirm_text: "OK",
        on_text: "On",
        off_text: "Off",
        message_title: "Message",
    },
};

/// 字母层每行最右边换行按钮的格子号，光标从 `(0, 1)` 的 `q` 起步。
const ROW0_MAX: u8 = 11; // 12 格
const ROW2_MAX: u8 = 10; // 11 格
const ROW3_MAX: u8 = 5; // 6 格
/// 功能行里 `esc` 与 `ok` 的格子。
const ESC_COL: u8 = 3;
const OK_COL: u8 = 4;

/// 一台只有按键的模拟面板。
struct Panel {
    ui: KkUi<'static>,
    now: u32,
}

/// 多个用例共享 `NAME`，并行跑会互相污染，用锁串起来并清零。
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn exclusive() -> std::sync::MutexGuard<'static, ()> {
    let guard = LOCK.lock().unwrap_or_else(|error| error.into_inner());
    NAME.set("");
    guard
}

impl Panel {
    fn new() -> Self {
        set_font(&GLYPHS);
        NAME.set("");
        let buffer: &'static mut [u8] =
            Box::leak(vec![0u8; Frame::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        let mut ui = KkUi::new(&APP, buffer, WIDTH, HEIGHT);
        assert_eq!(ui.init(), kk_ui::Status::Ok);
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

    /// 按下并松开一次按键，然后空转到页面转场跑完。
    fn press(&mut self, keys: u8) {
        for _ in 0..8 {
            self.step(keys);
        }
        for _ in 0..70 {
            self.step(0);
        }
    }

    /// 键盘内部的短按：不用等转场，按住 40 ms 再松开 60 ms 就够。
    fn tap(&mut self, keys: u8) {
        for _ in 0..8 {
            self.step(keys);
        }
        for _ in 0..12 {
            self.step(0);
        }
    }

    /// 在本行里下移 `count` 格。
    fn down(&mut self, count: u8) {
        for _ in 0..count {
            self.tap(KEY_DOWN);
        }
    }

    /// 在本行里上移 `count` 格。
    fn up(&mut self, count: u8) {
        for _ in 0..count {
            self.tap(KEY_UP);
        }
    }

    /// 打开菜单里的文本编辑器，光标落在 `(0, 1)` 的 `q`。
    fn open(&mut self) {
        self.press(KEY_OK);
        self.press(KEY_OK);
        assert!(self.ui.line_input_open(&LINE_INPUT));
    }
}

#[test]
fn typing_then_confirming_writes_the_value() {
    let _guard = exclusive();
    let mut panel = Panel::new();
    panel.open();
    panel.tap(KEY_OK); // q，光标留在原地
    panel.tap(KEY_OK); // 再打一个 q
    // 一路踩右端换行按钮下到功能行，再退回一格踩确定。
    panel.down(ROW0_MAX - 1);
    panel.tap(KEY_OK); // 第 0 行 → 第 1 行
    panel.tap(KEY_OK); // 第 1 行 → 第 2 行
    panel.tap(KEY_OK); // 第 2 行 → 第 3 行，落在右端换行按钮
    panel.up(ROW3_MAX - OK_COL);
    panel.tap(KEY_OK);
    assert!(!panel.ui.line_input_open(&LINE_INPUT));
    assert!(NAME.with(|text| text == "qq"));
    assert_eq!(panel.ui.poll_event(), Some(EVENT_NAME));
}

#[test]
fn caps_lock_keeps_uppercase_until_toggled_off() {
    let _guard = exclusive();
    let mut panel = Panel::new();
    panel.open();
    panel.down(ROW0_MAX - 1); // 第 0 行右端换行按钮
    panel.tap(KEY_OK);
    panel.tap(KEY_OK); // 下到第 2 行
    panel.up(ROW2_MAX - 1); // 退回第 1 格，也就是大写锁定键
    panel.tap(KEY_OK); // 开大写
    // 回到第 0 行的第一个字符键。
    panel.up(1);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.down(1);
    panel.tap(KEY_OK); // Q
    panel.tap(KEY_OK); // 还是 Q，锁定不自动关
    panel.down(ROW0_MAX - 1);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.up(ROW3_MAX - OK_COL);
    panel.tap(KEY_OK);
    assert!(NAME.with(|text| text == "QQ"));
}

#[test]
fn backspace_drops_the_last_character() {
    let _guard = exclusive();
    let mut panel = Panel::new();
    panel.open();
    panel.tap(KEY_OK); // q，光标留在原地
    // 退格只在最后一行：先踩换行按钮下到第 2 行，再退回左边一格。
    panel.down(ROW0_MAX - 1);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK); // 到第 2 行右端的换行按钮
    panel.up(1); // 左边一格是退格
    panel.tap(KEY_OK);
    panel.down(1); // 踩回换行按钮
    panel.tap(KEY_OK); // 下到功能行
    panel.up(ROW3_MAX - OK_COL);
    panel.tap(KEY_OK);
    assert!(NAME.with(|text| text.is_empty()));
}

#[test]
fn cancel_discards_the_buffer() {
    let _guard = exclusive();
    let mut panel = Panel::new();
    panel.open();
    NAME.set("keep");
    panel.tap(KEY_OK); // q，光标留在原地
    panel.down(ROW0_MAX - 1); // 第 0 行右端换行按钮
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK); // 下到功能行，落在右端换行按钮
    panel.up(ROW3_MAX - ESC_COL);
    panel.tap(KEY_OK);
    assert!(!panel.ui.line_input_open(&LINE_INPUT));
    assert!(NAME.with(|text| text == "keep"));
    assert_eq!(panel.ui.poll_event(), None);
}

#[test]
fn a_long_press_types_instead_of_returning() {
    let _guard = exclusive();
    let mut panel = Panel::new();
    panel.open();
    // 按住确定 500 ms：超过 400 ms 的返回阈值，但键盘上只是按了一下键。
    for _ in 0..100 {
        panel.step(KEY_OK);
    }
    for _ in 0..12 {
        panel.step(0);
    }
    assert!(
        panel.ui.line_input_open(&LINE_INPUT),
        "a long press must not close the keyboard"
    );
    // 打出的是光标下的 `q`：走到 `ok` 提交看看。
    panel.down(ROW0_MAX - 1);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.tap(KEY_OK);
    panel.up(ROW3_MAX - OK_COL);
    panel.tap(KEY_OK);
    assert!(!panel.ui.line_input_open(&LINE_INPUT));
    assert!(NAME.with(|text| text == "q"), "the long press typed once");
    assert_eq!(panel.ui.poll_event(), Some(EVENT_NAME));
}
