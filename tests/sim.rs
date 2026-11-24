//! 窗口字体来自 eframe 的 `default_fonts` 特性；漏掉它时按钮文字静默变成方块。
#![cfg(feature = "sim")]

#[test]
fn egui_has_default_fonts() {
    let fonts = eframe::egui::FontDefinitions::default();
    assert!(
        !fonts.font_data.is_empty(),
        "eframe default_fonts feature is off"
    );
}
