//! 新增组件：分段单选行、位掩码行、枚举选择弹窗都只靠上/下/确定三个键操作。

use kk_ui::{
    App, BitsBinding, ChipBinding, ChoiceBinding, Frame, Glyphs, HomePage, Input, KEY_DOWN, KEY_OK,
    KEY_UP, KkUi, MenuAction, MenuItem, MenuPage, PageRoute, Shared, Texts, set_font,
};

kk_ui::event_ids! {
    EVENT_LEVEL,
    EVENT_CHANNELS,
    EVENT_REFRESH,
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

static LEVEL: Shared<u8> = Shared::new(0);
static CHANNELS: Shared<u32> = Shared::new(0);
static REFRESH: Shared<u16> = Shared::new(0);

static MENU_ITEMS: [MenuItem; 3] = [
    MenuItem {
        label: "Level",
        action: MenuAction::Chip(&CHIP_LEVEL),
    },
    MenuItem {
        label: "Channels",
        action: MenuAction::Bits(&BITS_CHANNELS),
    },
    MenuItem {
        label: "Refresh",
        action: MenuAction::Choice(&CHOICE_REFRESH),
    },
];

static HOME_ITEMS: [kk_ui::HomeItem; 1] = [kk_ui::HomeItem {
    label: "Menu",
    icon: &ICON,
    target: &ROUTE_MENU,
}];

static ROUTE_HOME: PageRoute = PageRoute::Home(&HomePage { items: &HOME_ITEMS });
static ROUTE_MENU: PageRoute = PageRoute::Menu(&MenuPage {
    title: "Menu",
    items: &MENU_ITEMS,
    states: None,
});
static ROUTES: [&PageRoute; 2] = [&ROUTE_HOME, &ROUTE_MENU];

static CHIP_LEVEL: ChipBinding = ChipBinding {
    levels: &["Low", "Mid", "High"],
    value: &LEVEL,
    changed_event: EVENT_LEVEL,
};
static BITS_CHANNELS: BitsBinding = BitsBinding {
    title: "Channels",
    labels: &["CH0", "CH1", "CH2"],
    value: &CHANNELS,
    changed_event: EVENT_CHANNELS,
};
static CHOICE_REFRESH: ChoiceBinding = ChoiceBinding {
    title: "Refresh",
    labels: &["30Hz", "60Hz", "120Hz"],
    value: &REFRESH,
    changed_event: EVENT_REFRESH,
};

static APP: App = App {
    root_page: &ROUTE_HOME,
    routes: &ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
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

/// 用例之间共享 `LEVEL` / `CHANNELS` / `REFRESH`，并行跑会互相污染，用锁串起来。
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 一台只有按键的模拟面板；持有它期间独占上面那几个静态值。
struct Panel {
    ui: KkUi<'static>,
    now: u32,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Panel {
    fn new() -> Self {
        let guard = LOCK.lock().unwrap_or_else(|error| error.into_inner());
        // 测试不开 cjk-fonts，默认无字体；装上空字库让布局走真实代码路径。
        set_font(&GLYPHS);
        let buffer: &'static mut [u8] =
            Box::leak(vec![0u8; Frame::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        let mut ui = KkUi::new(&APP, buffer, WIDTH, HEIGHT);
        assert_eq!(ui.init(), kk_ui::Status::Ok);
        LEVEL.set(0);
        CHANNELS.set(0);
        REFRESH.set(0);
        let mut panel = Self {
            ui,
            now: 0,
            _guard: guard,
        };
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

    fn enter_menu(&mut self) {
        self.press(KEY_OK);
    }
}

#[test]
fn chip_cycles_through_every_level() {
    let mut panel = Panel::new();
    panel.enter_menu();
    assert_eq!(LEVEL.get(), 0);
    panel.press(KEY_OK);
    assert_eq!(LEVEL.get(), 1, "OK advances one level");
    assert_eq!(panel.ui.poll_event(), Some(EVENT_LEVEL));
    panel.press(KEY_OK);
    assert_eq!(LEVEL.get(), 2);
    panel.press(KEY_OK);
    assert_eq!(LEVEL.get(), 0, "the last level wraps to the first");
}

#[test]
fn bits_editor_flips_the_chosen_bit_and_skips_the_rest() {
    let mut panel = Panel::new();
    panel.enter_menu();
    panel.press(KEY_DOWN); // 选中通道行
    panel.press(KEY_OK); // 打开位掩码弹窗
    assert!(panel.ui.dialog_open());
    panel.press(KEY_OK); // 翻转第 0 位
    assert_eq!(panel.ui.get_bits_editor_draft(&BITS_CHANNELS), Some(0b001));
    panel.press(KEY_DOWN); // 跳过第 1 位
    panel.press(KEY_DOWN); // 游标到第 2 位
    panel.press(KEY_OK); // 翻转第 2 位，跳过的第 1 位保持关闭
    assert_eq!(
        panel.ui.get_bits_editor_draft(&BITS_CHANNELS),
        Some(0b101),
        "the bit we skipped must stay off"
    );
    // 末位按确定已经顺手把焦点交给确定键，再按一下就提交。
    panel.press(KEY_OK); // 提交并关闭
    assert!(!panel.ui.dialog_open(), "editor must close");
    assert_eq!(CHANNELS.get(), 0b101);
    assert_eq!(panel.ui.poll_event(), Some(EVENT_CHANNELS));
}

#[test]
fn bits_editor_cancels_from_the_first_bit() {
    let mut panel = Panel::new();
    panel.enter_menu();
    panel.press(KEY_DOWN);
    panel.press(KEY_OK); // 打开
    panel.press(KEY_OK); // 翻转第 0 位
    panel.press(KEY_UP); // 在第一位按上：焦点交给取消
    panel.press(KEY_OK); // 取消
    assert!(!panel.ui.dialog_open(), "editor must close");
    assert_eq!(CHANNELS.get(), 0, "cancel must not write the draft");
}

#[test]
fn choice_editor_changes_the_index_and_commits() {
    let mut panel = Panel::new();
    panel.enter_menu();
    panel.press(KEY_DOWN);
    panel.press(KEY_DOWN);
    panel.press(KEY_OK); // 打开枚举选择弹窗
    assert!(panel.ui.dialog_open());
    panel.press(KEY_DOWN); // 下一个标签
    panel.press(KEY_OK); // 结束编辑，焦点落到确认键
    panel.press(KEY_OK); // 提交并关闭
    assert!(!panel.ui.dialog_open(), "editor must close");
    assert_eq!(REFRESH.get(), 1, "committed index");
    assert_eq!(panel.ui.poll_event(), Some(EVENT_REFRESH));
}

#[test]
fn choice_editor_stops_at_the_last_label() {
    let mut panel = Panel::new();
    panel.enter_menu();
    panel.press(KEY_DOWN);
    panel.press(KEY_DOWN);
    panel.press(KEY_OK);
    for _ in 0..5 {
        panel.press(KEY_DOWN);
    }
    panel.press(KEY_OK);
    panel.press(KEY_OK);
    assert_eq!(REFRESH.get(), 2, "index is clamped to the label count");
}

#[test]
fn power_phase_follows_the_time_since_the_last_key() {
    let mut panel = Panel::new();
    panel.ui.set_power_timeouts(100, 300);
    assert_eq!(panel.ui.idle_ms(), 0);
    assert_eq!(panel.ui.power_phase(), kk_ui::PowerPhase::Awake);

    // 空转到变暗阈值：只过了时间，没有输入。
    for _ in 0..20 {
        panel.step(0);
    }
    assert_eq!(panel.ui.idle_ms(), 100);
    assert_eq!(panel.ui.power_phase(), kk_ui::PowerPhase::Dim);

    for _ in 0..40 {
        panel.step(0);
    }
    assert_eq!(panel.ui.power_phase(), kk_ui::PowerPhase::Sleep);

    // 一次按键把计时清零。
    panel.step(KEY_DOWN);
    assert_eq!(panel.ui.idle_ms(), 0);
    assert_eq!(panel.ui.power_phase(), kk_ui::PowerPhase::Awake);
}

#[test]
fn zero_timeouts_disable_the_phases() {
    let mut panel = Panel::new();
    panel.ui.set_power_timeouts(0, 0);
    for _ in 0..200 {
        panel.step(0);
    }
    assert_eq!(panel.ui.power_phase(), kk_ui::PowerPhase::Awake);
}

/// 按住 `keys` 若干拍再松开：`hold_ms` 超过 400 ms 才算长按。
fn hold(panel: &mut Panel, keys: u8, hold_ms: u32) {
    for _ in 0..hold_ms / 5 {
        panel.step(keys);
    }
    for _ in 0..70 {
        panel.step(0);
    }
}

#[test]
fn a_short_ok_press_confirms() {
    let mut panel = Panel::new();
    panel.enter_menu();
    hold(&mut panel, KEY_OK, 40);
    assert_eq!(LEVEL.get(), 1, "短按确认：切到下一档");
    assert_eq!(panel.ui.poll_event(), Some(EVENT_LEVEL));
}

#[test]
fn holding_ok_long_enough_goes_back_instead_of_confirming() {
    let mut panel = Panel::new();
    panel.enter_menu();
    hold(&mut panel, KEY_OK, 600);
    assert_eq!(LEVEL.get(), 0, "长按返回，不能顺手把取值改掉");
    assert_eq!(panel.ui.poll_event(), None, "返回不该入队取值事件");
    assert!(
        core::ptr::eq(panel.ui.current_page().expect("page"), &ROUTE_HOME),
        "长按返回上一页"
    );
}

#[test]
fn holding_ok_in_a_dialog_cancels_it() {
    let mut panel = Panel::new();
    panel.enter_menu();
    panel.press(KEY_DOWN); // 通道行
    panel.press(KEY_OK); // 打开位掩码弹窗
    assert!(panel.ui.dialog_open());
    hold(&mut panel, KEY_OK, 600);
    assert!(!panel.ui.dialog_open(), "长按取消，弹窗关闭");
    assert_eq!(CHANNELS.get(), 0, "取消不能写回草稿");
}

#[test]
fn an_ok_press_during_a_transition_is_swallowed() {
    let mut panel = Panel::new();
    // 直接发起导航，让转场处于播放中，此时才按住确定。
    assert_eq!(
        panel.ui.navigate_to(&ROUTE_MENU, panel.now),
        kk_ui::Status::Ok
    );
    hold(&mut panel, KEY_OK, 40);
    assert!(
        core::ptr::eq(panel.ui.current_page().expect("page"), &ROUTE_MENU),
        "转场中按下的确定键不能触发返回，页面保持不动"
    );
    assert_eq!(LEVEL.get(), 0, "也不该顺手确认任何条目");
}

#[test]
fn the_first_frame_is_fully_dirty_and_a_still_frame_reports_nothing() {
    let mut panel = Panel::new();
    // 画布 64 px 高 = 8 页，首次提交要全画。
    assert_eq!(panel.ui.take_dirty_pages(), 0xFF);
    assert_eq!(panel.ui.take_dirty_pages(), 0, "没有新画面就没有脏页");

    panel.step(0);
    panel.step(0);
    let idle = panel.ui.take_dirty_pages();
    assert_eq!(idle, 0, "静止时不该再报脏页");
}

#[test]
fn a_change_marks_at_least_the_touched_page() {
    let mut panel = Panel::new();
    panel.enter_menu();
    let _ = panel.ui.take_dirty_pages();
    panel.press(KEY_DOWN);
    let dirty = panel.ui.take_dirty_pages();
    assert_ne!(dirty, 0, "换行要报脏页");
    assert_eq!(dirty & !0xFF, 0, "只能报 8 页范围内的位");
}

#[test]
fn the_fade_transition_works_without_a_second_buffer() {
    let mut panel = Panel::new();
    panel.ui.set_transition_kind(kk_ui::TransitionKind::Fade);
    assert_eq!(
        panel.ui.navigate_to(&ROUTE_MENU, panel.now),
        kk_ui::Status::Ok
    );
    // 转场跑完前后都要能出画面，不 panic、不越界。
    panel.step(0);
    let mid = panel.ui.frame().buffer().to_vec();
    for _ in 0..70 {
        panel.step(0);
    }
    let settled = panel.ui.frame().buffer().to_vec();
    assert_ne!(mid, settled, "fade 中途与终态的画面不同");
    assert!(
        core::ptr::eq(panel.ui.current_page().expect("page"), &ROUTE_MENU),
        "fade 结束后停在目标页"
    );
}
