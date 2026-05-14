use eframe::egui::{
    self, Color32, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle, Visuals,
};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::sync::Arc;

pub const OUTER_MARGIN: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    CatppuccinMocha,
    RosePineMoon,
    TokyoNightStorm,
    HorizonBold,
    MonochromeZero,
}

impl ThemeId {
    pub const ALL: [Self; 5] = [
        Self::CatppuccinMocha,
        Self::RosePineMoon,
        Self::TokyoNightStorm,
        Self::HorizonBold,
        Self::MonochromeZero,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "Catppuccin",
            Self::RosePineMoon => "Rose Pine",
            Self::TokyoNightStorm => "Tokyo Night",
            Self::HorizonBold => "Horizon",
            Self::MonochromeZero => "Monochrome",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    System,
    Dark,
    Light,
}

impl AppearanceMode {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    fn is_light(self, ctx: &egui::Context) -> bool {
        match self {
            Self::System => ctx.system_theme() == Some(egui::Theme::Light),
            Self::Dark => false,
            Self::Light => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub bg_dark: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_card_active: Color32,
    pub bg_field: Color32,
    pub text_primary: Color32,
    pub text_dim: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub border: Color32,
    pub git_badge: Color32,
}

const DEFAULT_COLORS: Colors = Colors {
    bg_dark: hex(0x00, 0x00, 0x00),
    bg_panel: hex(0x00, 0x00, 0x00),
    bg_card: hex(0x03, 0x03, 0x03),
    bg_card_hover: hex(0x08, 0x08, 0x08),
    bg_card_active: hex(0x0d, 0x0d, 0x0d),
    bg_field: hex(0x00, 0x00, 0x00),
    text_primary: hex(0xeb, 0xeb, 0xeb),
    text_dim: hex(0x7a, 0x7a, 0x7a),
    text_muted: hex(0x3a, 0x3a, 0x3a),
    accent: hex(0xf0, 0xf0, 0xf0),
    accent_dim: hex(0x1a, 0x1a, 0x1a),
    border: hex(0x06, 0x06, 0x06),
    git_badge: hex(0x99, 0x99, 0x99),
};

thread_local! {
    static ACTIVE_COLORS: Cell<Colors> = const { Cell::new(DEFAULT_COLORS) };
}

pub fn configure(ctx: &egui::Context, theme_id: ThemeId, appearance: AppearanceMode) {
    configure_fonts(ctx);
    let mut style = Style {
        text_styles: text_styles(),
        ..Default::default()
    };
    configure_spacing(&mut style);
    style.visuals = visuals_for(
        apply_colors(ctx, theme_id, appearance),
        appearance.is_light(ctx),
    );
    ctx.set_style(style);
}

pub fn apply(ctx: &egui::Context, theme_id: ThemeId, appearance: AppearanceMode) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = text_styles();
    configure_spacing(&mut style);
    style.visuals = visuals_for(
        apply_colors(ctx, theme_id, appearance),
        appearance.is_light(ctx),
    );
    ctx.set_style(style);
}

pub fn colors() -> Colors {
    ACTIVE_COLORS.with(Cell::get)
}

pub fn bg_dark() -> Color32 {
    colors().bg_dark
}

pub fn bg_panel() -> Color32 {
    colors().bg_panel
}

pub fn bg_card() -> Color32 {
    colors().bg_card
}

pub fn bg_card_hover() -> Color32 {
    colors().bg_card_hover
}

pub fn bg_card_active() -> Color32 {
    colors().bg_card_active
}

pub fn bg_field() -> Color32 {
    colors().bg_field
}

pub fn text_primary() -> Color32 {
    colors().text_primary
}

pub fn text_dim() -> Color32 {
    colors().text_dim
}

pub fn text_muted() -> Color32 {
    colors().text_muted
}

pub fn accent() -> Color32 {
    colors().accent
}

pub fn accent_dim() -> Color32 {
    colors().accent_dim
}

pub fn border() -> Color32 {
    colors().border
}

pub fn git_badge() -> Color32 {
    colors().git_badge
}

fn configure_fonts(ctx: &egui::Context) {
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
}

fn text_styles() -> std::collections::BTreeMap<TextStyle, FontId> {
    [
        (TextStyle::Small, FontId::new(11.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Monospace)),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Monospace)),
    ]
    .into()
}

fn configure_spacing(style: &mut Style) {
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 5.0);
    style.spacing.interact_size = egui::vec2(26.0, 26.0);
    style.spacing.window_margin = egui::Margin::same(OUTER_MARGIN as i8);
}

