//! 菜单滚动：滚到底再绕回第一行，第一行必须重新出现在窗口里。

use kk_ui::{
    App, Frame, Glyphs, Input, KEY_DOWN, KEY_OK, KEY_UP, KkUi, MenuAction, MenuItem, MenuPage,
    PageRoute, Shared, Texts, set_font,
};

kk_ui::event_ids! {
    EVENT_ROW,
}

/// 画布尺寸：128 高时按 18 px 行距只放得下 5 行，8 个条目必须滚动。
const WIDTH: i16 = 160;
const HEIGHT: i16 = 120;

/// 空字库：文字不画，但按全角步进占位，布局与真字库一致。
static GLYPHS: Glyphs = Glyphs {
    bitmaps: &[],
    widths: &[],
    codepoints: &[],
};

static ROW0: Shared<u8> = Shared::new(0);

static ITEMS: [MenuItem; 7] = [
    MenuItem {
        label: "Zero",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "One",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "Two",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "Three",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "Four",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "Five",
        action: MenuAction::Event(EVENT_ROW),
    },
    MenuItem {
        label: "Six",
        action: MenuAction::Event(EVENT_ROW),
    },
];

static ROUTE_MENU: PageRoute = PageRoute::Menu(&MenuPage {
    title: "Scroll",
    items: &ITEMS,
    states: None,
});

/// 第 0 行标成禁用：设置页的「屏幕亮度」就是这样，选不中但占一行。
static DISABLED_STATES: [Shared<u8>; 2] = [Shared::new(0b10), Shared::new(0)];

static ROUTE_DISABLED: PageRoute = PageRoute::Menu(&MenuPage {
    title: "Scroll",
    items: &ITEMS,
    states: Some(&DISABLED_STATES),
});

static ROUTES: [&PageRoute; 2] = [&ROUTE_MENU, &ROUTE_DISABLED];

static APP: App = App {
    root_page: &ROUTE_MENU,
    routes: &ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
    texts: Texts {
        return_text: "Back",
        cancel_text: "Cancel",
        confirm_text: "OK",
        on_text: "On",
        off_text: "Off",
        message_title: "Message",
    },
};

static APP_DISABLED: App = App {
    root_page: &ROUTE_DISABLED,
    routes: &ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
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
    fn new(app: &'static App) -> Self {
        set_font(&GLYPHS);
        let _ = ROW0;
        let buffer: &'static mut [u8] =
            Box::leak(vec![0u8; Frame::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        let mut ui = KkUi::new(app, buffer, WIDTH, HEIGHT);
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

    /// 按下并松开一次按键（按住 40 ms，超过 20 ms 消抖），然后空转到动画跑完。
    fn press(&mut self, keys: u8) {
        for _ in 0..8 {
            self.step(keys);
        }
        for _ in 0..70 {
            self.step(0);
        }
    }

    /// 当前画面。
    fn shot(&self) -> Vec<u8> {
        self.ui.frame().buffer().to_vec()
    }
}

#[test]
fn the_first_row_comes_back_after_wrapping_to_the_end() {
    let mut panel = Panel::new(&APP);
    let top = panel.shot();
    let rows = ITEMS.len() as u16 + 1;
    for _ in 0..rows {
        panel.press(KEY_DOWN);
    }
    assert_eq!(panel.ui.selected_index(), 0, "a full turn lands on row 0");
    assert_eq!(
        panel.shot(),
        top,
        "row 0 must be back in the window, scrolled to the top"
    );
}

#[test]
fn scrolling_up_wraps_to_the_last_row_and_back() {
    let mut panel = Panel::new(&APP);
    panel.press(KEY_UP);
    assert_eq!(
        panel.ui.selected_index(),
        ITEMS.len() as u16,
        "up from the first row wraps to the return item"
    );
    panel.press(KEY_DOWN);
    assert_eq!(panel.ui.selected_index(), 0);
    assert_eq!(panel.shot(), Panel::new(&APP).shot());
}

#[test]
fn every_row_is_visible_when_it_is_selected() {
    let mut panel = Panel::new(&APP);
    for step in 0..ITEMS.len() as u16 + 2 {
        assert_eq!(
            panel.ui.selected_index(),
            step % (ITEMS.len() as u16 + 1),
            "focus must follow the keys"
        );
        let focused = panel.shot();
        panel.press(KEY_OK);
        assert_eq!(
            panel.shot(),
            focused,
            "opening a row must not move the list"
        );
        panel.press(KEY_DOWN);
    }
}

#[test]
fn a_disabled_first_row_comes_back_into_the_window() {
    let mut panel = Panel::new(&APP_DISABLED);
    // 第 0 行禁用，焦点从第 1 行起步。
    assert_eq!(panel.ui.selected_index(), 1);
    let top = panel.shot();
    // 往下滚到底，再绕回第 1 行：窗口必须重新露出第 0 行。
    for _ in 0..7 {
        panel.press(KEY_DOWN);
    }
    assert_eq!(panel.ui.selected_index(), 1);
    assert_eq!(
        panel.shot(),
        top,
        "the disabled row above must be scrolled back into view"
    );
}
