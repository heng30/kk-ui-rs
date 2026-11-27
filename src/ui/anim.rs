//! 补间与定点换算。
//!
//! 曲线都是 Q12 采样表，65 个点均匀取样，取值 0..4096；`Overshoot` 会冲过 4096
//! 再回落，用它的调用方要接受「进度可以大于 1」。

use super::Q12_ONE;

/// 缓动曲线。整条 UI 的节奏从这里挑，页面不自己写数字。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Curve {
    /// 标准：进场与位移，`cubic-bezier(0.25, 0, 0.20, 1)`。
    Standard,
    /// 减速：抵达终点前先快后慢。
    Decel,
    /// 加速：离场，先慢后快。
    Accel,
    /// 过冲：小幅弹出后回稳，进度会超过 4096。
    Overshoot,
    /// 缓入缓出：扫掠类动作。
    InOut,
}

/// 标准曲线：`cubic-bezier(0.25, 0, 0.20, 1)` 均匀取样。
const STANDARD: [u16; 65] = [
    0, 6, 23, 53, 98, 159, 236, 329, 439, 565, 704, 853, 1010, 1170, 1331, 1490, 1645, 1795, 1938,
    2075, 2205, 2328, 2444, 2554, 2658, 2756, 2848, 2936, 3018, 3095, 3168, 3237, 3302, 3363, 3420,
    3475, 3526, 3574, 3619, 3661, 3701, 3738, 3773, 3806, 3836, 3864, 3891, 3915, 3938, 3959, 3978,
    3995, 4011, 4025, 4038, 4050, 4060, 4069, 4076, 4082, 4087, 4091, 4094, 4095, 4096,
];

/// 减速曲线：`cubic-bezier(0.05, 0.7, 0.1, 1)`。
const DECEL: [u16; 65] = [
    0, 802, 1370, 1768, 2063, 2293, 2480, 2635, 2768, 2883, 2984, 3074, 3154, 3226, 3292, 3351,
    3406, 3456, 3502, 3545, 3584, 3621, 3655, 3686, 3716, 3743, 3769, 3793, 3816, 3837, 3856, 3875,
    3892, 3908, 3924, 3938, 3951, 3964, 3976, 3987, 3997, 4007, 4015, 4024, 4032, 4039, 4045, 4051,
    4057, 4062, 4067, 4071, 4075, 4079, 4082, 4085, 4087, 4089, 4091, 4093, 4094, 4095, 4095, 4096,
    4096,
];

/// 加速曲线：`cubic-bezier(0.3, 0, 0.8, 0.15)`。
const ACCEL: [u16; 65] = [
    0, 1, 2, 5, 9, 14, 20, 27, 36, 45, 56, 67, 80, 95, 110, 127, 145, 164, 185, 206, 230, 254, 281,
    308, 337, 368, 400, 434, 470, 507, 547, 588, 631, 676, 723, 772, 824, 878, 934, 993, 1054,
    1119, 1186, 1256, 1330, 1407, 1488, 1573, 1661, 1754, 1852, 1955, 2064, 2178, 2299, 2427, 2563,
    2707, 2862, 3027, 3205, 3398, 3608, 3839, 4096,
];

/// 过冲曲线：`cubic-bezier(0.34, 1.45, 0.64, 1)`，峰值约 4366（≈1.066）。
const OVERSHOOT: [u16; 65] = [
    0, 268, 526, 774, 1013, 1242, 1461, 1671, 1871, 2062, 2244, 2417, 2581, 2737, 2884, 3023, 3153,
    3276, 3390, 3497, 3597, 3689, 3774, 3852, 3924, 3990, 4049, 4102, 4150, 4192, 4229, 4260, 4288,
    4310, 4329, 4343, 4354, 4361, 4365, 4366, 4365, 4360, 4354, 4345, 4335, 4323, 4310, 4296, 4281,
    4266, 4250, 4233, 4217, 4201, 4186, 4171, 4157, 4144, 4132, 4122, 4113, 4106, 4100, 4097, 4096,
];

