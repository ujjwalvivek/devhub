use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, mpsc};

use eframe::egui::{self, RichText, Stroke, StrokeKind};

use crate::cache;
use crate::config::{Config, RemoteHostConfig};
use crate::discovery::{self, Project, ScanStatus};
use crate::editor;
use crate::ui::theme;
use crate::workspace::{self, DirEntry, FileEntry, SearchHit};

const ECHOPOINT_BASE: &str = "https://echopoint.ujjwalvivek.com";
const MONO_BADGE_PARAMS: &str =
    "bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4";
const MONO_CARD_PARAMS: &str = "bg=0b0b0b&border=555555&borderWidth=4&rx=0&px=12&py=10&textColor=e8e8e8&accentColor=cfcfcf&lineColor=555555&positiveColor=cfcfcf&negativeColor=8a8a8a";
const TRACKED_GITHUB_REPOS: &[&str] = &[
    "portfolio",
    "journey",
    "synclippy",
    "requiem",
    "dino-blink",
    "echopoint",
    "substrate",
    "baremetal",
    "devhub",
    "ujjwalvivek",
    "thereckoning",
    "greedysnek",
    "unitycoordinationframework",
];

struct ScanOutcome {
    projects: Vec<Project>,
    errors: Vec<String>,
}

#[derive(Default)]
struct WorkbenchState {
    project_key: Option<String>,
    files: Vec<FileEntry>,
    expanded_dirs: HashSet<PathBuf>,
    selected_file: Option<PathBuf>,
    file_content: String,
    file_error: Option<String>,
    search_query: String,
    search_hits: Vec<SearchHit>,
    search_error: Option<String>,
}

#[derive(Default)]
enum PickerState {
    #[default]
    Inactive,
    Local {
        current: PathBuf,
        entries: Vec<DirEntry>,
    },
    RemoteHosts,
    BrowseRemote {
        host_idx: usize,
        current: String,
        entries: Vec<DirEntry>,
    },
}

type SvgImageEntry = Result<Arc<egui::ColorImage>, String>;

#[derive(Default)]
struct EchopointSvgLoader {
    cache: egui::mutex::Mutex<HashMap<(Cow<'static, str>, egui::load::SizeHint), SvgImageEntry>>,
}

impl egui::load::ImageLoader for EchopointSvgLoader {
    fn id(&self) -> &str {
        "devhub::EchopointSvgLoader"
    }

    fn load(
        &self,
        ctx: &egui::Context,
        uri: &str,
        size_hint: egui::load::SizeHint,
    ) -> egui::load::ImageLoadResult {
        if !is_echopoint_svg_uri(uri) {
            return Err(egui::load::LoadError::NotSupported);
        }

        let mut actual_hint = size_hint;
        if let Some(h_str) = uri
            .split("&egui_height=")
            .nth(1)
            .and_then(|s| s.split('&').next())
        {
            if let Ok(h) = h_str.parse::<u32>() {
                actual_hint = egui::load::SizeHint::Height(h);
            }
        } else if let Some(c_str) = uri
            .split("&egui_width=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            && let Ok(w) = c_str.parse::<u32>()
        {
            actual_hint = egui::load::SizeHint::Width(w);
        }

        let mut cache = self.cache.lock();
        if let Some(entry) = cache.get(&(Cow::Borrowed(uri), actual_hint)).cloned() {
            return match entry {
                Ok(image) => Ok(egui::load::ImagePoll::Ready { image }),
                Err(error) => Err(egui::load::LoadError::Loading(error)),
            };
        }

        match ctx.try_load_bytes(uri) {
            Ok(egui::load::BytesPoll::Ready { bytes, .. }) => {
                let image = render_echopoint_svg(bytes.as_ref(), Some(actual_hint)).map(Arc::new);
                cache.insert((Cow::Owned(uri.to_owned()), actual_hint), image.clone());
                match image {
                    Ok(image) => Ok(egui::load::ImagePoll::Ready { image }),
                    Err(error) => Err(egui::load::LoadError::Loading(error)),
                }
            }
            Ok(egui::load::BytesPoll::Pending { size }) => {
                Ok(egui::load::ImagePoll::Pending { size })
            }
            Err(error) => Err(error),
        }
    }

    fn forget(&self, uri: &str) {
        self.cache
            .lock()
            .retain(|(cached_uri, _), _| cached_uri != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .map(|entry| match entry {
                Ok(image) => image.pixels.len() * std::mem::size_of::<egui::Color32>(),
                Err(error) => error.len(),
            })
            .sum()
    }
}

fn is_echopoint_svg_uri(uri: &str) -> bool {
    uri.starts_with(ECHOPOINT_BASE) && uri.contains("/svg/")
}

fn render_echopoint_svg(
    svg_bytes: &[u8],
    size_hint: Option<egui::load::SizeHint>,
) -> Result<egui::ColorImage, String> {
    use resvg::tiny_skia::{IntSize, Pixmap};
    use resvg::usvg::{Options, Tree, TreeParsing, TreeTextToPath};

    let options = Options::default();
    let mut tree = Tree::from_data(svg_bytes, &options).map_err(|error| error.to_string())?;
    tree.convert_text(echopoint_fontdb());

    let mut size = tree.size.to_int_size();
    match size_hint {
        None => {}
        Some(egui::load::SizeHint::Size(width, height)) => {
            size = IntSize::from_wh(width, height)
                .ok_or_else(|| format!("failed to scale svg to {width}x{height}"))?;
        }
        Some(egui::load::SizeHint::Height(height)) => {
            size = size
                .scale_to_height(height)
                .ok_or_else(|| format!("failed to scale svg to height {height}"))?;
        }
        Some(egui::load::SizeHint::Width(width)) => {
            size = size
                .scale_to_width(width)
                .ok_or_else(|| format!("failed to scale svg to width {width}"))?;
        }
        Some(egui::load::SizeHint::Scale(scale)) => {
            let scale = scale.into_inner();
            size = size
                .scale_by(scale)
                .ok_or_else(|| format!("failed to scale svg by {scale}"))?;
        }
    }

    let (width, height) = (size.width(), size.height());
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| format!("failed to create svg pixmap {width}x{height}"))?;

    tree.size = size.to_size();
    resvg::Tree::from_usvg(&tree).render(Default::default(), &mut pixmap.as_mut());

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        pixmap.data(),
    ))
}

