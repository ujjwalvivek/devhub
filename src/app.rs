use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui::{self, RichText, Stroke};

use crate::cache;
use crate::config::{Config, RemoteHostConfig};
use crate::discovery::{self, Project, ScanStatus};
use crate::editor;
use crate::ui::theme;
use crate::workspace::{self, FileEntry, SearchHit};

struct ScanOutcome {
    projects: Vec<Project>,
    errors: Vec<String>,
}

#[derive(Default)]
struct OnboardingState {
    local_root: String,
    remote_name: String,
    remote_host: String,
    remote_root: String,
    error: Option<String>,
}

#[derive(Default)]
struct WorkbenchState {
    project_key: Option<String>,
    files: Vec<FileEntry>,
    selected_file: Option<PathBuf>,
    file_content: String,
    file_error: Option<String>,
    search_query: String,
    search_hits: Vec<SearchHit>,
    search_error: Option<String>,
}

impl OnboardingState {
    fn from_config(config: &Config) -> Self {
        let remote = config.remote_hosts.first();
        Self {
            local_root: config
                .scan_dirs
                .first()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            remote_name: remote.map(|host| host.name.clone()).unwrap_or_default(),
            remote_host: remote.map(|host| host.host.clone()).unwrap_or_default(),
            remote_root: remote
                .and_then(|host| host.roots.first().cloned())
                .unwrap_or_default(),
            error: None,
        }
    }
}

pub struct DevHub {
    config: Config,
    projects: Vec<Project>,
    selected: Option<usize>,
    filter: String,
    scan_status: ScanStatus,
    scan_errors: Vec<String>,
    onboarding: OnboardingState,
    workbench: WorkbenchState,
    scan_rx: Option<mpsc::Receiver<ScanOutcome>>,
}

impl DevHub {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        theme::configure(&cc.egui_ctx);