/// 缓入缓出曲线：`cubic-bezier(0.65, 0, 0.35, 1)`。
const INOUT: [u16; 65] = [
    0, 1, 3, 8, 14, 22, 32, 44, 59, 77, 97, 120, 147, 176, 210, 248, 290, 337, 390, 448, 513, 585,
    665, 754, 852, 962, 1082, 1216, 1361, 1519, 1688, 1866, 2048, 2230, 2408, 2577, 2735, 2880,
    3014, 3134, 3244, 3342, 3431, 3511, 3583, 3648, 3706, 3759, 3806, 3848, 3886, 3920, 3949, 3976,
    3999, 4019, 4037, 4052, 4064, 4074, 4082, 4088, 4093, 4095, 4096,
];

impl Curve {
    /// 这条曲线的 Q12 采样表。
    fn table(self) -> &'static [u16; 65] {
        match self {
            Curve::Standard => &STANDARD,
            Curve::Decel => &DECEL,
            Curve::Accel => &ACCEL,
            Curve::Overshoot => &OVERSHOOT,
            Curve::InOut => &INOUT,
        }
    }
}

/// 按指定曲线把已过时间换算成 Q12 进度；`Overshoot` 的返回值可能大于 4096。
pub(crate) fn curve_q12(curve: Curve, elapsed: u32, duration: u32) -> u16 {
    let table = curve.table();
    if duration == 0 || elapsed >= duration {
        return table[64];
    }
    let reference = elapsed * 300 / duration;
    let scaled = reference * 64;
    let index = (scaled / 300) as usize;
    let fraction = scaled % 300;
    // 过冲曲线在峰值后会回落，相邻两点之差可能为负，必须用有符号运算。
    let value = table[index] as i64;
    let step = table[index + 1] as i64 - value;
    (value + step * fraction as i64 / 300) as u16
}

/// 按标准曲线把已过时间换算成 Q12 进度。
pub(crate) fn ease_q12(elapsed: u32, duration: u32) -> u16 {
    curve_q12(Curve::Standard, elapsed, duration)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 所有曲线都必须从 0 走到 4096。
    #[test]
    fn every_curve_spans_the_full_range() {
        for curve in [
            Curve::Standard,
            Curve::Decel,
            Curve::Accel,
            Curve::Overshoot,
            Curve::InOut,
        ] {
            assert_eq!(curve_q12(curve, 0, 200), 0, "{curve:?} must start at 0");
            assert_eq!(curve_q12(curve, 200, 200), 4096, "{curve:?} must end at 1");
            assert_eq!(
                curve_q12(curve, 500, 200),
                4096,
                "{curve:?} clamps past the end"
            );
            assert_eq!(
                curve_q12(curve, 0, 0),
                4096,
                "{curve:?} treats 0 ms as done"
            );
        }
    }

    /// 只有过冲曲线允许超过 4096，其余必须单调且不越界。
    #[test]
    fn only_overshoot_leaves_the_unit_interval() {
        for curve in [Curve::Standard, Curve::Decel, Curve::Accel, Curve::InOut] {
            let mut previous = 0;
            for step in 0..=200u32 {
                let value = curve_q12(curve, step, 200);
                assert!(value >= previous, "{curve:?} must never go backwards");
                assert!(value <= 4096, "{curve:?} must not overshoot");
                previous = value;
            }
        }
        let peak = (0..=200u32)
            .map(|step| curve_q12(Curve::Overshoot, step, 200))
            .max()
            .unwrap();
        assert!(peak > 4096, "overshoot must pass 4096, peaked at {peak}");
    }

    /// 减速曲线前段必须比加速曲线跑得快。
    #[test]
    fn decel_leads_and_accel_trails() {
        assert!(curve_q12(Curve::Decel, 60, 200) > curve_q12(Curve::Accel, 60, 200));
        assert!(curve_q12(Curve::Decel, 20, 200) > 1000);
    }
}
