use egui::{Color32, Context, Stroke, Visuals};

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    // Background colors - warm grays
    visuals.panel_fill = Color32::from_rgb(28, 24, 20);
    visuals.window_fill = Color32::from_rgb(33, 29, 25);
    visuals.extreme_bg_color = Color32::from_rgb(18, 16, 14);

    // Text colors
    visuals.override_text_color = Some(Color32::WHITE);

    // Selection color - orange/yellow tones
    visuals.selection.bg_fill = Color32::from_rgb(180, 100, 40);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(220, 140, 70));

    // Widget colors - warm grays
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(38, 33, 28);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(43, 38, 33);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(53, 47, 40);
    visuals.widgets.active.bg_fill = Color32::from_rgb(63, 56, 48);

    // Accent color (yellow like TUI)
    let accent = Color32::from_rgb(255, 215, 0);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, accent);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, accent);

    ctx.set_visuals(visuals);

    // Set font sizes
    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));

    ctx.set_style(style);
}