        let should_load_library = config.onboarding_complete && config_has_sources(&config);
        let projects = if should_load_library {
            match cache::load_projects() {
                Ok(projects) => {
                    if !projects.is_empty() {
                        tracing::info!(count = projects.len(), "loaded project cache");
                    }
                    projects
                }
                Err(error) => {
                    tracing::warn!(%error, "failed to load project cache");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let scan_status = if projects.is_empty() {
            ScanStatus::Idle
        } else {
            ScanStatus::Done {
                count: projects.len(),
            }
        };
        let onboarding = OnboardingState::from_config(&config);

        let mut app = Self {
            config,
            projects,
            selected: None,
            filter: String::new(),
            scan_status,
            scan_errors: Vec::new(),
            onboarding,
            workbench: WorkbenchState::default(),
            scan_rx: None,
        };

        if should_load_library {
            app.start_scan(cc.egui_ctx.clone());
        }
        app
    }

    fn start_scan(&mut self, ctx: egui::Context) {
        if !self.has_sources() {
            self.scan_status = ScanStatus::Idle;
            return;
        }

        self.scan_status = ScanStatus::Scanning;

        let dirs = self.config.scan_dirs.clone();
        let max_depth = self.config.max_depth;
        let remote_hosts = self.config.remote_hosts.clone();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        std::thread::spawn(move || {
            tracing::info!("background scan started");
            let mut projects = discovery::scan_directories(&dirs, max_depth);
            let mut errors = Vec::new();

            for remote in remote_hosts {
                tracing::info!(host = %remote.host, "remote scan started");
                match discovery::scan_remote_host(&remote) {
                    Ok(mut remote_projects) => {
                        tracing::info!(
                            host = %remote.host,
                            count = remote_projects.len(),
                            "remote scan complete"
                        );
                        projects.append(&mut remote_projects);
                    }
                    Err(error) => {
                        tracing::warn!(host = %remote.host, %error, "remote scan failed");
                        errors.push(format!("{}: {error}", remote.label()));
                    }
                }
            }

            discovery::sort_projects(&mut projects);
            if let Err(error) = cache::save_projects(&projects) {
                tracing::warn!(%error, "failed to save project cache");
            }

            tracing::info!(
                count = projects.len(),
                errors = errors.len(),
                "scan complete"
            );
            let _ = tx.send(ScanOutcome { projects, errors });
            ctx.request_repaint();
        });
    }

    fn has_sources(&self) -> bool {
        config_has_sources(&self.config)
    }

    fn source_count(&self) -> usize {
        self.config.scan_dirs.len()
            + self
                .config
                .remote_hosts
                .iter()
                .map(|host| host.roots.len())
                .sum::<usize>()
    }

    fn onboarding_active(&self) -> bool {
        !self.config.onboarding_complete || !self.has_sources()
    }

    fn add_local_root(&mut self) {
        let raw = self.onboarding.local_root.trim();
        if raw.is_empty() {
            self.onboarding.error = Some("enter a local root path".into());
            return;
        }

        let path = PathBuf::from(raw);
        if !self
            .config
            .scan_dirs
            .iter()
            .any(|existing| existing == &path)
        {
            self.config.scan_dirs.push(path);
        }
        self.onboarding.local_root.clear();
        self.onboarding.error = None;
    }

    fn add_remote_root(&mut self) {
        let host = normalize_ssh_host(self.onboarding.remote_host.trim());
        let root = self.onboarding.remote_root.trim();

        if host.is_empty() || root.is_empty() {
            self.onboarding.error = Some("enter an ssh host and remote root".into());
            return;
        }

        self.config.remote_hosts.push(RemoteHostConfig {
            name: self.onboarding.remote_name.trim().to_string(),
            host,
            roots: vec![root.to_string()],
            max_depth: self.config.max_depth,
        });
        self.onboarding.remote_name.clear();
        self.onboarding.remote_host.clear();
        self.onboarding.remote_root.clear();
        self.onboarding.error = None;
    }

    fn finish_onboarding(&mut self, ctx: &egui::Context) {
        if !self.has_sources() {
            self.onboarding.error = Some("add at least one source".into());
            return;
        }

        self.config.onboarding_complete = true;
        match self.config.save() {
            Ok(()) => {
                self.projects.clear();
                self.selected = None;
                self.filter.clear();
                self.workbench = WorkbenchState::default();
                self.onboarding.error = None;
                self.start_scan(ctx.clone());
            }
            Err(error) => {
                tracing::warn!(%error, "failed to save onboarding config");
                self.onboarding.error = Some(format!("failed to save config: {error}"));
            }
        }
    }

    fn draw_onboarding(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("setup_top_bar")
            .exact_height(38.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin {
                        left: 14,
                        right: 14,
                        top: 8,
                        bottom: 8,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("devhub").color(theme::ACCENT).size(15.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("setup").color(theme::TEXT_DIM).size(11.0));
                });
            });

        egui::TopBottomPanel::top("setup_sep")
            .exact_height(1.0)
            .frame(egui::Frame::NONE.fill(theme::SEPARATOR))
            .show(ctx, |_ui| {});

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_DARK)
                    .inner_margin(egui::Margin {
                        left: 36,
                        right: 36,
                        top: 32,
                        bottom: 32,
                    }),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(820.0);
                        ui.label(
                            RichText::new("Set up project sources")
                                .color(theme::TEXT_PRIMARY)
                                .size(22.0),
                        );
                        ui.add_space(6.0);
                        ui.label(
                    RichText::new(
                        "Add a local folder or remote SSH root. Scanning starts only after setup.",
                    )
                    .color(theme::TEXT_DIM)
                    .size(12.0),
                );

