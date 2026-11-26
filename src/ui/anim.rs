//! 补间与定点换算。

use super::Q12_ONE;

/// Q12 缓动曲线的采样表：`cubic-bezier(0.25, 0, 0.20, 1)` 均匀取样。
const EASE_CURVE: [u16; 65] = [
    0, 6, 23, 53, 98, 159, 236, 329, 439, 565, 704, 853, 1010, 1170, 1331, 1490, 1645, 1795, 1938,
    2075, 2205, 2328, 2444, 2554, 2658, 2756, 2848, 2936, 3018, 3095, 3168, 3237, 3302, 3363, 3420,
    3475, 3526, 3574, 3619, 3661, 3701, 3738, 3773, 3806, 3836, 3864, 3891, 3915, 3938, 3959, 3978,
    3995, 4011, 4025, 4038, 4050, 4060, 4069, 4076, 4082, 4087, 4091, 4094, 4095, 4096,
];

/// 按缓动曲线把已过时间换算成 Q12 进度。
pub(crate) fn ease_q12(elapsed: u32, duration: u32) -> u16 {
    if duration == 0 || elapsed >= duration {
        return Q12_ONE;
    }
    let reference = elapsed * 300 / duration;
    let scaled = reference * 64;
    let index = (scaled / 300) as usize;
    let fraction = scaled % 300;
    let value = EASE_CURVE[index] as u32;
    (value + (EASE_CURVE[index + 1] as u32 - value) * fraction / 300) as u16
}

/// Q12 线性插值。
pub(crate) fn lerp_q12(from: i32, to: i32, progress: u16) -> i32 {
    from + ((to as i64 - from as i64) * progress as i64 / Q12_ONE as i64) as i32
}

/// Q8 定点数四舍五入到整数像素。
pub(crate) fn round_q8(value: i32) -> i16 {
    if value >= 0 {
        ((value + 128) / 256) as i16
    } else {
        ((value - 128) / 256) as i16
    }
}
