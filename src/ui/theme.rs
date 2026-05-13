use eframe::egui::{
    self, Color32, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle, Visuals,
};
use std::sync::Arc;

pub const BG_DARK: Color32 = Color32::from_rgb(10, 11, 14);
pub const BG_PANEL: Color32 = Color32::from_rgb(16, 17, 22);
pub const BG_CARD: Color32 = Color32::from_rgb(24, 25, 31);
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(30, 32, 39);
pub const BG_CARD_ACTIVE: Color32 = Color32::from_rgb(28, 36, 41);
pub const BG_FIELD: Color32 = Color32::from_rgb(12, 13, 17);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(222, 225, 232);
pub const TEXT_DIM: Color32 = Color32::from_rgb(132, 137, 150);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(82, 87, 100);
pub const ACCENT: Color32 = Color32::from_rgb(77, 221, 204);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(39, 126, 118);
pub const BORDER: Color32 = Color32::from_rgb(42, 45, 56);
// pub const BORDER_HOVER: Color32 = Color32::from_rgb(55, 55, 75);
pub const SEPARATOR: Color32 = Color32::from_rgb(31, 34, 43);
pub const GIT_BADGE: Color32 = Color32::from_rgb(217, 153, 74);
pub const OUTER_MARGIN: f32 = 12.0;

pub fn configure(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "monaspace".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/MonaspaceNeon-Regular.otf"
        ))),
    );

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "monaspace".into());
    }
    ctx.set_fonts(fonts);

    let mut style = Style {
        text_styles: [
            (TextStyle::Small, FontId::new(11.0, FontFamily::Monospace)),
            (TextStyle::Body, FontId::new(13.0, FontFamily::Monospace)),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
            (TextStyle::Button, FontId::new(12.0, FontFamily::Monospace)),
            (TextStyle::Heading, FontId::new(18.0, FontFamily::Monospace)),
        ]
        .into(),
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.interact_size = egui::vec2(28.0, 28.0);
    style.spacing.window_margin = egui::Margin::same(OUTER_MARGIN as i8);

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_DARK;
    visuals.faint_bg_color = BG_FIELD;

    visuals.widgets.noninteractive.bg_fill = BG_FIELD;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    visuals.widgets.noninteractive.corner_radius = Default::default();

    visuals.widgets.inactive.bg_fill = BG_FIELD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = Default::default();

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.corner_radius = Default::default();

    visuals.widgets.active.bg_fill = BG_CARD_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.corner_radius = Default::default();

    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.window_stroke = Stroke::new(1.0, BORDER);

    style.visuals = visuals;
    ctx.set_style(style);
}