fn apply_colors(ctx: &egui::Context, theme_id: ThemeId, appearance: AppearanceMode) -> Colors {
    let colors = palette(theme_id, appearance.is_light(ctx));
    ACTIVE_COLORS.with(|active| active.set(colors));
    colors
}

fn visuals_for(colors: Colors, light: bool) -> Visuals {
    let mut visuals = if light {
        Visuals::light()
    } else {
        Visuals::dark()
    };

    visuals.override_text_color = Some(colors.text_primary);
    visuals.panel_fill = colors.bg_dark;
    visuals.window_fill = colors.bg_panel;
    visuals.extreme_bg_color = colors.bg_dark;
    visuals.faint_bg_color = colors.bg_field;

    visuals.widgets.noninteractive.bg_fill = colors.bg_field;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_dim);
    visuals.widgets.noninteractive.corner_radius = Default::default();

    visuals.widgets.inactive.bg_fill = colors.bg_field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_primary);
    visuals.widgets.inactive.corner_radius = Default::default();

    visuals.widgets.hovered.bg_fill = colors.bg_card_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.accent);
    visuals.widgets.hovered.corner_radius = Default::default();

    visuals.widgets.active.bg_fill = colors.bg_card_active;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.accent);
    visuals.widgets.active.corner_radius = Default::default();

    visuals.selection.bg_fill = colors.accent_dim;
    visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    visuals.hyperlink_color = colors.accent;
    visuals.window_stroke = Stroke::new(1.0, colors.border);
    visuals
}