                        ui.add_space(24.0);

                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.label(
                                    RichText::new("local windows root")
                                        .color(theme::ACCENT)
                                        .size(13.0),
                                );
                                ui.add_space(8.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.onboarding.local_root)
                                        .hint_text(
                                            RichText::new("F:/_Engine").color(theme::TEXT_MUTED),
                                        )
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::Margin::symmetric(8, 6))
                                        .text_color(theme::TEXT_PRIMARY),
                                );
                                ui.add_space(8.0);
                                if ui.button("add local root").clicked() {
                                    self.add_local_root();
                                }
                            });

                            columns[1].vertical(|ui| {
                                ui.label(
                                    RichText::new("remote linux root")
                                        .color(theme::ACCENT)
                                        .size(13.0),
                                );
                                ui.add_space(8.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.onboarding.remote_name)
                                        .hint_text(RichText::new("label").color(theme::TEXT_MUTED))
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::Margin::symmetric(8, 6))
                                        .text_color(theme::TEXT_PRIMARY),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.onboarding.remote_host)
                                        .hint_text(
                                            RichText::new("datacenter").color(theme::TEXT_MUTED),
                                        )
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::Margin::symmetric(8, 6))
                                        .text_color(theme::TEXT_PRIMARY),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.onboarding.remote_root)
                                        .hint_text(
                                            RichText::new("/home/vivi/dev")
                                                .color(theme::TEXT_MUTED),
                                        )
                                        .desired_width(f32::INFINITY)
                                        .margin(egui::Margin::symmetric(8, 6))
                                        .text_color(theme::TEXT_PRIMARY),
                                );
                                ui.add_space(8.0);
                                if ui.button("add remote root").clicked() {
                                    self.add_remote_root();
                                }
                            });
                        });

                        ui.add_space(24.0);
                        ui.separator();
                        ui.add_space(14.0);
                        ui.label(RichText::new("sources").color(theme::TEXT_DIM).size(11.0));
                        ui.add_space(6.0);

                        let mut remove_local = None;
                        for (idx, root) in self.config.scan_dirs.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("local").color(theme::TEXT_MUTED).size(11.0),
                                );
                                ui.label(
                                    RichText::new(root.display().to_string())
                                        .color(theme::TEXT_PRIMARY)
                                        .size(12.0),
                                );
                                if ui.button("remove").clicked() {
                                    remove_local = Some(idx);
                                }
                            });
                        }
                        if let Some(idx) = remove_local {
                            self.config.scan_dirs.remove(idx);
                        }

                        let mut remove_remote = None;
                        for (idx, remote) in self.config.remote_hosts.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(remote.label())
                                        .color(theme::ACCENT)
                                        .size(11.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} {}",
                                        remote.host,
                                        remote.roots.join(", ")
                                    ))
                                    .color(theme::TEXT_PRIMARY)
                                    .size(12.0),
                                );
                                if ui.button("remove").clicked() {
                                    remove_remote = Some(idx);
                                }
                            });
                        }
                        if let Some(idx) = remove_remote {
                            self.config.remote_hosts.remove(idx);
                        }

                        if self.config.scan_dirs.is_empty() && self.config.remote_hosts.is_empty() {
                            ui.label(
                                RichText::new("no sources yet")
                                    .color(theme::TEXT_MUTED)
                                    .size(12.0),
                            );
                        }

                        if let Some(error) = &self.onboarding.error {
                            ui.add_space(12.0);
                            ui.label(RichText::new(error).color(theme::GIT_BADGE).size(12.0));
                        }

                        ui.add_space(24.0);
                        let start = egui::Button::new(
                            RichText::new("start library")
                                .color(theme::BG_DARK)
                                .size(13.0),
                        )
                        .fill(theme::ACCENT)
                        .stroke(Stroke::new(1.0, theme::ACCENT_DIM))
                        .min_size(egui::vec2(168.0, 34.0));

                        if ui.add(start).clicked() {
                            self.finish_onboarding(ctx);
                        }
                    });
            });
    }

    fn select_project(&mut self, idx: usize) {
        if self.selected == Some(idx) {
            return;
        }
        self.selected = Some(idx);
        self.refresh_workbench();
    }

    fn selected_project(&self) -> Option<Project> {
        self.selected
            .and_then(|idx| self.projects.get(idx))
            .cloned()
    }

    fn refresh_workbench(&mut self) {
        let Some(project) = self.selected_project() else {
            self.workbench = WorkbenchState::default();
            return;
        };

        let key = project_key(&project);
        self.workbench = WorkbenchState {
            project_key: Some(key),
            ..WorkbenchState::default()
        };
        match workspace::list_tree(&project, self.config.max_depth + 2) {
            Ok(files) => self.workbench.files = files,
            Err(error) => self.workbench.file_error = Some(error.to_string()),
        }
    }

    fn open_workbench_file(&mut self, project: &Project, path: PathBuf) {
        self.workbench.selected_file = Some(path.clone());
        match workspace::read_file(project, &path) {
            Ok(content) => {
                self.workbench.file_content = content;
                self.workbench.file_error = None;
            }
            Err(error) => {
                self.workbench.file_content.clear();
                self.workbench.file_error = Some(error.to_string());
            }
        }
    }

    fn run_search(&mut self, project: &Project) {
        match workspace::search(project, &self.workbench.search_query) {
            Ok(hits) => {
                self.workbench.search_hits = hits;
                self.workbench.search_error = None;
            }
            Err(error) => {
                self.workbench.search_hits.clear();
                self.workbench.search_error = Some(error.to_string());
            }
        }
    }

    fn draw_main_workspace(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 0.0);
        ui.columns(2, |columns| {
            pane_frame().show(&mut columns[0], |ui| {
                self.draw_project_cards(ui);
            });
            pane_frame().show(&mut columns[1], |ui| {
                self.draw_project_workbench(ui);
            });
        });
    }

    fn draw_project_cards(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "projects");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{}", self.projects.len()))
                        .color(theme::TEXT_DIM)
                        .size(11.0),
                );
            });
        });
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.filter)
                .hint_text(RichText::new("filter projects...").color(theme::TEXT_MUTED))
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6))
                .text_color(theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        let filter = self.filter.to_lowercase();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for idx in 0..self.projects.len() {
                    let project = self.projects[idx].clone();
                    if !filter.is_empty() && !project.search_key.contains(&filter) {
                        continue;
                    }
                    self.draw_project_card(ui, idx, &project);
                    ui.add_space(8.0);
                }
            });
    }

    fn draw_project_card(&mut self, ui: &mut egui::Ui, idx: usize, project: &Project) {
        let is_selected = self.selected == Some(idx);
        let fill = if is_selected {
            theme::BG_CARD_ACTIVE
        } else {
            theme::BG_CARD
        };

        let response = egui::Frame::NONE
            .fill(fill)
            .stroke(Stroke::new(
                1.0,
                if is_selected {
                    theme::ACCENT_DIM
                } else {
                    theme::BORDER
                },
            ))
            .inner_margin(egui::Margin::symmetric(10, 9))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let title = ui.add(
                        egui::Label::new(
                            RichText::new(&project.name)
                                .color(theme::TEXT_PRIMARY)
                                .size(15.0)
                                .strong(),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if title.clicked() {
                        self.select_project(idx);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if project.has_git {
                            pill(ui, "git", theme::GIT_BADGE, theme::BG_FIELD);
                        }
                    });
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    pill(
                        ui,
                        project.project_type.label(),
                        project.project_type.accent(),
                        theme::BG_FIELD,
                    );
                    pill(
                        ui,
                        project.source.label(),
                        if project.source.is_remote() {
                            theme::ACCENT
                        } else {
                            theme::TEXT_DIM
                        },
                        theme::BG_FIELD,
                    );
                    ui.label(
                        RichText::new(project.path.display().to_string())
                            .color(theme::TEXT_MUTED)
                            .size(10.0),
                    );
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let mut editors = self
                        .config
                        .editors
                        .iter()
                        .map(|(key, cfg)| (key.clone(), cfg.clone()))
                        .collect::<Vec<_>>();
                    editors.sort_by(|(left, _), (right, _)| left.cmp(right));
                    for (_, editor_cfg) in editors {
                        if editor::can_open_project(&editor_cfg, project)
                            && action_button(ui, &editor_cfg.name).clicked()
                        {
                            editor::open_project(&editor_cfg, project);
                        }
                    }
                    if action_button(ui, "files").clicked() {
                        self.select_project(idx);
                    }
                    if !project.source.is_remote() && action_button(ui, "folder").clicked() {
                        let _ = open::that(&project.path);
                    }
                });
            })
            .response;

        if response.clicked() {
            self.select_project(idx);
        }
    }

    fn draw_project_workbench(&mut self, ui: &mut egui::Ui) {
        let Some(project) = self.selected_project() else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("select a project")
                        .color(theme::TEXT_MUTED)
                        .size(16.0),
                );
            });
            return;
        };

        let key = project_key(&project);
        if self.workbench.project_key.as_deref() != Some(key.as_str()) {
            self.refresh_workbench();
        }

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&project.name)
                    .color(theme::TEXT_PRIMARY)
                    .size(20.0)
                    .strong(),
            );
            pill(
                ui,
                project.project_type.label(),
                project.project_type.accent(),
                theme::BG_FIELD,
            );
            pill(ui, project.source.label(), theme::TEXT_DIM, theme::BG_FIELD);
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(project.path.display().to_string())
                .color(theme::TEXT_MUTED)
                .size(11.0),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if action_button(ui, "refresh files").clicked() {
                self.refresh_workbench();
            }
            let mut editors = self
                .config
                .editors
                .iter()
                .map(|(key, cfg)| (key.clone(), cfg.clone()))
                .collect::<Vec<_>>();
            editors.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (_, editor_cfg) in editors {
                if editor::can_open_project(&editor_cfg, &project)
                    && action_button(ui, &format!("open {}", editor_cfg.name)).clicked()
                {
                    editor::open_project(&editor_cfg, &project);
                }
            }
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.workbench.search_query)
                    .hint_text(RichText::new("search inside project...").color(theme::TEXT_MUTED))
                    .desired_width((ui.available_width() - 70.0).max(80.0))
                    .margin(egui::Margin::symmetric(8, 6))
                    .text_color(theme::TEXT_PRIMARY),
            );
            let enter =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if primary_action_button(ui, "search").clicked() || enter {
                self.run_search(&project);
            }
        });

        if let Some(error) = &self.workbench.search_error {
            ui.label(RichText::new(error).color(theme::GIT_BADGE).size(11.0));
        }

        ui.add_space(10.0);
        ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);
        ui.columns(2, |columns| {
            tool_frame().show(&mut columns[0], |ui| {
                self.draw_file_tree(ui, &project);
            });
            tool_frame().show(&mut columns[1], |ui| {
                self.draw_file_preview(ui);
            });
        });
    }

    fn draw_file_tree(&mut self, ui: &mut egui::Ui, project: &Project) {
        if !self.workbench.search_hits.is_empty() {
            section_label(ui, &format!("matches {}", self.workbench.search_hits.len()));
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let hits = self.workbench.search_hits.clone();
                    for hit in hits {
                        let label = format!("{}:{} {}", hit.path.display(), hit.line, hit.preview);
                        if ui
                            .selectable_label(
                                false,
                                RichText::new(label).color(theme::TEXT_PRIMARY).size(11.0),
                            )
                            .clicked()
                        {
                            self.open_workbench_file(project, hit.path);
                        }
                    }
                });
            ui.add_space(8.0);
        }

        section_label(ui, &format!("files {}", self.workbench.files.len()));
        if let Some(error) = &self.workbench.file_error {
            ui.label(RichText::new(error).color(theme::GIT_BADGE).size(11.0));
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let entries = self.workbench.files.clone();
                for entry in entries {
                    let indent = "  ".repeat(entry.depth.min(5));
                    let prefix = if entry.is_dir { "[d]" } else { "   " };
                    let label = format!("{indent}{prefix} {}", entry.name);
                    let is_selected = self.workbench.selected_file.as_ref() == Some(&entry.path);
                    let color = if entry.is_dir {
                        theme::TEXT_DIM
                    } else {
                        theme::TEXT_PRIMARY
                    };
                    if ui
                        .selectable_label(is_selected, RichText::new(label).color(color).size(11.0))
                        .clicked()
                        && !entry.is_dir
                    {
                        self.open_workbench_file(project, entry.path);
                    }
                }
            });
    }

    fn draw_file_preview(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "preview");
            if let Some(path) = &self.workbench.selected_file {
                ui.label(
                    RichText::new(path.display().to_string())
                        .color(theme::TEXT_MUTED)
                        .size(10.0),
                );
            }
        });
        ui.add_space(6.0);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.workbench.file_content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(28)
                        .interactive(false),
                );
            });
    }
}