fn echopoint_fontdb() -> &'static resvg::usvg::fontdb::Database {
    static FONTDB: OnceLock<resvg::usvg::fontdb::Database> = OnceLock::new();
    FONTDB.get_or_init(|| {
        let mut database = resvg::usvg::fontdb::Database::new();
        database.load_system_fonts();
        database.set_sans_serif_family("Arial");
        database.set_monospace_family("Consolas");
        database
    })
}

pub struct DevHub {
    config: Config,
    projects: Vec<Project>,
    selected: Option<usize>,
    filter: String,
    scan_status: ScanStatus,
    scan_errors: Vec<String>,
    show_sources: bool,
    picker_state: PickerState,
    picker_committed: bool,
    picker_error: Option<String>,
    add_host_name: String,
    add_host_host: String,
    workbench: WorkbenchState,
    scan_rx: Option<mpsc::Receiver<ScanOutcome>>,
    applied_system_theme: Option<egui::Theme>,
}

impl DevHub {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        theme::configure(&cc.egui_ctx, config.theme, config.appearance);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx
            .add_image_loader(Arc::new(EchopointSvgLoader::default()));

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
        let has_projects = !projects.is_empty();
        let mut app = Self {
            config,
            projects,
            selected: if has_projects { Some(0) } else { None },
            filter: String::new(),
            scan_status,
            scan_errors: Vec::new(),
            show_sources: !has_projects,
            picker_state: PickerState::Inactive,
            picker_committed: false,
            picker_error: None,
            add_host_name: String::new(),
            add_host_host: String::new(),
            workbench: WorkbenchState::default(),
            scan_rx: None,
            applied_system_theme: cc.egui_ctx.system_theme(),
        };

        if has_projects {
            app.refresh_workbench();
        }
        app.sync_system_theme(&cc.egui_ctx);

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

    fn draw_theme_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let old_theme = self.config.theme;
        let old_appearance = self.config.appearance;

        let next_appearance = {
            let all = theme::AppearanceMode::ALL;
            let pos = all
                .iter()
                .position(|m| *m == self.config.appearance)
                .unwrap_or(0);
            all[(pos + 1) % all.len()]
        };
        let app_label = self.config.appearance.label();
        let btn = egui::Button::new(
            RichText::new(app_label)
                .color(theme::text_primary())
                .size(11.0),
        )
        .fill(theme::bg_card_hover())
        .min_size(egui::vec2(52.0, 18.0));
        if ui.add(btn).clicked() {
            self.config.appearance = next_appearance;
        }

        ui.add_space(4.0);

        let next_theme = {
            let all = theme::ThemeId::ALL;
            let pos = all
                .iter()
                .position(|t| *t == self.config.theme)
                .unwrap_or(0);
            all[(pos + 1) % all.len()]
        };
        let btn = egui::Button::new(
            RichText::new(self.config.theme.label())
                .color(theme::text_primary())
                .size(11.0),
        )
        .fill(theme::bg_card_hover())
        .min_size(egui::vec2(80.0, 18.0));
        if ui.add(btn).clicked() {
            self.config.theme = next_theme;
        }

