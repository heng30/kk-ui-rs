//! 按键消抖、长按重复与编码器归一化。

use super::{KEY_DEBOUNCE_MS, KEY_REPEAT_DELAY_MS, KkUi};
use crate::model::{
    Input, InputAction, InputEvent, InputSource, KEY_DOWN, KEY_OK, KEY_UP, PageType,
};
use crate::widgets::dialog;
use crate::widgets::pages;

/// 单个按键的消抖与重复状态。
#[derive(Clone, Copy, Default)]
pub(super) struct KeyState {
    /// 原始电平最近一次变化的时刻。
    pub(super) changed_at: u32,
    /// 本次按下生效的时刻。
    pub(super) pressed_at: u32,
    /// 下一次触发长按重复的时刻。
    pub(super) repeat_at: u32,
    /// 消抖前的原始电平。
    pub(super) candidate: bool,
    /// 消抖后的稳定电平。
    pub(super) stable: bool,
    /// 本次按下是否还未被消费。
    pub(super) armed: bool,
}

/// 长按重复的间隔：按已按住时间分三档。
fn repeat_interval(held: u32) -> u32 {
    if held >= 2000 {
        100
    } else if held >= 1000 {
        160
    } else {
        240
    }
}

impl KkUi<'_> {
    /// 消抖、识别长按重复与编码器，并发出一条归一化输入事件。
    #[allow(clippy::needless_range_loop)]
    pub(super) fn process_input(&mut self, now: u32, input: Input) {
        let mut press = 0u8;
        let mut repeat = 0u8;
        let keys = input.keys & (KEY_UP | KEY_DOWN | KEY_OK);
        if !self.rt.keys_initialized {
            for index in 0..3 {
                let down = keys & (1 << index) != 0;
                let key = &mut self.rt.keys[index];
                key.changed_at = now;
                key.pressed_at = now;
                key.repeat_at = now;
                key.candidate = down;
                key.stable = down;
                key.armed = !down;
            }
            self.rt.keys_initialized = true;
        } else {
            for index in 0..3 {
                let key = &mut self.rt.keys[index];
                let down = keys & (1 << index) != 0;
                if down != key.candidate {
                    key.candidate = down;
                    key.changed_at = now;
                }
                if key.candidate != key.stable
                    && now.wrapping_sub(key.changed_at) >= KEY_DEBOUNCE_MS
                {
                    key.stable = key.candidate;
                    if !key.stable {
                        key.armed = true;
                    } else if key.armed {
                        key.pressed_at = now;
                        key.repeat_at = now.wrapping_add(KEY_REPEAT_DELAY_MS);
                        press |= 1 << index;
                    }
                }
                // 只有上下键支持长按重复。
                if index < 2
                    && key.stable
                    && key.candidate
                    && key.armed
                    && (now.wrapping_sub(key.repeat_at) as i32) >= 0
                {
                    repeat |= 1 << index;
                    key.repeat_at =
                        now.wrapping_add(repeat_interval(now.wrapping_sub(key.pressed_at)));
                }
            }
        }

        let mut stable = 0u8;
        for (index, key) in self.rt.keys.iter().enumerate() {
            if key.stable {
                stable |= 1 << index;
            }
        }
        self.rt.blocked_keys &= stable;
        if self.rt.transition.active {
            self.rt.blocked_keys |= stable;
            return;
        }
        press &= !self.rt.blocked_keys;
        repeat &= !self.rt.blocked_keys;

        let mut event = InputEvent {
            action: InputAction::Up,
            source: InputSource::Press,
            steps: 1,
        };
        if press & KEY_OK != 0 {
            event.action = InputAction::Ok;
            self.dispatch_input(event, now);
            return;
        }
        if input.encoder_delta != 0 {
            let delta = input.encoder_delta as i32;
            event.action = if delta < 0 {
                InputAction::Up
            } else {
                InputAction::Down
            };
            event.source = InputSource::Encoder;
            event.steps = delta.unsigned_abs().min(u16::MAX as u32) as u16;
            self.dispatch_input(event, now);
            return;
        }
        if stable & (KEY_UP | KEY_DOWN) == (KEY_UP | KEY_DOWN) {
            return;
        }
        if (press | repeat) & KEY_UP != 0 {
            event.action = InputAction::Up;
            event.source = if repeat & KEY_UP != 0 {
                InputSource::Repeat
            } else {
                InputSource::Press
            };
            self.dispatch_input(event, now);
        } else if (press | repeat) & KEY_DOWN != 0 {
            event.action = InputAction::Down;
            event.source = if repeat & KEY_DOWN != 0 {
                InputSource::Repeat
            } else {
                InputSource::Press
            };
            self.dispatch_input(event, now);
        }
    }

    /// 按当前页面类型把输入分发下去；有覆盖层时优先给弹窗。
    fn dispatch_input(&mut self, event: InputEvent, now: u32) {
        if dialog::dialog_active(&self.rt) {
            dialog::input(&mut self.rt, event, now);
            return;
        }
        match self.rt.current_page_type() {
            PageType::Home => pages::home_input(&mut self.rt, event, now),
            PageType::Menu => pages::menu_input(&mut self.rt, event, now),
            PageType::Info => pages::info_input(&mut self.rt, event, now),
            PageType::Custom => {
                let page = self.rt.current.page;
                if let Some(desc) = self.custom_page(page) {
                    self.rt.in_callback = true;
                    desc.on_input(&mut self.rt, page, event);
                    self.rt.in_callback = false;
                }
            }
        }
    }
}
