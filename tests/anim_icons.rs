//! 动图图标：单尺寸位图的搬运与按时间取帧。
//!
//! 动图本体由库在 `kk_ui::anim_icons` 里声明，这里只验证绘制。

#![cfg(feature = "anim-icons")]

use kk_ui::anim_icons::{ACTIVITY_32, ACTIVITY_48, ACTIVITY_64};
use kk_ui::{
    AnimIcon, App, Frame, Glyphs, HomeIcon, HomeItem, HomePage, IconSize, Input, KkUi, MenuPage,
    PageRoute, Status, Texts, draw_anim_icon, draw_anim_icon_at, set_font,
};

/// 动图某一帧的置位像素必须原样落到帧缓冲的对应坐标。
#[test]
fn blits_a_frame_pixel_for_pixel() {
    let index = 3;
    let blob = ACTIVITY_32.frames;
    let stride = 32;
    let mut buf = [0u8; 128 * 8];
    let mut frame = Frame::new(&mut buf, 128, 64);
    draw_anim_icon(&mut frame, 4, 8, &ACTIVITY_32, index);
    for dy in 0..stride as i16 {
        for dx in 0..stride as i16 {
            let byte = blob[index * 128 + dx as usize + (dy as usize >> 3) * stride];
            let on = byte & (1 << (dy & 7)) != 0;
            assert_eq!(frame.pixel(4 + dx, 8 + dy), on, "pixel {dx},{dy}");
        }
    }
}

/// 落在画布外的部分被裁掉，剩下的仍按原坐标对齐。
#[test]
fn clips_at_the_canvas_edge() {
    let mut buf = [0u8; 128 * 8];
    let mut frame = Frame::new(&mut buf, 128, 64);
    draw_anim_icon(&mut frame, -8, -8, &ACTIVITY_32, 0);
    let blob = ACTIVITY_32.frames;
    for cy in 0..24usize {
        for cx in 0..24usize {
            let byte = blob[(cx + 8) + ((cy + 8) >> 3) * 32];
            let on = byte & (1 << ((cy + 8) & 7)) != 0;
            assert_eq!(frame.pixel(cx as i16, cy as i16), on);
        }
    }
}

/// 帧序号循环。
#[test]
fn wraps_frames() {
    let count = ACTIVITY_48.frame_count();
    let mut a = [0u8; 128 * 8];
    let mut b = [0u8; 128 * 8];
    draw_anim_icon(&mut Frame::new(&mut a, 128, 64), 0, 0, &ACTIVITY_48, 1);
    draw_anim_icon(
        &mut Frame::new(&mut b, 128, 64),
        0,
        0,
        &ACTIVITY_48,
        1 + count,
    );
    assert_eq!(a, b);
}

/// 按毫秒取帧：24 fps，42 ms 就是第 1 帧。
#[test]
fn picks_the_frame_from_elapsed_time() {
    let mut a = [0u8; 128 * 8];
    let mut b = [0u8; 128 * 8];
    draw_anim_icon_at(&mut Frame::new(&mut a, 128, 64), 0, 0, &ACTIVITY_64, 0);
    draw_anim_icon(&mut Frame::new(&mut b, 128, 64), 0, 0, &ACTIVITY_64, 0);
    assert_eq!(a, b);

    let mut c = [0u8; 128 * 8];
    let mut d = [0u8; 128 * 8];
    draw_anim_icon_at(&mut Frame::new(&mut c, 128, 64), 0, 0, &ACTIVITY_64, 42);
    draw_anim_icon(&mut Frame::new(&mut d, 128, 64), 0, 0, &ACTIVITY_64, 1);
    assert_eq!(c, d);
}

/// 每个尺寸各自成静态：边长、单帧字节数与帧数互相对得上。
#[test]
fn sizes_are_separate_statics() {
    let all: [(IconSize, &AnimIcon); 3] = [
        (IconSize::Px32, &ACTIVITY_32),
        (IconSize::Px48, &ACTIVITY_48),
        (IconSize::Px64, &ACTIVITY_64),
    ];
    assert_eq!(
        IconSize::ALL,
        [IconSize::Px32, IconSize::Px48, IconSize::Px64]
    );
    for (size, anim) in all {
        assert_eq!(anim.size, size);
        assert_eq!(
            size.frame_len(),
            size.px() as usize * size.px() as usize / 8
        );
        assert_eq!(anim.frames.len(), anim.frame_count() * size.frame_len());
        assert_eq!(anim.frame_count(), ACTIVITY_32.frame_count());
    }
}