        if old_theme != self.config.theme || old_appearance != self.config.appearance {
            theme::apply(ctx, self.config.theme, self.config.appearance);
            self.applied_system_theme = ctx.system_theme();
            if let Err(error) = self.config.save() {
                tracing::warn!(%error, "failed to save theme config");
            }
        }
    }

    fn sync_system_theme(&mut self, ctx: &egui::Context) {
        if self.config.appearance != theme::AppearanceMode::System {
            self.applied_system_theme = ctx.system_theme();
            return;
        }

        let system_theme = ctx.system_theme();
        if self.applied_system_theme != system_theme {
            theme::apply(ctx, self.config.theme, self.config.appearance);
            self.applied_system_theme = system_theme;
        }
    }

    fn open_local_picker(&mut self) {
        let start = self
            .config
            .scan_dirs
            .first()
            .cloned()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("/"));
        let entries = workspace::list_local_subdirs(&start).unwrap_or_default();
        self.picker_state = PickerState::Local {
            current: start,
            entries,
        };
        self.picker_error = None;
    }

    fn add_picked_local_dir(&mut self, path: PathBuf, ctx: &egui::Context) {
        if !self.config.scan_dirs.iter().any(|d| d == &path) {
            self.config.scan_dirs.push(path);
        }
        self.commit_sources(ctx);
    }

    fn add_picked_remote_dir(&mut self, host_idx: usize, root: String, ctx: &egui::Context) {
        if let Some(host) = self.config.remote_hosts.get_mut(host_idx)
            && !host.roots.iter().any(|r| r == &root)
        {
            host.roots.push(root);
        }
        self.commit_sources(ctx);
    }

    fn add_new_remote_host(&mut self) {
        let host = normalize_ssh_host(self.add_host_host.trim());
        if host.is_empty() {
            self.picker_error = Some("enter a host address".into());
            return;
        }
        if !self.config.remote_hosts.iter().any(|h| h.host == host) {
            self.config.remote_hosts.push(RemoteHostConfig {
                name: self.add_host_name.trim().to_string(),
                host,
                roots: Vec::new(),
                max_depth: self.config.max_depth,
            });
        }
        self.add_host_name.clear();
        self.add_host_host.clear();
        self.picker_error = None;
        self.picker_state = PickerState::RemoteHosts;
    }

    fn commit_sources(&mut self, ctx: &egui::Context) {
        self.config.onboarding_complete = true;
        if let Err(error) = self.config.save() {
            tracing::warn!(%error, "failed to save config after adding source");
            self.picker_error = Some(format!("failed to save: {error}"));
            return;
        }
        self.projects.clear();
        self.selected = None;
        self.filter.clear();
        self.workbench = WorkbenchState::default();
        self.picker_state = PickerState::Inactive;
        self.picker_committed = true;
        self.start_scan(ctx.clone());
    }

    fn draw_sources_overlay(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        ui.painter()
            .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(180));

        let margin = 48.0;
        let inner = egui::Rect::from_min_size(
            egui::pos2(screen.left() + margin, screen.top() + margin - 8.0),
            egui::vec2(
                screen.width() - 2.0 * margin,
                screen.height() - 2.0 * margin,
            ),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
            egui::Frame::NONE
                .fill(theme::bg_dark())
                .stroke(egui::Stroke::new(1.0, theme::text_muted()))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_min_width(inner.width());
                    ui.set_min_height(inner.height());

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Sources").color(theme::accent()).size(16.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_size = egui::vec2(24.0, 20.0);
                        let (close_rect, close_resp) =
                            ui.allocate_exact_size(close_size, egui::Sense::click());
                        let dim = theme::text_dim();
                        if close_resp.hovered() {
                            ui.painter()
                                .rect_filled(close_rect, 0.0, theme::bg_card_hover());
                        }
                        let c = close_rect.center();
                        let d = 5.0;
                        ui.painter().line_segment(
                            [egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)],
                            egui::Stroke::new(1.5, dim),
                        );
                        ui.painter().line_segment(
                            [egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)],
                            egui::Stroke::new(1.5, dim),
                        );
                        if close_resp.clicked() {
                            self.show_sources = false;
                        }

                        ui.add_space(8.0);
                        let scanning = matches!(self.scan_status, ScanStatus::Scanning);
                        let btn = egui::Button::new(
                            RichText::new(if scanning { "scanning..." } else { "scan" })
                                .color(if scanning { theme::text_dim() } else { theme::text_primary() })
                                .size(11.0),
                        )
                        .fill(theme::bg_card_hover())
                        .min_size(egui::vec2(86.0, 20.0));
                        if ui.add(btn).clicked() && !scanning {
                            self.start_scan(ctx.clone());
                        }

                        let source_count = self.config.scan_dirs.len()
                            + self.config.remote_hosts.iter().map(|h| h.roots.len()).sum::<usize>();
                        let proj_count = match &self.scan_status {
                            ScanStatus::Done { count } => *count,
                            _ => self.projects.len(),
                        };
                        ui.label(
                            RichText::new(format!("{proj_count} projects, {source_count} sources"))
                                .color(theme::text_dim())
                                .size(11.0),
                        );
                    });
                });

                    ui.add_space(12.0);

                    egui::ScrollArea::vertical()
                        .id_salt("sources_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_max_width(inner.width() - 24.0);

                            self.picker_committed = false;
                            let mut picker = std::mem::take(&mut self.picker_state);
                            match &mut picker {
                                PickerState::Inactive => self.draw_onboarding_cards(ui, ctx),
                                PickerState::Local { current, entries } => {
                                    let cur_ref = &mut *current;
                                    let ent_ref = &mut *entries;
                                    self.draw_local_picker(ui, ctx, cur_ref, ent_ref);
                                }
                                PickerState::RemoteHosts => {
                                    self.draw_remote_hosts(ui, ctx);
                                }
                                PickerState::BrowseRemote {
                                    host_idx,
                                    current,
                                    entries,
                                } => {
                                    let h = *host_idx;
                                    let cur_ref = &mut *current;
                                    let ent_ref = &mut *entries;
                                    self.draw_remote_dir_picker(ui, ctx, h, cur_ref, ent_ref);
                                }
                            }
                            if !self.picker_committed && !matches!(picker, PickerState::Inactive) {
                                self.picker_state = picker;
                            }
                        });
                });
        });
    }

    fn draw_onboarding_cards(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.label(
            RichText::new("Link a local folder or remote SSH root.")
                .color(theme::text_dim())
                .size(12.0),
        );

        ui.add_space(20.0);

        let col_spacing = 12.0;
        let card_w = (ui.available_width() - col_spacing) * 0.5;
        ui.horizontal(|ui| {
            let local_card = egui::Frame::NONE
                .fill(theme::bg_card())
                .stroke(Stroke::new(1.0, theme::bg_card_hover()))
                .inner_margin(egui::Margin::same(14));
            ui.allocate_ui_with_layout(
                egui::vec2(card_w, 72.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    local_card.show(ui, |ui| {
                        ui.label(
                            RichText::new("+ Link a local project")
                                .color(theme::accent())
                                .size(13.0),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Browse your filesystem for a project folder")
                                .color(theme::text_dim())
                                .size(11.0),
                        );
                        let resp =
                            ui.interact(ui.min_rect(), ui.next_auto_id(), egui::Sense::click());
                        if resp.clicked() {
                            self.open_local_picker();
                        }
                    });
                },
            );
            ui.add_space(col_spacing);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 72.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::Frame::NONE
                        .fill(theme::bg_card())
                        .stroke(Stroke::new(1.0, theme::bg_card_hover()))
                        .inner_margin(egui::Margin::same(14))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("+ Link a remote project")
                                    .color(theme::accent())
                                    .size(13.0),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Connect to an SSH host and pick a remote folder")
                                    .color(theme::text_dim())
                                    .size(11.0),
                            );
                            let resp =
                                ui.interact(ui.min_rect(), ui.next_auto_id(), egui::Sense::click());
                            if resp.clicked() {
                                self.picker_state = PickerState::RemoteHosts;
                                self.picker_error = None;
                            }
                        });
                },
            );
        });

        ui.add_space(20.0);
        section_label(ui, "sources");
        ui.add_space(6.0);

        let has_sources = !self.config.scan_dirs.is_empty() || !self.config.remote_hosts.is_empty();

        if has_sources {
            let mut remove_local: Option<usize> = None;
            for (idx, root) in self.config.scan_dirs.iter().enumerate() {
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("local").color(theme::text_muted()).size(11.0));
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new(root.display().to_string())
                                    .color(theme::text_primary())
                                    .size(11.0),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if action_button(ui, "remove").clicked() {
                                        remove_local = Some(idx);
                                    }
                                },
                            );
                        });
                    });
            }
            if let Some(idx) = remove_local {
                self.config.scan_dirs.remove(idx);
            }

            let mut remove_remote_root: Option<(usize, usize)> = None;
            for (idx, remote) in self.config.remote_hosts.iter().enumerate() {
                for (ridx, root) in remote.roots.iter().enumerate() {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("remote")
                                        .color(theme::text_muted())
                                        .size(11.0),
                                );
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(format!("{}  {}", remote.label(), root))
                                        .color(theme::text_primary())
                                        .size(11.0),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if action_button(ui, "remove").clicked() {
                                            remove_remote_root = Some((idx, ridx));
                                        }
                                    },
                                );
                            });
                        });
                }
            }
            if let Some((idx, ridx)) = remove_remote_root {
                if let Some(host) = self.config.remote_hosts.get_mut(idx) {
                    host.roots.remove(ridx);
                }
                self.config.remote_hosts.retain(|h| !h.roots.is_empty());
            }
        }

        if let Some(error) = &self.picker_error {
            ui.add_space(12.0);
            ui.label(RichText::new(error).color(theme::git_badge()).size(12.0));
        }
    }

    fn draw_local_picker(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        current: &mut PathBuf,
        entries: &mut Vec<DirEntry>,
    ) {
        ui.horizontal(|ui| {
            if action_button(ui, "< back").clicked() {
                self.picker_state = PickerState::Inactive;
                self.picker_committed = true;
                self.picker_error = None;
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(current.display().to_string())
                    .color(theme::text_dim())
                    .size(11.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_action_button(ui, "Link this folder").clicked() {
                    self.add_picked_local_dir(current.clone(), ctx);
                }
            });
        });

        ui.add_space(8.0);

        let row_h = 22.0;
        let list_h = ui.available_height().max(100.0);
        let is_at_root = current.parent().is_none();
        egui::ScrollArea::vertical()
            .id_salt("local_picker")
            .max_height(list_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (rrect, rresp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_h),
                    egui::Sense::click(),
                );
                ui.painter().text(
                    egui::pos2(rrect.left() + 4.0, rrect.center().y),
                    egui::Align2::LEFT_CENTER,
                    if is_at_root { "[Drives]" } else { ".." },
                    egui::FontId::proportional(11.0),
                    theme::text_dim(),
                );
                if rresp.clicked() {
                    if let Some(parent) = current.parent().map(|p| p.to_path_buf()) {
                        *current = parent;
                        *entries = workspace::list_local_subdirs(current).unwrap_or_default();
                    } else {
                        *entries = workspace::list_drives();
                    }
                }

                let cloned = entries.clone();
                let at_drives = is_at_root
                    && cloned
                        .iter()
                        .any(|e| e.name.len() == 2 && e.name.ends_with(':'));
                for entry in &cloned {
                    let (rrect, rresp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_h),
                        egui::Sense::click(),
                    );
                    if rresp.hovered() {
                        ui.painter().rect_filled(rrect, 0.0, theme::bg_card_hover());
                    }
                    ui.painter().text(
                        egui::pos2(rrect.left() + 4.0, rrect.center().y),
                        egui::Align2::LEFT_CENTER,
                        if at_drives {
                            format!("{}  {}", entry.name, entry.path.display())
                        } else {
                            entry.name.clone()
                        },
                        egui::FontId::proportional(11.0),
                        theme::text_primary(),
                    );
                    if rresp.clicked() {
                        *current = entry.path.clone();
                        *entries = workspace::list_local_subdirs(current).unwrap_or_default();
                    }
                }
            });
    }

    fn draw_remote_hosts(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            if action_button(ui, "< back").clicked() {
                self.picker_state = PickerState::Inactive;
                self.picker_committed = true;
                self.picker_error = None;
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("Configured hosts")
                    .color(theme::text_dim())
                    .size(11.0),
            );
        });

        ui.add_space(12.0);

        let row_h = 28.0;
        let hosts = self.config.remote_hosts.clone();
        for (idx, remote) in hosts.iter().enumerate() {
            let (rrect, rresp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_h),
                egui::Sense::click(),
            );
            if rresp.hovered() {
                ui.painter().rect_filled(rrect, 0.0, theme::bg_card_hover());
            }
            ui.painter().text(
                egui::pos2(rrect.left() + 4.0, rrect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{} ({})", remote.label(), remote.host),
                egui::FontId::proportional(12.0),
                theme::text_primary(),
            );
            if rresp.clicked() {
                let start = remote
                    .roots
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "/home".into());
                let entries =
                    workspace::list_remote_subdirs(&remote.host, &start).unwrap_or_default();
                self.picker_state = PickerState::BrowseRemote {
                    host_idx: idx,
                    current: start,
                    entries,
                };
                self.picker_committed = true;
                self.picker_error = None;
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(
            RichText::new("Add a new host")
                .color(theme::accent())
                .size(12.0),
        );
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.add_host_name)
                .hint_text(RichText::new("label (optional)").color(theme::text_muted()))
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6))
                .text_color(theme::text_primary()),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.add_host_host)
                .hint_text(RichText::new("host (e.g. user@hostname)").color(theme::text_muted()))
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6))
                .text_color(theme::text_primary()),
        );
        ui.add_space(6.0);
        if primary_action_button(ui, "add host").clicked() {
            self.add_new_remote_host();
        }

        if let Some(error) = &self.picker_error {
            ui.add_space(8.0);
            ui.label(RichText::new(error).color(theme::git_badge()).size(12.0));
        }
    }

    fn draw_remote_dir_picker(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        host_idx: usize,
        current: &mut String,
        entries: &mut Vec<DirEntry>,
    ) {
        let host_label = self
            .config
            .remote_hosts
            .get(host_idx)
            .map(|h| h.label().to_string())
            .unwrap_or_default();

        ui.horizontal(|ui| {
            if action_button(ui, "< back").clicked() {
                self.picker_state = PickerState::RemoteHosts;
                self.picker_committed = true;
                self.picker_error = None;
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("{} {}", host_label, current))
                    .color(theme::text_dim())
                    .size(11.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_action_button(ui, "Link this folder").clicked() {
                    self.add_picked_remote_dir(host_idx, current.clone(), ctx);
                }
            });
        });

        ui.add_space(8.0);

        let row_h = 22.0;
        egui::ScrollArea::vertical()
            .id_salt("remote_picker")
            .max_height(ui.available_height().max(100.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(parent) = parent_remote_path(current) {
                    let (rrect, rresp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_h),
                        egui::Sense::click(),
                    );
                    ui.painter().text(
                        egui::pos2(rrect.left() + 4.0, rrect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "..",
                        egui::FontId::proportional(11.0),
                        theme::text_dim(),
                    );
                    if rresp.clicked() {
                        *current = parent;
                        let host = self.config.remote_hosts[host_idx].host.clone();
                        *entries =
                            workspace::list_remote_subdirs(&host, current).unwrap_or_default();
                    }
                }

                let current_host = self
                    .config
                    .remote_hosts
                    .get(host_idx)
                    .map(|h| h.host.clone())
                    .unwrap_or_default();
                let cloned = entries.clone();
                for entry in &cloned {
                    let (rrect, rresp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_h),
                        egui::Sense::click(),
                    );
                    if rresp.hovered() {
                        ui.painter().rect_filled(rrect, 0.0, theme::bg_card_hover());
                    }
                    ui.painter().text(
                        egui::pos2(rrect.left() + 4.0, rrect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &entry.name,
                        egui::FontId::proportional(11.0),
                        theme::text_primary(),
                    );
                    if rresp.clicked() {
                        *current = entry.path.to_string_lossy().to_string();
                        *entries = workspace::list_remote_subdirs(&current_host, current)
                            .unwrap_or_default();
                    }
                }
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
            Ok(files) => {
                self.workbench.expanded_dirs = files
                    .iter()
                    .filter(|e| e.is_dir)
                    .map(|e| e.path.clone())
                    .collect();
                self.workbench.files = files;
            }
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
        let pane_body_height = (ui.available_height() - 0.0).max(0.0);
        let spacing = 8.0;

        let total_w = ui.available_width();
        let right_pad = 12.0;
        let left_w = (total_w * 0.30).floor();
        let mid_w = (total_w * 0.20).floor();
        let right_w = total_w - left_w - mid_w - spacing * 2.0 - right_pad;

        let top = ui.cursor().top();

        let lr = egui::Rect::from_min_size(
            egui::pos2(ui.cursor().left(), top),
            egui::vec2(left_w, pane_body_height),
        );
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(lr).layout(*ui.layout()),
            |ui| {
                pane_frame().show(ui, |ui| {
                    ui.set_height(pane_body_height);
                    self.draw_project_cards(ui);
                });
            },
        );

        let mr = egui::Rect::from_min_size(
            egui::pos2(lr.right() + spacing, top),
            egui::vec2(mid_w, pane_body_height),
        );
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(mr).layout(*ui.layout()),
            |ui| {
                pane_frame().show(ui, |ui| {
                    ui.set_height(pane_body_height);
                    self.draw_project_stats(ui);
                });
            },
        );

        let rr = egui::Rect::from_min_size(
            egui::pos2(mr.right() + spacing, top),
            egui::vec2(right_w, pane_body_height),
        );
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(rr).layout(*ui.layout()),
            |ui| {
                pane_frame().show(ui, |ui| {
                    ui.set_height(pane_body_height);
                    self.draw_project_editor(ui);
                });
            },
        );

        ui.advance_cursor_after_rect(rr);
    }

    fn draw_project_cards(&mut self, ui: &mut egui::Ui) {
        let filter = self.filter.to_lowercase();

        egui::ScrollArea::vertical()
            .id_salt("project_cards")
            .max_height(ui.available_height())
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);
                for idx in 0..self.projects.len() {
                    let project = self.projects[idx].clone();
                    if !filter.is_empty() && !project.search_key.contains(&filter) {
                        continue;
                    }
                    self.draw_project_card(ui, idx, &project);
                    ui.add_space(6.0);
                }
            });
    }

    fn draw_project_card(&mut self, ui: &mut egui::Ui, idx: usize, project: &Project) {
        let is_selected = self.selected == Some(idx);
        let fill = if is_selected {
            theme::bg_card_active()
        } else {
            theme::bg_card()
        };

        let stroke_color = if is_selected {
            theme::accent_dim()
        } else {
            theme::border()
        };

        let card_frame = egui::Frame::NONE
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(14, 12))
            .stroke(Stroke::new(1.0, stroke_color));

        let response = card_frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

                    let title = ui.add(
                        egui::Label::new(
                            RichText::new(&project.name)
                                .color(theme::text_primary())
                                .size(16.0)
                                .strong(),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if title.clicked() {
                        self.select_project(idx);
                    }

                    pill(
                        ui,
                        project.project_type.label(),
                        project.project_type.accent(),
                        theme::bg_field(),
                    );
                    pill(
                        ui,
                        project.source.label(),
                        if project.source.is_remote() {
                            theme::accent()
                        } else {
                            theme::text_dim()
                        },
                        theme::bg_field(),
                    );

                    if project.has_git {
                        pill(ui, "git", theme::git_badge(), theme::bg_field());
                    }
                });

                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(project.path.display().to_string())
                            .color(theme::text_muted())
                            .size(11.0),
                    )
                    .truncate(),
                );

                if let Some(alias) = telemetry_alias(project) {
                    ui.add_space(8.0);
                    draw_card_telemetry(ui, &alias);
                }

                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
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
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        if response.clicked() {
            self.select_project(idx);
        }
    }

    fn draw_project_stats(&mut self, ui: &mut egui::Ui) {
        let Some(project) = self.selected_project() else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("select a project")
                        .color(theme::text_muted())
                        .size(16.0),
                );
            });
            return;
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&project.name)
                    .color(theme::text_primary())
                    .size(16.0)
                    .strong(),
            );
            pill(
                ui,
                project.project_type.label(),
                project.project_type.accent(),
                theme::bg_field(),
            );
            pill(
                ui,
                project.source.label(),
                theme::text_dim(),
                theme::bg_field(),
            );
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(project.path.display().to_string())
                .color(theme::text_muted())
                .size(11.0),
        );
        ui.add_space(12.0);

        if let Some(alias) = telemetry_alias(&project) {
            draw_workbench_telemetry(ui, &alias);
        }
    }

    fn draw_project_editor(&mut self, ui: &mut egui::Ui) {
        let Some(project) = self.selected_project() else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("select a project")
                        .color(theme::text_muted())
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

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.workbench.search_query)
                    .hint_text(RichText::new("search inside project...").color(theme::text_muted()))
                    .desired_width((ui.available_width() - 84.0).max(80.0))
                    .margin(egui::Margin::symmetric(8, 6))
                    .text_color(theme::text_primary()),
            );
            let enter =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if primary_action_button(ui, "search").clicked() || enter {
                self.run_search(&project);
            }
            ui.add_space(4.0);
        });

        if let Some(error) = &self.workbench.search_error {
            ui.label(RichText::new(error).color(theme::git_badge()).size(11.0));
        }

        ui.add_space(6.0);
        let tool_height = ui.available_height();
        if tool_height <= 1.0 {
            return;
        }
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
        let col_w = ui.available_width();
        let tree_w = (col_w - 6.0) * 0.30;
        let prev_w = col_w - tree_w - 6.0;
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(tree_w, tool_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    tool_frame().show(ui, |ui| {
                        ui.set_height((tool_height - 22.0).max(0.0));
                        self.draw_file_tree(ui, &project);
                    });
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(prev_w, tool_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    tool_frame().show(ui, |ui| {
                        ui.set_height((tool_height - 22.0).max(0.0));
                        self.draw_file_preview(ui);
                    });
                },
            );
        });
    }

    fn draw_file_tree(&mut self, ui: &mut egui::Ui, project: &Project) {
        if !self.workbench.search_hits.is_empty() {
            section_label(ui, &format!("matches {}", self.workbench.search_hits.len()));
            let hits_height = (ui.available_height() * 0.32)
                .clamp(72.0, 160.0)
                .min(ui.available_height());
            egui::ScrollArea::vertical()
                .id_salt("search_hits")
                .max_height(hits_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let hits = self.workbench.search_hits.clone();
                    for hit in hits {
                        let label = format!("{}:{} {}", hit.path.display(), hit.line, hit.preview);
                        if ui
                            .selectable_label(
                                false,
                                RichText::new(label).color(theme::text_primary()).size(11.0),
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
            ui.label(RichText::new(error).color(theme::git_badge()).size(11.0));
        }

        egui::ScrollArea::vertical()
            .id_salt("file_tree")
            .max_height(ui.available_height())
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let entries = self.workbench.files.clone();
                let mut skip_depth: Option<usize> = None;
                for entry in &entries {
                    if let Some(skip) = skip_depth {
                        if entry.depth > skip {
                            continue;
                        }
                        skip_depth = None;
                    }

                    let row_h = 20.0;
                    let is_dir = entry.is_dir;
                    let color = if is_dir {
                        theme::text_dim()
                    } else {
                        theme::text_primary()
                    };
                    let is_selected = self.workbench.selected_file.as_ref() == Some(&entry.path);
                    let indent_px = entry.depth as f32 * 14.0;

                    let (row_rect, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_h),
                        egui::Sense::click(),
                    );

                    if is_selected {
                        ui.painter()
                            .rect_filled(row_rect, 0.0, theme::bg_card_hover());
                    }

                    let cx = row_rect.left() + 4.0 + indent_px;
                    let cy = row_rect.center().y;

                    if is_dir {
                        let expanded = self.workbench.expanded_dirs.contains(&entry.path);
                        let arrow = if expanded { "v" } else { ">" };
                        ui.painter().text(
                            egui::pos2(cx, cy),
                            egui::Align2::CENTER_CENTER,
                            arrow,
                            egui::FontId::proportional(10.0),
                            theme::text_dim(),
                        );

                        let ir = egui::Rect::from_min_size(
                            egui::pos2(cx + 10.0, cy - 5.0),
                            egui::vec2(12.0, 10.0),
                        );
                        let body = egui::Rect::from_min_max(
                            egui::pos2(ir.left(), ir.top() + 3.0),
                            egui::pos2(ir.right(), ir.bottom()),
                        );
                        ui.painter().rect_filled(body, 2.0, theme::bg_card_hover());
                        ui.painter().rect_stroke(
                            body,
                            2.0,
                            egui::Stroke::new(1.0, color),
                            StrokeKind::Inside,
                        );
                        let tab = egui::Rect::from_min_max(
                            egui::pos2(ir.left(), ir.top()),
                            egui::pos2(ir.left() + 5.0, ir.top() + 3.0),
                        );
                        ui.painter().rect_filled(tab, 1.0, theme::bg_card_hover());
                        ui.painter().rect_stroke(
                            tab,
                            1.0,
                            egui::Stroke::new(1.0, color),
                            StrokeKind::Inside,
                        );

                        ui.painter().text(
                            egui::pos2(cx + 26.0, cy),
                            egui::Align2::LEFT_CENTER,
                            &entry.name,
                            egui::FontId::proportional(11.0),
                            color,
                        );

                        if resp.clicked() {
                            if expanded {
                                self.workbench.expanded_dirs.remove(&entry.path);
                                skip_depth = Some(entry.depth);
                            } else {
                                self.workbench.expanded_dirs.insert(entry.path.clone());
                            }
                        } else if !expanded {
                            skip_depth = Some(entry.depth);
                        }
                    } else {
                        let ir = egui::Rect::from_min_size(
                            egui::pos2(cx + 10.0, cy - 5.5),
                            egui::vec2(9.0, 11.0),
                        );
                        let body = egui::Rect::from_min_max(
                            egui::pos2(ir.left(), ir.top()),
                            egui::pos2(ir.right(), ir.bottom()),
                        );
                        ui.painter().rect_filled(body, 1.5, theme::bg_card_hover());
                        ui.painter().rect_stroke(
                            body,
                            1.5,
                            egui::Stroke::new(1.0, color),
                            StrokeKind::Inside,
                        );
                        let lx = ir.left() + 2.0;
                        let ly = ir.top() + 3.0;
                        ui.painter().line_segment(
                            [egui::pos2(lx, ly), egui::pos2(lx + 3.0, ly)],
                            egui::Stroke::new(1.0, color),
                        );
                        ui.painter().line_segment(
                            [egui::pos2(lx, ly + 3.0), egui::pos2(lx + 2.0, ly + 3.0)],
                            egui::Stroke::new(1.0, color),
                        );

                        ui.painter().text(
                            egui::pos2(cx + 23.0, cy),
                            egui::Align2::LEFT_CENTER,
                            &entry.name,
                            egui::FontId::proportional(11.0),
                            color,
                        );

                        if resp.clicked() {
                            self.open_workbench_file(project, entry.path.clone());
                        }
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
                        .color(theme::text_muted())
                        .size(10.0),
                );
            }
        });
        ui.add_space(6.0);

        egui::ScrollArea::both()
            .id_salt("file_preview")
            .max_width(ui.available_width())
            .max_height(ui.available_height())
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(&self.workbench.file_content)
                            .color(theme::text_primary())
                            .monospace()
                            .size(11.0),
                    )
                    .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
    }

    fn draw_titlebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let btn_size = egui::vec2(18.0, 18.0);
            let (rect, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());

            let cx = rect.center().x;
            let cy = rect.center().y;
            let stripe_w = 14.0;
            let spacing = 4.0;
            let start_y = cy - spacing;
            let color = theme::text_dim();
            for i in 0..3 {
                ui.painter().line_segment(
                    [
                        egui::pos2(cx - stripe_w / 2.0, start_y + i as f32 * spacing),
                        egui::pos2(cx + stripe_w / 2.0, start_y + i as f32 * spacing),
                    ],
                    egui::Stroke::new(1.5, color),
                );
            }

            if resp.clicked() {
                self.show_sources = !self.show_sources;
                if self.show_sources {
                    self.picker_state = PickerState::Inactive;
                    self.picker_error = None;
                }
            }

            let brand_font = egui::FontId::proportional(14.0);
            let brand_w = ctx.fonts(|f| {
                let g = f.layout_no_wrap(
                    "devhub".to_owned(),
                    brand_font.clone(),
                    egui::Color32::WHITE,
                );
                g.size().x
            });
            let brand_h = 18.0;
            let (brand_rect, brand_resp) =
                ui.allocate_exact_size(egui::vec2(brand_w, brand_h), egui::Sense::click_and_drag());
            ui.painter().text(
                brand_rect.center() + egui::vec2(0.0, 2.0),
                egui::Align2::CENTER_CENTER,
                "devhub",
                brand_font,
                theme::accent(),
            );
            if brand_resp.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if brand_resp.double_clicked() {
                let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            ui.add_space(6.0);
            let sep_font = egui::FontId::proportional(11.0);
            let sep_w = ctx.fonts(|f| {
                let g = f.layout_no_wrap(">".to_owned(), sep_font.clone(), egui::Color32::WHITE);
                g.size().x
            });
            let (sep_rect, _) =
                ui.allocate_exact_size(egui::vec2(sep_w + 4.0, 18.0), egui::Sense::hover());
            ui.painter().text(
                sep_rect.center() + egui::vec2(0.0, 2.0),
                egui::Align2::CENTER_CENTER,
                ">",
                sep_font,
                theme::text_muted(),
            );
            ui.add_space(2.0);

            let (rect, _) = ui.allocate_exact_size(egui::vec2(160.0, 20.0), egui::Sense::hover());
            let edit_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.top() + 2.0),
                egui::vec2(160.0, 18.0),
            );
            ui.put(
                edit_rect,
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text(RichText::new("search projects...").color(theme::text_muted()))
                    .desired_width(160.0)
                    .margin(egui::Margin {
                        left: 4,
                        right: 4,
                        top: 2,
                        bottom: 2,
                    })
                    .text_color(theme::text_primary()),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.draw_window_controls(ui, ctx);
                ui.add_space(4.0);
                self.draw_theme_controls(ui, ctx);

                let remaining = ui.available_width();
                if remaining > 10.0 {
                    let (id, rect) =
                        ui.allocate_space(egui::vec2(remaining, ui.available_height()));
                    let drag = ui.interact(rect, id, egui::Sense::click_and_drag());
                    if drag.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                }
            });
        });
    }

    fn draw_window_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        let dim = theme::text_dim();
        let btn_size = egui::vec2(26.0, 18.0);

        let (_, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_filled(resp.rect, 0.0, egui::Color32::from_rgb(196, 43, 28));
        }
        let c = resp.rect.center();
        let d = 4.0;
        ui.painter().line_segment(
            [egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)],
            egui::Stroke::new(1.5, dim),
        );
        ui.painter().line_segment(
            [egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)],
            egui::Stroke::new(1.5, dim),
        );
        if resp.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let (_, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_filled(resp.rect, 0.0, theme::bg_card_hover());
        }
        let sq = egui::Rect::from_center_size(resp.rect.center(), egui::vec2(8.0, 8.0));
        ui.painter()
            .rect_stroke(sq, 0.0, egui::Stroke::new(1.5, dim), StrokeKind::Outside);
        if resp.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        }

        let (_, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_filled(resp.rect, 0.0, theme::bg_card_hover());
        }
        ui.painter().line_segment(
            [
                egui::pos2(resp.rect.center().x - 5.0, resp.rect.center().y),
                egui::pos2(resp.rect.center().x + 5.0, resp.rect.center().y),
            ],
            egui::Stroke::new(1.5, dim),
        );
        if resp.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }
}

