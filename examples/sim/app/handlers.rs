//! 事件处理与每帧更新：文案刷新、组件状态、Toast 补发与运行时间。

use kk_ui::{ItemState, KkUi, Status, TransitionKind};

use super::tables::*;

pub(super) fn update_progress_text(progress: u16, manual: bool) {
    let mut buffer = [0u8; 20];
    let prefix = if manual { "手动 " } else { "自动 " };
    let mut text = [0u8; 20];
    let prefix_len = prefix.len();
    text[..prefix_len].copy_from_slice(prefix.as_bytes());
    let digits = format_u32(progress as u32, "%", &mut buffer);
    let len = prefix_len + digits.len();
    text[prefix_len..len].copy_from_slice(digits.as_bytes());
    PROGRESS_TEXT.set(core::str::from_utf8(&text[..len]).unwrap_or(""));
}

/// 把无符号整数和单位后缀写进缓冲，返回其中的 UTF-8 片段。
pub(super) fn format_u32<'a>(value: u32, unit: &str, buffer: &'a mut [u8; 20]) -> &'a str {
    let mut rest = value;
    let mut digits = [0u8; 10];
    let mut count = 0usize;
    loop {
        digits[count] = b'0' + (rest % 10) as u8;
        count += 1;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    let mut out = 0usize;
    while count > 0 && out + 1 < buffer.len() {
        count -= 1;
        buffer[out] = digits[count];
        out += 1;
    }
    for byte in unit.as_bytes() {
        if out + 1 >= buffer.len() {
            break;
        }
        buffer[out] = *byte;
        out += 1;
    }
    core::str::from_utf8(&buffer[..out]).unwrap_or("")
}

/// 本次演示的加载弹窗持续时长（毫秒）。
const LOADING_MS: u32 = 1200;

fn update_switch_text() {
    SWITCH_TEXT.set(if DEMO_SWITCH.get() { "开" } else { "关" });
}

fn reset_parameters() {
    BRIGHTNESS.set(80);
    ANIMATION_INTERVAL.set(20);
    DEMO_SWITCH.set(true);
    SHOW_ITEM.set(true);
    DISABLE_ITEM.set(false);
    FADE_TRANSITION.set(false);
    LEVEL.set(1);
    CHANNELS.set(0b0101);
    REFRESH_INDEX.set(1);
    let mut buffer = [0u8; 20];
    BRIGHTNESS_TEXT.set(format_u32(BRIGHTNESS.get() as u32, "%", &mut buffer));
    update_switch_text();
    INTERVAL_TEXT.set(format_u32(
        ANIMATION_INTERVAL.get() as u32,
        "ms",
        &mut buffer,
    ));
    update_progress_text(0, false);
}

/// 按「显示条目 / 禁用条目」刷新组件演示页最后一行的状态。
fn apply_component_state(ui: &mut KkUi<'_>) {
    let state = if !SHOW_ITEM.get() {
        ItemState::Hidden
    } else if DISABLE_ITEM.get() {
        ItemState::Disabled
    } else {
        ItemState::Normal
    };
    ui.set_menu_item_state(&ROUTE_COMPONENTS, 3, state);
}

/// 弹 Toast；覆盖层占用时留到下一帧补发。
fn toast(ui: &mut KkUi<'_>, text: &'static str, duration: u32) {
    if ui.show_toast(text, duration) == Status::Busy {
        PENDING_TOAST.set(Some((text, duration)));
    } else {
        PENDING_TOAST.set(None);
    }
}

/// 本屏背光接 GND，没有对比度控制：把「屏幕亮度」标成禁用。
fn mark_brightness_unsupported(ui: &mut KkUi<'_>) {
    BRIGHTNESS_STATE.set("不支持");
    ui.set_menu_item_state(&ROUTE_SETTINGS, 0, ItemState::Disabled);
}

/// 屏幕亮度依赖 OLED 的对比度命令；只能如实上报不支持。
fn report_brightness_unsupported(ui: &mut KkUi<'_>) {
    mark_brightness_unsupported(ui);
    toast(ui, "亮度不支持", 0);
}

/// 演示用：闲置多久报告「变暗」（毫秒）。
const DIM_AFTER_MS: u32 = 10_000;
/// 演示用：闲置多久报告「息屏」（毫秒）。
const SLEEP_AFTER_MS: u32 = 30_000;

/// 初始化应用状态；在 [`KkUi::init`] 之前或之后调用都可以。
pub fn reset(ui: &mut KkUi<'_>, now: u32) {
    reset_parameters();
    for slot in &SETTINGS_STATES {
        slot.set(0);
    }
    for slot in &COMPONENT_STATES {
        slot.set(0);
    }
    mark_brightness_unsupported(ui);
    UPTIME.set("0s");
    APP_STARTED_AT.set(now);
    INFO_UPDATED_AT.set(now);
    APP_NOW.set(now);
    UPDATE_EPOCH.set(0);
    PENDING_TOAST.set(None);
    LOADING_UNTIL.set(0);
    apply_component_state(ui);
    apply_transition(ui);
    // 演示电源阶段：闲置 10 s 变暗、30 s 息屏，监控页每秒报告一次。
    ui.set_power_timeouts(DIM_AFTER_MS, SLEEP_AFTER_MS);
    ui.invalidate();
}

