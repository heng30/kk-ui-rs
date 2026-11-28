//! 动图图标：单尺寸位图的搬运与按时间取帧。
//!
//! 动图本体由库在 `kk_ui::anim_icons` 里声明，这里只验证绘制。

#![cfg(feature = "anim-icons")]

use kk_ui::anim_icons::{ACTIVITY_32, ACTIVITY_48, ACTIVITY_64};
use kk_ui::{AnimIcon, Frame, IconSize, draw_anim_icon, draw_anim_icon_at};

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