/// 单色静态图标（128 字节，32×32）。
static STATIC_ICON: [u8; 128] = [0xaa; 128];

/// 空字库：缺字，文字绘制为空但仍占位。
static GLYPHS: Glyphs = Glyphs {
    bitmaps: &[],
    widths: &[],
    codepoints: &[],
};

static TEXT_TARGET: PageRoute = PageRoute::Menu(&MenuPage {
    title: "T",
    items: &[],
    states: None,
});

/// 首页入口：一个动图加一个静态图标。
static MIXED_ITEMS: [HomeItem; 2] = [
    HomeItem {
        label: "A",
        icon: HomeIcon::Anim(&ACTIVITY_32),
        target: &TEXT_TARGET,
    },
    HomeItem {
        label: "B",
        icon: HomeIcon::Static(&STATIC_ICON),
        target: &TEXT_TARGET,
    },
];

/// 首页入口：只有静态图标。
static STATIC_ITEMS: [HomeItem; 2] = [
    HomeItem {
        label: "A",
        icon: HomeIcon::Static(&STATIC_ICON),
        target: &TEXT_TARGET,
    },
    HomeItem {
        label: "B",
        icon: HomeIcon::Static(&STATIC_ICON),
        target: &TEXT_TARGET,
    },
];

static ROUTE_MIXED: PageRoute = PageRoute::Home(&HomePage {
    items: &MIXED_ITEMS,
});
static ROUTE_STATIC: PageRoute = PageRoute::Home(&HomePage {
    items: &STATIC_ITEMS,
});

static MIXED_APP: App = App {
    root_page: &ROUTE_MIXED,
    routes: &MIXED_ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
    line_inputs: &[],
    texts: TEXTS,
};
static STATIC_APP: App = App {
    root_page: &ROUTE_STATIC,
    routes: &STATIC_ROUTES,
    int_bindings: &[],
    bool_bindings: &[],
    confirm_descs: &[],
    line_inputs: &[],
    texts: TEXTS,
};
static MIXED_ROUTES: [&PageRoute; 2] = [&ROUTE_MIXED, &TEXT_TARGET];
static STATIC_ROUTES: [&PageRoute; 2] = [&ROUTE_STATIC, &TEXT_TARGET];

const TEXTS: Texts = Texts {
    return_text: "Back",
    cancel_text: "Cancel",
    confirm_text: "OK",
    on_text: "On",
    off_text: "Off",
    message_title: "Message",
};

/// 在一台 128×64 的画布上起一个首页。
fn home_ui(app: &'static App) -> KkUi<'static> {
    set_font(&GLYPHS);
    let buffer: &'static mut [u8] =
        Box::leak(vec![0u8; Frame::buffer_len(128, 64)].into_boxed_slice());
    let mut ui = KkUi::new(app, buffer, 128, 64);
    assert_eq!(ui.init(), Status::Ok);
    ui
}

/// 空按若干拍，返回画布快照。
fn advance(ui: &mut KkUi<'static>, now: &mut u32, ticks: u32) -> Vec<u8> {
    for _ in 0..ticks {
        *now += 5;
        ui.update(
            *now,
            Input {
                keys: 0,
                encoder_delta: 0,
            },
        );
    }
    ui.frame().buffer().to_vec()
}

/// 首页绘制动图而不止静态图标：没有输入也会随 24 fps 换帧重画。
#[test]
fn home_page_animates_its_icon() {
    let mut ui = home_ui(&MIXED_APP);
    let mut now = 0;
    advance(&mut ui, &mut now, 40);
    let first = advance(&mut ui, &mut now, 1);
    let later = advance(&mut ui, &mut now, 40);
    assert_ne!(first, later);
    assert!(later.iter().any(|&byte| byte != 0));
}

/// 全是静态图标的首页没人按键就不重画。
#[test]
fn static_home_page_stays_quiet() {
    let mut ui = home_ui(&STATIC_APP);
    let mut now = 0;
    advance(&mut ui, &mut now, 40);
    ui.take_dirty_pages();
    advance(&mut ui, &mut now, 40);
    assert_eq!(ui.take_dirty_pages(), 0);
}