fn project_key(project: &Project) -> String {
    format!("{}:{}", project.source.label(), project.path.display())
}

fn pane_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme::bg_panel())
        .inner_margin(egui::Margin::same(8))
}

fn tool_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme::bg_card())
        .inner_margin(egui::Margin::same(6))
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_ascii_uppercase())
            .color(theme::accent())
            .size(11.0)
            .strong(),
    );
}

fn pill(ui: &mut egui::Ui, text: &str, color: egui::Color32, fill: egui::Color32) {
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, color.linear_multiply(0.45)))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).size(10.0));
        });
}

fn draw_card_telemetry(ui: &mut egui::Ui, alias: &str) {
    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
    ui.horizontal(|ui| {
        let h = 24.0;
        telemetry_badge(ui, badge_uri("stars", alias, Some("github")), h, "stars");
        telemetry_badge(
            ui,
            badge_uri("updated", alias, Some("github")),
            h,
            "updated",
        );
        telemetry_badge(ui, badge_uri("health", alias, Some("github")), h, "health");
    });
}

fn draw_workbench_telemetry(ui: &mut egui::Ui, alias: &str) {
    section_label(ui, "echopoint");
    ui.add_space(6.0);

    let card_width = ui.available_width();
    let card_width_px = card_width.round() as u32;

    ui.spacing_mut().item_spacing = egui::vec2(0.0, 12.0);
    egui::ScrollArea::vertical()
        .id_salt("stats_telemetry")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            telemetry_image(
                ui,
                project_uri(alias, card_width_px),
                card_width,
                "project telemetry",
            );
            telemetry_image(
                ui,
                commits_uri(alias, card_width_px),
                card_width,
                "recent commits",
            );
            telemetry_image(
                ui,
                langs_uri(alias, card_width_px),
                card_width,
                "language mj^",
            );
            telemetry_image(
                ui,
                releases_uri(alias, card_width_px),
                card_width,
                "releases",
            );
        });
}

