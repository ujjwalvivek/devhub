use eframe::egui::{self, RichText, Stroke, StrokeKind, Vec2};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::discovery::Project;
use crate::editor;
use crate::ui::theme::*;

pub fn draw_detail(ui: &mut egui::Ui, project: &Project, config: &Config) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            ui.label(RichText::new(&project.name).color(TEXT_PRIMARY).size(22.0));
            ui.add_space(2.0);

            let badge_color = project.project_type.accent();
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(50.0, 18.0), egui::Sense::hover());
                ui.painter().rect(
                    rect,
                    0u8,
                    badge_color.linear_multiply(0.15),
                    Stroke::new(1.0, badge_color.linear_multiply(0.4)),
                    StrokeKind::Outside,
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    project.project_type.label(),
                    egui::FontId::new(10.0, egui::FontFamily::Monospace),
                    badge_color,
                );

                if project.has_git {
                    ui.add_space(4.0);
                    let (git_rect, _) =
                        ui.allocate_exact_size(Vec2::new(30.0, 18.0), egui::Sense::hover());
                    ui.painter().rect(
                        git_rect,
                        0u8,
                        GIT_BADGE.linear_multiply(0.15),
                        Stroke::new(1.0, GIT_BADGE.linear_multiply(0.4)),
                        StrokeKind::Outside,
                    );
                    ui.painter().text(
                        git_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "git",
                        egui::FontId::new(10.0, egui::FontFamily::Monospace),
                        GIT_BADGE,
                    );
                }
            });

            ui.add_space(16.0);

            draw_info_row(ui, "source", project.source.label());
            if let Some(host) = project.source.host() {
                draw_info_row(ui, "ssh host", host);
            }
            draw_info_row(ui, "path", &project.path.display().to_string());

            if let Some(ref remote) = project.git_remote {
                draw_info_row(ui, "remote", remote);
            }

            let markers = project.markers_found.join(", ");
            draw_info_row(ui, "markers", &markers);

            if let Some(modified) = project.last_modified {
                if let Some(now) = unix_now() {
                    let secs = now.saturating_sub(modified);
                    let label = if secs < 60 {
                        "just now".into()
                    } else if secs < 3600 {
                        format!("{}m ago", secs / 60)
                    } else if secs < 86400 {
                        format!("{}h ago", secs / 3600)
                    } else {
                        format!("{}d ago", secs / 86400)
                    };
                    draw_info_row(ui, "modified", &label);
                }
            }

            ui.add_space(24.0);

            ui.label(RichText::new("open in").color(TEXT_DIM).size(11.0));
            ui.add_space(4.0);

            let mut editors = config.editors.iter().collect::<Vec<_>>();
            editors.sort_by(|(left, _), (right, _)| left.cmp(right));

            for (key, editor_cfg) in editors {
                if !editor::can_open_project(editor_cfg, project) {
                    continue;
                }

                let is_default = key == &config.default_editor;
                let label = if is_default {
                    format!("> {}", editor_cfg.name)
                } else {
                    format!("  {}", editor_cfg.name)
                };

                let btn = egui::Button::new(
                    RichText::new(label)
                        .color(if is_default { ACCENT } else { TEXT_PRIMARY })
                        .size(13.0),
                )
                .min_size(Vec2::new(ui.available_width().min(220.0), 32.0))
                .fill(BG_CARD)
                .stroke(Stroke::new(
                    1.0,
                    if is_default { ACCENT_DIM } else { BORDER },
                ));

                if ui.add(btn).clicked() {
                    editor::open_project(editor_cfg, project);
                }
            }

            if !project.source.is_remote() {
                ui.add_space(16.0);

                let explorer_btn =
                    egui::Button::new(RichText::new("open folder").color(TEXT_DIM).size(12.0))
                        .min_size(Vec2::new(ui.available_width().min(220.0), 28.0))
                        .fill(BG_CARD)
                        .stroke(Stroke::new(1.0, BORDER));

                if ui.add(explorer_btn).clicked() {
                    let _ = open::that(&project.path);
                }
            }
        });
}

fn draw_info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{:<10}", label))
                .color(TEXT_DIM)
                .size(12.0),
        );
        ui.label(RichText::new(value).color(TEXT_PRIMARY).size(12.0));
    });
    ui.add_space(2.0);
}

pub fn draw_empty(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label(
            RichText::new("select a project")
                .color(TEXT_MUTED)
                .size(16.0),
        );
    });
}

fn unix_now() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}