fn project_key(project: &Project) -> String {
    format!("{}:{}", project.source.label(), project.path.display())
}

fn pane_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme::BG_PANEL)
        .stroke(Stroke::new(1.0, theme::SEPARATOR))
        .inner_margin(egui::Margin::same(12))
}

fn tool_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme::BG_CARD)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .inner_margin(egui::Margin::same(10))
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_ascii_uppercase())
            .color(theme::ACCENT)
            .size(11.0)
            .strong(),
    );
}

fn pill(ui: &mut egui::Ui, text: &str, color: egui::Color32, fill: egui::Color32) {
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, color.linear_multiply(0.45)))
        .inner_margin(egui::Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).size(10.0));
        });
}

fn action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(theme::TEXT_PRIMARY).size(11.0))
            .fill(theme::BG_CARD_HOVER)
            .stroke(Stroke::new(1.0, theme::ACCENT_DIM))
            .min_size(egui::vec2(64.0, 28.0)),
    )
}

fn primary_action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(theme::BG_DARK).size(11.0))
            .fill(theme::ACCENT)
            .stroke(Stroke::new(1.0, theme::ACCENT_DIM))
            .min_size(egui::vec2(72.0, 30.0)),
    )
}

fn config_has_sources(config: &Config) -> bool {
    !config.scan_dirs.is_empty()
        || config
            .remote_hosts
            .iter()
            .any(|host| !host.host.trim().is_empty() && !host.roots.is_empty())
}

