//! 跑马灯：放不下的一行先停一下，匀速滑到尾部再停一下，然后匀速回位。
//!
//! 一次只滚一行（当前焦点行），所以一份状态就够。位置不缓存：每次按「阶段起点 +
//! 匀速」现算，掉帧也不会越走越偏。

use crate::ui::{MARQUEE_HOLD_MS, MARQUEE_SPEED_PX_S, MarqueeState};

/// 停在起点的阶段。
pub(crate) const HOLD_START: u8 = 0;
/// 向左滑到尾部的阶段。
pub(crate) const FORWARD: u8 = 1;
/// 停在尾部的阶段。
pub(crate) const HOLD_END: u8 = 2;
/// 向右滑回起点的阶段。
pub(crate) const BACK: u8 = 3;

/// 从阶段起点到现在走了多少像素。
fn travelled(now: u32, started: u32) -> i32 {
    now.wrapping_sub(started) as i32 * MARQUEE_SPEED_PX_S / 1000
}

/// 推进跑马灯并返回这一帧的横向偏移（像素，负值表示向左）。
///
/// `overflow` 是文字超出可用宽度的像素数，不大于 0 表示放得下，不需要滚动。
pub(crate) fn advance(state: &mut MarqueeState, row: u16, now: u32, overflow: i16) -> i16 {
    if overflow <= 0 || state.row != row {
        state.row = row;
        state.phase = HOLD_START;
        state.started = now;
        return 0;
    }
    let overflow = overflow as i32;
    let elapsed = now.wrapping_sub(state.started);
    match state.phase {
        FORWARD => {
            let moved = travelled(now, state.started);
            if moved >= overflow {
                state.phase = HOLD_END;
                state.started = now;
                return -overflow as i16;
            }
            -moved as i16
        }
        HOLD_END => {
            if elapsed >= MARQUEE_HOLD_MS {
                state.phase = BACK;
                state.started = now;
            }
            -overflow as i16
        }
        BACK => {
            let moved = travelled(now, state.started);
            if moved >= overflow {
                state.phase = HOLD_START;
                state.started = now;
                return 0;
            }
            -(overflow - moved) as i16
        }
        _ => {
            if elapsed >= MARQUEE_HOLD_MS {
                state.phase = FORWARD;
                state.started = now;
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 推进到某个阶段，返回那一拍的偏移。
    fn run_until(state: &mut MarqueeState, now: &mut u32, end: u8) -> i16 {
        let mut offset = 0;
        for _ in 0..600 {
            *now += 10;
            offset = advance(state, 7, *now, 40);
            if state.phase == end {
                return offset;
            }
        }
        offset
    }

    #[test]
    fn a_label_that_fits_never_moves() {
        let mut state = MarqueeState::EMPTY;
        let mut now = 0;
        for _ in 0..60 {
            now += 10;
            assert_eq!(advance(&mut state, 7, now, 0), 0);
        }
        assert_eq!(state.phase, HOLD_START);
    }

    #[test]
    fn the_text_moves_at_a_constant_speed() {
        let mut state = MarqueeState::EMPTY;
        let mut now = 0;
        let mut offset = 0;
        while state.phase == HOLD_START {
            now += 10;
            offset = advance(&mut state, 7, now, 48);
        }
        assert_eq!(offset, 0, "it starts from the beginning");
        let start = state.started;
        // 每秒 24 px：半秒 12 px，一秒 24 px，与已走的时长成正比。
        assert_eq!(advance(&mut state, 7, start + 500, 48), -12);
        assert_eq!(advance(&mut state, 7, start + 1000, 48), -24);
        assert_eq!(advance(&mut state, 7, start + 1500, 48), -36);
        assert_eq!(advance(&mut state, 7, start + 2000, 48), -48);
        assert_eq!(state.phase, HOLD_END, "it stops at the tail");
    }

    #[test]
    fn it_holds_then_comes_back_at_the_same_speed() {
        let mut state = MarqueeState::EMPTY;
        let mut now = 0;
        assert_eq!(run_until(&mut state, &mut now, HOLD_END), -40);
        while state.phase == HOLD_END {
            now += 10;
            advance(&mut state, 7, now, 40);
        }
        assert_eq!(state.phase, BACK);
        let start = state.started;
        assert_eq!(advance(&mut state, 7, start + 500, 40), -28);
        assert_eq!(advance(&mut state, 7, start + 1000, 40), -16);
        assert_eq!(advance(&mut state, 7, start + 2000, 40), 0);
        assert_eq!(state.phase, HOLD_START, "it is back at the start");
    }

    #[test]
    fn moving_to_another_row_restarts_the_marquee() {
        let mut state = MarqueeState::EMPTY;
        let mut now = 0;
        run_until(&mut state, &mut now, HOLD_END);
        assert_eq!(advance(&mut state, 8, now, 40), 0);
        assert_eq!(state.phase, HOLD_START);
    }

    #[test]
    fn a_label_that_now_fits_jumps_back() {
        let mut state = MarqueeState::EMPTY;
        let mut now = 0;
        run_until(&mut state, &mut now, FORWARD);
        now += 500;
        assert!(advance(&mut state, 7, now, 40) < 0, "it was scrolled left");
        assert_eq!(advance(&mut state, 7, now, 0), 0, "and snaps back at once");
        assert_eq!(state.phase, HOLD_START);
    }
}