/// 每秒刷新一次运行时间。
pub fn update(ui: &mut KkUi<'_>, now: u32) {
    APP_NOW.set(now);
    let until = LOADING_UNTIL.get();
    if until != 0 && (now.wrapping_sub(until) as i32) >= 0 {
        LOADING_UNTIL.set(0);
        ui.hide_loading();
    }
    UPDATE_EPOCH.set(UPDATE_EPOCH.get().wrapping_add(1));
    if now.wrapping_sub(INFO_UPDATED_AT.get()) >= 1000 {
        INFO_UPDATED_AT.set(now);
        let seconds = now.wrapping_sub(APP_STARTED_AT.get()) / 1000;
        let mut buffer = [0u8; 20];
        UPTIME.set(format_u32(seconds, "s", &mut buffer));
        update_power_text(ui);
        ui.invalidate();
    }
}

/// 把当前开关同步到运行时转场效果。
fn apply_transition(ui: &mut KkUi<'_>) {
    ui.set_transition_kind(if FADE_TRANSITION.get() {
        TransitionKind::Fade
    } else {
        TransitionKind::Slide
    });
}

/// 空闲时长、电源阶段与上一秒变化的页数。
fn update_power_text(ui: &mut KkUi<'_>) {
    use kk_ui::PowerPhase;
    let mut buffer = [0u8; 20];
    let idle = ui.idle_ms() / 1000;
    IDLE_TEXT.set(format_u32(idle, "s", &mut buffer));
    PHASE_TEXT.set(match ui.power_phase() {
        PowerPhase::Awake => "正常",
        PowerPhase::Dim => "变暗",
        PowerPhase::Sleep => "息屏",
    });
    let mut buffer = [0u8; 20];
    let dirty = ui.take_dirty_pages();
    if dirty == u32::MAX {
        DIRTY_TEXT.set("全部");
    } else {
        let pages = ui.frame().pages();
        let mut text = [0u8; 20];
        let mut page_buffer = [0u8; 20];
        let count = format_u32(dirty.count_ones(), "", &mut buffer);
        let slash = format_u32(pages as u32, "", &mut page_buffer);
        let len = count.len() + 1 + slash.len();
        if len <= text.len() {
            text[..count.len()].copy_from_slice(count.as_bytes());
            text[count.len()] = b'/';
            text[count.len() + 1..len].copy_from_slice(slash.as_bytes());
            DIRTY_TEXT.set(core::str::from_utf8(&text[..len]).unwrap_or(""));
        }
    }
}

/// 处理应用事件。
pub fn process_events(ui: &mut KkUi<'_>) {
    if let Some((text, duration)) = PENDING_TOAST.get() {
        toast(ui, text, duration);
    }
    while let Some(event) = ui.poll_event() {
        match event {
            EVENT_RESET_CONFIRMED => {
                reset_parameters();
                apply_component_state(ui);
                ui.invalidate();
                report_brightness_unsupported(ui);
            }
            EVENT_SHOW_MESSAGE => {
                ui.show_message("确定关闭", EVENT_MESSAGE_ACKNOWLEDGED);
            }
            EVENT_SHOW_TOAST => toast(ui, "这是 Toast 提示", 0),
            EVENT_SHOW_LOADING => {
                if ui.show_loading("正在加载") == Status::Ok {
                    LOADING_UNTIL.set(APP_NOW.get().wrapping_add(LOADING_MS));
                }
            }
            EVENT_SWITCH_CHANGED => {
                update_switch_text();
                ui.invalidate();
                toast(ui, "设置已更新", 0);
            }
            EVENT_INTERVAL_CHANGED => {
                let mut buffer = [0u8; 20];
                INTERVAL_TEXT.set(format_u32(
                    ANIMATION_INTERVAL.get() as u32,
                    "ms",
                    &mut buffer,
                ));
                ui.invalidate();
                toast(ui, "设置已更新", 0);
            }
            EVENT_BRIGHTNESS_CHANGED => {
                let mut buffer = [0u8; 20];
                BRIGHTNESS_TEXT.set(format_u32(BRIGHTNESS.get() as u32, "%", &mut buffer));
                ui.invalidate();
                toast(ui, "亮度不支持", 0);
            }
            EVENT_ADVANCED_VISIBILITY_CHANGED | EVENT_DEMO_LOCK_CHANGED => {
                apply_component_state(ui);
                toast(ui, "设置已更新", 0);
            }
            EVENT_ADVANCED_ACTION => toast(ui, "操作已触发", 0),
            EVENT_TRANSITION_CHANGED => {
                apply_transition(ui);
                toast(ui, "转场已切换", 0);
            }
            EVENT_LEVEL_CHANGED | EVENT_CHANNELS_CHANGED | EVENT_REFRESH_CHANGED => {
                toast(ui, "组件已更新", 0)
            }
            _ => {}
        }
    }
    if !BRIGHTNESS_REPORTED.get() {
        BRIGHTNESS_REPORTED.set(true);
        report_brightness_unsupported(ui);
    }
}