fn telemetry_badge(ui: &mut egui::Ui, uri: String, height: f32, alt: &str) {
    let response = ui
        .add(
            egui::Image::from_uri(uri.clone())
                .max_height(height)
                .alt_text(alt)
                .sense(egui::Sense::click()),
        )
        .on_hover_text("open endpoint");

    if response.clicked() {
        let _ = open::that(uri);
    }
}

fn telemetry_image(ui: &mut egui::Ui, uri: String, _width: f32, alt: &str) {
    let response = ui
        .add(
            egui::Image::from_uri(uri.clone())
                .fit_to_original_size(1.0)
                .alt_text(alt)
                .sense(egui::Sense::click()),
        )
        .on_hover_text("open endpoint");

    if response.clicked() {
        let _ = open::that(uri);
    }
}

fn badge_uri(kind: &str, alias: &str, logo: Option<&str>) -> String {
    let logo = logo.map(|logo| format!("&logo={logo}")).unwrap_or_default();
    format!(
        "{ECHOPOINT_BASE}/svg/badges/{kind}?repo={alias}{logo}&{MONO_BADGE_PARAMS}&egui_height=24"
    )
}

fn project_uri(alias: &str, width: u32) -> String {
    format!(
        "{ECHOPOINT_BASE}/svg/project?repo={alias}&width={width}&{MONO_CARD_PARAMS}&egui_width={width}"
    )
}

