//! 电源阶段：距上次输入多久，该不该变暗或息屏。
//!
//! 库只报告阶段，不碰显示驱动：对比度、背光、面板开关都由应用按阶段自己调。

use super::Runtime;

/// 屏幕应当处于的电源阶段。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerPhase {
    /// 正常显示。
    Awake,
    /// 长时间无输入，应当调暗。
    Dim,
    /// 更长时间无输入，应当息屏。
    Sleep,
}

impl Runtime<'_> {
    /// 距最近一次按键或编码器动作过去了多少毫秒。
    pub fn idle_ms(&self) -> u32 {
        self.last_update.wrapping_sub(self.last_input)
    }

    /// 按当前空闲时长判断电源阶段。
    ///
    /// 阈值都为 0 时永远返回 [`PowerPhase::Awake`]；只设息屏不设变暗也可以。
    pub fn power_phase(&self) -> PowerPhase {
        let idle = self.idle_ms();
        if self.sleep_after_ms != 0 && idle >= self.sleep_after_ms {
            return PowerPhase::Sleep;
        }
        if self.dim_after_ms != 0 && idle >= self.dim_after_ms {
            return PowerPhase::Dim;
        }
        PowerPhase::Awake
    }

    /// 设置变暗与息屏的空闲阈值（毫秒）；任一项为 0 表示该阶段不启用。
    pub fn set_power_timeouts(&mut self, dim_after_ms: u32, sleep_after_ms: u32) {
        self.dim_after_ms = dim_after_ms;
        self.sleep_after_ms = sleep_after_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 只有阈值不参与构造，直接造一个最小运行时太啰嗦，这里只测判定逻辑。
    fn phase(idle: u32, dim: u32, sleep: u32) -> PowerPhase {
        if sleep != 0 && idle >= sleep {
            return PowerPhase::Sleep;
        }
        if dim != 0 && idle >= dim {
            return PowerPhase::Dim;
        }
        PowerPhase::Awake
    }

    #[test]
    fn no_thresholds_never_sleeps() {
        assert_eq!(phase(999_999, 0, 0), PowerPhase::Awake);
    }

    #[test]
    fn the_longest_elapsed_stage_wins() {
        assert_eq!(phase(100, 1000, 5000), PowerPhase::Awake);
        assert_eq!(phase(1000, 1000, 5000), PowerPhase::Dim);
        assert_eq!(phase(6000, 1000, 5000), PowerPhase::Sleep);
    }

    #[test]
    fn a_sleep_threshold_alone_still_fires() {
        assert_eq!(phase(4000, 0, 5000), PowerPhase::Awake);
        assert_eq!(phase(5000, 0, 5000), PowerPhase::Sleep);
    }
}