fn normalize_ssh_host(raw: &str) -> String {
    raw.strip_prefix("ssh ")
        .map(str::trim)
        .unwrap_or(raw)
        .to_string()
}

impl eframe::App for DevHub {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(ref rx) = self.scan_rx
            && let Ok(outcome) = rx.try_recv()
        {
            let count = outcome.projects.len();
            self.projects = outcome.projects;
            self.scan_errors = outcome.errors;
            self.scan_status = ScanStatus::Done { count };
            self.scan_rx = None;
            if self.selected.is_some_and(|idx| idx >= self.projects.len()) {
                self.selected = None;
            }
        }

        if self.onboarding_active() {
            self.draw_onboarding(ctx);
            return;
        }

        egui::TopBottomPanel::top("top_bar")
            .exact_height(42.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin {
                        left: 14,
                        right: 14,
                        top: 6,
                        bottom: 6,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("devhub").color(theme::ACCENT).size(15.0));

                    ui.add_space(12.0);

                    let status_text = match &self.scan_status {
                        ScanStatus::Idle => "idle".into(),
                        ScanStatus::Scanning => "scanning...".into(),
                        ScanStatus::Done { count } if self.scan_errors.is_empty() => {
                            format!("{count} projects, {} sources", self.source_count())
                        }
                        ScanStatus::Done { count } => {
                            format!(
                                "{count} projects, {} sources, {} scan errors",
                                self.source_count(),
                                self.scan_errors.len()
                            )
                        }
                        ScanStatus::Error(e) => format!("error: {e}"),
                    };
                    ui.label(RichText::new(status_text).color(theme::TEXT_DIM).size(11.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if action_button(ui, "scan").clicked()
                            && !matches!(self.scan_status, ScanStatus::Scanning)
                        {
                            self.start_scan(ctx.clone());
                        }

                        if action_button(ui, "sources").clicked() {
                            self.config.onboarding_complete = false;
                            self.scan_rx = None;
                            self.scan_status = ScanStatus::Idle;
                            self.onboarding = OnboardingState::from_config(&self.config);
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(22.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin {
                        left: 14,
                        right: 14,
                        top: 3,
                        bottom: 3,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(idx) = self.selected
                        && let Some(p) = self.projects.get(idx)
                    {
                        ui.label(
                            RichText::new(p.path.display().to_string())
                                .color(theme::TEXT_MUTED)
                                .size(10.0),
                        );
                    }
                });
            });

        let top_sep = egui::TopBottomPanel::top("top_sep")
            .exact_height(1.0)
            .frame(egui::Frame::NONE.fill(theme::SEPARATOR));
        top_sep.show(ctx, |_ui| {});

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_DARK)
                    .inner_margin(egui::Margin {
                        left: 20,
                        right: 20,
                        top: 16,
                        bottom: 16,
                    }),
            )
            .show(ctx, |ui| {
                self.draw_main_workspace(ui);
            });
    }
}