fn palette(theme_id: ThemeId, light: bool) -> Colors {
    match (theme_id, light) {
        (ThemeId::CatppuccinMocha, false) => Colors {
            bg_dark: hex(0x1e, 0x1e, 0x2e),
            bg_panel: hex(0x27, 0x28, 0x3a),
            bg_card: hex(0x31, 0x32, 0x44),
            bg_card_hover: hex(0x37, 0x3d, 0x55),
            bg_card_active: hex(0x3b, 0x45, 0x61),
            bg_field: hex(0x18, 0x18, 0x27),
            text_primary: hex(0xcd, 0xd6, 0xf4),
            text_dim: hex(0x93, 0x9a, 0xbb),
            text_muted: hex(0x6f, 0x75, 0x90),
            accent: hex(0x89, 0xb4, 0xfa),
            accent_dim: hex(0x52, 0x6c, 0x9a),
            border: hex(0x45, 0x46, 0x5b),
            git_badge: hex(0xfa, 0xb3, 0x87),
        },
        (ThemeId::RosePineMoon, false) => Colors {
            bg_dark: hex(0x23, 0x21, 0x36),
            bg_panel: hex(0x26, 0x24, 0x3a),
            bg_card: hex(0x2a, 0x27, 0x3f),
            bg_card_hover: hex(0x34, 0x2f, 0x4c),
            bg_card_active: hex(0x3b, 0x33, 0x50),
            bg_field: hex(0x1d, 0x1b, 0x2b),
            text_primary: hex(0xe0, 0xde, 0xf4),
            text_dim: hex(0xa9, 0xa4, 0xc6),
            text_muted: hex(0x78, 0x73, 0x92),
            accent: hex(0xea, 0x9a, 0x97),
            accent_dim: hex(0x8e, 0x5e, 0x69),
            border: hex(0x48, 0x43, 0x61),
            git_badge: hex(0xf6, 0xc1, 0x77),
        },
        (ThemeId::TokyoNightStorm, false) => Colors {
            bg_dark: hex(0x24, 0x28, 0x3b),
            bg_panel: hex(0x29, 0x2e, 0x43),
            bg_card: hex(0x2f, 0x35, 0x49),
            bg_card_hover: hex(0x36, 0x3e, 0x57),
            bg_card_active: hex(0x3a, 0x45, 0x62),
            bg_field: hex(0x1f, 0x23, 0x33),
            text_primary: hex(0xa9, 0xb1, 0xd6),
            text_dim: hex(0x7f, 0x88, 0xb0),
            text_muted: hex(0x5e, 0x66, 0x87),
            accent: hex(0x7a, 0xa2, 0xf7),
            accent_dim: hex(0x4b, 0x61, 0x93),
            border: hex(0x43, 0x4a, 0x63),
            git_badge: hex(0xbb, 0x9a, 0xf7),
        },
        (ThemeId::HorizonBold, false) => Colors {
            bg_dark: hex(0x1c, 0x1e, 0x26),
            bg_panel: hex(0x20, 0x22, 0x2c),
            bg_card: hex(0x23, 0x25, 0x30),
            bg_card_hover: hex(0x2b, 0x2d, 0x3a),
            bg_card_active: hex(0x35, 0x2d, 0x39),
            bg_field: hex(0x16, 0x18, 0x1f),
            text_primary: hex(0xd5, 0xd8, 0xda),
            text_dim: hex(0x9b, 0xa0, 0xa6),
            text_muted: hex(0x6f, 0x74, 0x7d),
            accent: hex(0xe9, 0x56, 0x78),
            accent_dim: hex(0x8a, 0x36, 0x4b),
            border: hex(0x43, 0x45, 0x50),
            git_badge: hex(0xfa, 0xb7, 0x95),
        },
        (ThemeId::MonochromeZero, false) => Colors {
            bg_dark: hex(0x00, 0x00, 0x00),
            bg_panel: hex(0x00, 0x00, 0x00),
            bg_card: hex(0x00, 0x00, 0x00),
            bg_card_hover: hex(0x04, 0x04, 0x04),
            bg_card_active: hex(0x08, 0x08, 0x08),
            bg_field: hex(0x00, 0x00, 0x00),
            text_primary: hex(0xff, 0xff, 0xff),
            text_dim: hex(0xbb, 0xbb, 0xbb),
            text_muted: hex(0x66, 0x66, 0x66),
            accent: hex(0xff, 0xff, 0xff),
            accent_dim: hex(0x08, 0x08, 0x08),
            border: hex(0x00, 0x00, 0x00),
            git_badge: hex(0xcc, 0xcc, 0xcc),
        },
        (ThemeId::CatppuccinMocha, true) => light_colors(
            hex(0xef, 0xf1, 0xf5),
            hex(0xff, 0xff, 0xff),
            hex(0x1e, 0x66, 0xf5),
            hex(0xfe, 0x64, 0x0b),
            hex(0x4c, 0x4f, 0x69),
        ),
        (ThemeId::RosePineMoon, true) => light_colors(
            hex(0xfa, 0xf4, 0xed),
            hex(0xff, 0xfa, 0xf3),
            hex(0xd7, 0x82, 0x7e),
            hex(0xea, 0x9d, 0x34),
            hex(0x57, 0x52, 0x79),
        ),
        (ThemeId::TokyoNightStorm, true) => light_colors(
            hex(0xe1, 0xe2, 0xe7),
            hex(0xf4, 0xf5, 0xfa),
            hex(0x2e, 0x7d, 0xe9),
            hex(0x98, 0x52, 0xe0),
            hex(0x34, 0x3b, 0x58),
        ),
        (ThemeId::HorizonBold, true) => light_colors(
            hex(0xf3, 0xf0, 0xf2),
            hex(0xff, 0xff, 0xff),
            hex(0xd4, 0x38, 0x5f),
            hex(0xc7, 0x75, 0x47),
            hex(0x2a, 0x2d, 0x35),
        ),
        (ThemeId::MonochromeZero, true) => light_colors(
            hex(0xff, 0xff, 0xff),
            hex(0xf6, 0xf6, 0xf6),
            hex(0x00, 0x00, 0x00),
            hex(0x4d, 0x4d, 0x4d),
            hex(0x11, 0x11, 0x11),
        ),
    }
}

fn light_colors(
    bg: Color32,
    card: Color32,
    accent: Color32,
    git_badge: Color32,
    text: Color32,
) -> Colors {
    Colors {
        bg_dark: bg,
        bg_panel: mix(bg, card, 0.45),
        bg_card: card,
        bg_card_hover: mix(card, accent, 0.10),
        bg_card_active: mix(card, accent, 0.18),
        bg_field: mix(bg, card, 0.22),
        text_primary: text,
        text_dim: mix(text, bg, 0.36),
        text_muted: mix(text, bg, 0.58),
        accent,
        accent_dim: mix(accent, bg, 0.58),
        border: mix(text, bg, 0.75),
        git_badge,
    }
}

const fn hex(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn mix(a: Color32, b: Color32, b_weight: f32) -> Color32 {
    let a_weight = 1.0 - b_weight;
    Color32::from_rgb(
        ((a.r() as f32 * a_weight) + (b.r() as f32 * b_weight)).round() as u8,
        ((a.g() as f32 * a_weight) + (b.g() as f32 * b_weight)).round() as u8,
        ((a.b() as f32 * a_weight) + (b.b() as f32 * b_weight)).round() as u8,
    )
}
