use eframe::egui::{self, Color32, Layout, RichText, Sense, Stroke, Vec2};

use crate::discovery::Project;
use crate::ui::theme::*;

pub fn draw_sidebar(
    ui: &mut egui::Ui,
    projects: &[Project],
    selected: Option<usize>,
    filter: &mut String,
) -> Option<usize> {
    let mut new_selection = selected;

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("projects").color(ACCENT).size(14.0));
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{}", projects.len()))
                        .color(TEXT_DIM)
                        .size(11.0),
                );
            });
        });

        ui.add_space(4.0);

        let filter_response = ui.add(
            egui::TextEdit::singleline(filter)
                .hint_text(RichText::new("filter...").color(TEXT_MUTED))
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6))
                .text_color(TEXT_PRIMARY),
        );

        if ui
            .ctx()
            .input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl)
        {
            filter_response.request_focus();
        }

        ui.add_space(6.0);

        let rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
            ],
            Stroke::new(1.0, SEPARATOR),
        );
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let filter_lower = filter.to_lowercase();

                for (i, project) in projects.iter().enumerate() {
                    if !filter_lower.is_empty() && !project.search_key.contains(&filter_lower) {
                        continue;
                    }

                    let is_selected = selected == Some(i);
                    if draw_project_row(ui, project, is_selected) {
                        new_selection = Some(i);
                    }
                }
            });
    });

    new_selection
}

fn draw_project_row(ui: &mut egui::Ui, project: &Project, is_selected: bool) -> bool {
    let desired_size = Vec2::new(ui.available_width(), 40.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if !ui.is_rect_visible(rect) {
        return false;
    }

    let painter = ui.painter();
    let hovered = response.hovered();

    let bg = if is_selected {
        BG_CARD_ACTIVE
    } else if hovered {
        BG_CARD_HOVER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 0u8, bg);

    if is_selected {
        let bar = egui::Rect::from_min_size(rect.left_top(), Vec2::new(2.0, rect.height()));
        painter.rect_filled(bar, 0u8, ACCENT);
    }

    let badge_text = project.project_type.label();
    let badge_color = project.project_type.accent();
    let badge_pos = egui::pos2(rect.left() + 10.0, rect.center().y - 5.0);
    painter.text(
        badge_pos,
        egui::Align2::LEFT_CENTER,
        badge_text,
        egui::FontId::new(9.0, egui::FontFamily::Monospace),
        badge_color,
    );

    let source_pos = egui::pos2(rect.left() + 52.0, rect.center().y - 5.0);
    let source_color = if project.source.is_remote() {
        ACCENT
    } else {
        TEXT_MUTED
    };
    painter.text(
        source_pos,
        egui::Align2::LEFT_CENTER,
        project.source.label(),
        egui::FontId::new(9.0, egui::FontFamily::Monospace),
        source_color,
    );

    let name_pos = egui::pos2(rect.left() + 106.0, rect.center().y - 5.0);
    let name_color = if is_selected || hovered {
        TEXT_PRIMARY
    } else {
        Color32::from_rgb(180, 180, 195)
    };
    painter.text(
        name_pos,
        egui::Align2::LEFT_CENTER,
        &project.name,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
        name_color,
    );

    if project.has_git {
        let git_pos = egui::pos2(rect.right() - 10.0, rect.center().y - 5.0);
        painter.text(
            git_pos,
            egui::Align2::RIGHT_CENTER,
            "git",
            egui::FontId::new(9.0, egui::FontFamily::Monospace),
            GIT_BADGE,
        );
    }

    let sep_y = rect.bottom() - 0.5;
    painter.line_segment(
        [
            egui::pos2(rect.left() + 8.0, sep_y),
            egui::pos2(rect.right() - 8.0, sep_y),
        ],
        Stroke::new(0.5, SEPARATOR),
    );

    response.clicked()
}
