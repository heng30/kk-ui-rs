//! 定点弹簧：一个动画值只有位置、速度与目标三个整数，不需要浮点。
//!
//! 子步长固定为 [`SPRING_SUBSTEP_MS`]，用半隐式欧拉积分；刚度与阻尼在编译期就乘好
//! 子步长（Q12），所以每个子步只有两次乘法与两次移位。位置与速度的单位都是
//! 「像素 × 256」，即 Q8。

use super::{SPRING_MAX_SUBSTEPS, SPRING_SUBSTEP_MS};

/// 位置与目标相差不到 1/4 像素、速度不到 4 像素每秒时算静止。
const SETTLE_POSITION_Q8: i32 = 64;
/// 速度的静止阈值，Q8。
const SETTLE_VELOCITY_Q8: i32 = 1024;
/// 子步长本身，Q12。
const SUBSTEP_Q12: i32 = 33;

/// 弹簧的积分参数：物理系数乘上子步长后的定点值，Q12。
///
/// `stiffness = (2π / 响应时长)² · 子步长`，`damping = 4π · 阻尼比 / 响应时长 · 子步长`，
/// 与 SwiftUI 的 Response / Damping 写法一致，只是全部预先算成整数。
#[derive(Clone, Copy)]
pub(crate) struct SpringPreset {
    /// 刚度，Q12。
    stiffness_q12: i32,
    /// 阻尼，Q12。
    damping_q12: i32,
}

impl SpringPreset {
    /// 由预先算好的两个系数构造。
    pub(crate) const fn new(stiffness_q12: i32, damping_q12: i32) -> Self {
        Self {
            stiffness_q12,
            damping_q12,
        }
    }
}

/// 一个弹向目标的定点值。
#[derive(Clone, Copy, Default)]
pub(crate) struct Spring {
    /// 当前位置，Q8。
    position_q8: i32,
    /// 当前速度，Q8 每秒。
    velocity_q8: i32,
    /// 目标位置，Q8。
    target_q8: i32,
    /// 是否还在动。
    active: bool,
}

impl Spring {
    /// 静止在 0。
    pub(crate) const REST: Self = Self::at(0);

    /// 静止在给定位置。
    pub(crate) const fn at(position_q8: i32) -> Self {
        Self {
            position_q8,
            velocity_q8: 0,
            target_q8: position_q8,
            active: false,
        }
    }

    /// 当前位置，Q8。
    pub(crate) fn value_q8(&self) -> i32 {
        self.position_q8
    }

    /// 瞬移到给定位置并停下。
    pub(crate) fn jump(&mut self, position_q8: i32) {
        self.position_q8 = position_q8;
        self.target_q8 = position_q8;
        self.velocity_q8 = 0;
        self.active = false;
    }

    /// 改目标；目标不变时什么也不做，避免重启已经静止的动画。
    pub(crate) fn set(&mut self, target_q8: i32) {
        if target_q8 == self.target_q8 {
            return;
        }
        self.target_q8 = target_q8;
        self.active = true;
    }

    /// 推进 `dt_ms` 毫秒，返回是否还在动。
    ///
    /// 一帧最多积分 [`SPRING_MAX_SUBSTEPS`] 个子步，掉帧时宁可慢一点也不要抖。
    pub(crate) fn step(&mut self, dt_ms: i32, preset: SpringPreset) -> bool {
        if !self.active {
            return false;
        }
        let substeps = (dt_ms / SPRING_SUBSTEP_MS).clamp(1, SPRING_MAX_SUBSTEPS);
        for _ in 0..substeps {
            let offset = self.position_q8 - self.target_q8;
            let pull = (preset.stiffness_q12 * offset) >> 12;
            let drag = (preset.damping_q12 * self.velocity_q8) >> 12;
            self.velocity_q8 -= pull + drag;
            self.position_q8 += (self.velocity_q8 * SUBSTEP_Q12) >> 12;
            if (self.position_q8 - self.target_q8).abs() <= SETTLE_POSITION_Q8
                && self.velocity_q8.abs() <= SETTLE_VELOCITY_Q8
            {
                self.position_q8 = self.target_q8;
                self.velocity_q8 = 0;
                self.active = false;
                break;
            }
        }
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::SPRING_BOUNCY;

    /// 临界阻尼：响应 0.8 s、阻尼比 1，用来验证「不过冲」这条性质。
    const GLIDE: SpringPreset = SpringPreset::new(2021, 515);

    /// 跑一帧（20 ms），返回有没有停。
    fn run(spring: &mut Spring, preset: SpringPreset, frames: usize) -> usize {
        for frame in 1..=frames {
            if !spring.step(20, preset) {
                return frame;
            }
        }
        frames + 1
    }

    #[test]
    fn critically_damped_spring_never_overshoots() {
        let mut spring = Spring::REST;
        spring.set(65 * 256);
        for _ in 0..120 {
            spring.step(20, GLIDE);
            assert!(
                spring.value_q8() <= 65 * 256,
                "critically damped spring overshot: {}",
                spring.value_q8()
            );
        }
        assert_eq!(spring.value_q8(), 65 * 256);
        assert!(!spring.step(20, GLIDE), "it must be at rest");
    }

    #[test]
    fn bouncy_overshoots_then_settles() {
        let mut spring = Spring::REST;
        spring.set(40 * 256);
        let mut peak = 0;
        for _ in 0..80 {
            spring.step(20, SPRING_BOUNCY);
            peak = peak.max(spring.value_q8());
        }
        assert!(peak > 40 * 256, "underdamped spring never overshot");
        assert_eq!(spring.value_q8(), 40 * 256);
    }

    #[test]
    fn settles_in_reasonable_time() {
        let mut spring = Spring::REST;
        spring.set(90 * 256);
        let frames = run(&mut spring, SPRING_BOUNCY, 120);
        assert!(frames <= 60, "spring took {frames} frames to settle");
    }

    #[test]
    fn dt_is_clamped_so_a_stall_cannot_fling_it() {
        let mut spring = Spring::REST;
        spring.set(20 * 256);
        spring.step(1000, SPRING_BOUNCY);
        assert!(
            spring.value_q8().abs() < 200 * 256,
            "one huge frame flew off"
        );
    }
}