fn commits_uri(alias: &str, width: u32) -> String {
    format!(
        "{ECHOPOINT_BASE}/svg/commits?repo={alias}&limit=3&width={width}&{MONO_CARD_PARAMS}&egui_width={width}"
    )
}

fn releases_uri(alias: &str, width: u32) -> String {
    format!(
        "{ECHOPOINT_BASE}/svg/releases?repo={alias}&limit=3&width={width}&{MONO_CARD_PARAMS}&egui_width={width}"
    )
}

fn langs_uri(alias: &str, width: u32) -> String {
    format!(
        "{ECHOPOINT_BASE}/svg/langs?repo={alias}&limit=6&width={width}&height=8&{MONO_CARD_PARAMS}&pctColor=a6a6a6&color1=e8e8e8&color2=c0c0c0&color3=969696&color4=6d6d6d&color5=464646&egui_width={width}"
    )
}

fn telemetry_alias(project: &Project) -> Option<String> {
    project
        .git_remote
        .as_deref()
        .and_then(github_remote_alias)
        .or_else(|| tracked_repo_alias(&project.name))
}

fn github_remote_alias(remote: &str) -> Option<String> {
    let remote = remote.trim().trim_end_matches(".git");
    let path = if let Some(path) = remote.strip_prefix("git@github.com:") {
        path
    } else if let Some((_, path)) = remote.split_once("github.com/") {
        path
    } else {
        return None;
    };

    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.eq_ignore_ascii_case("ujjwalvivek") {
        tracked_repo_alias(repo)
    } else {
        None
    }
}

fn tracked_repo_alias(raw: &str) -> Option<String> {
    let normalized = raw.trim().trim_end_matches(".git");
    TRACKED_GITHUB_REPOS
        .iter()
        .find(|alias| alias.eq_ignore_ascii_case(normalized))
        .map(|alias| (*alias).to_string())
}

fn action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(theme::text_primary()).size(11.0))
            .fill(theme::bg_card_hover())
            .stroke(Stroke::new(1.0, theme::border()))
            .min_size(egui::vec2(64.0, 26.0)),
    )
}

fn primary_action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(theme::bg_dark()).size(11.0))
            .fill(theme::accent())
            .stroke(Stroke::new(1.0, theme::accent_dim()))
            .min_size(egui::vec2(72.0, 28.0)),
    )
}

fn parent_remote_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    let idx = trimmed.rfind('/')?;
    if idx == 0 {
        Some("/".into())
    } else {
        Some(trimmed[..idx].into())
    }
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
        self.sync_system_theme(ctx);

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
            if self.selected.is_none() && !self.projects.is_empty() {
                self.select_project(0);
            }
        }

        egui::TopBottomPanel::top("titlebar")
            .exact_height(32.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::NONE
                    .fill(theme::bg_dark())
                    .inner_margin(egui::Margin {
                        left: 4,
                        right: 2,
                        top: 1,
                        bottom: 0,
                    }),
            )
            .show(ctx, |ui| {
                self.draw_titlebar(ui, ctx);
            });

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::bg_dark())
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 4,
                        bottom: 4,
                    }),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if let Some(idx) = self.selected
                        && let Some(p) = self.projects.get(idx)
                    {
                        ui.label(
                            RichText::new(p.path.display().to_string())
                                .color(theme::text_muted())
                                .size(10.0),
                        );
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::bg_dark())
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 16,
                        top: 0,
                        bottom: 8,
                    }),
            )
            .show(ctx, |ui| {
                self.draw_main_workspace(ui);
            });

        if self.show_sources {
            egui::Area::new(egui::Id::new("sources_overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    self.draw_sources_overlay(ui, ctx);
                });
        }
    }
}
