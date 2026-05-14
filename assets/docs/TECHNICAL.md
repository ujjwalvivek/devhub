# Devhub, A Technical Reference

<p>
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/stars?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="stars">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/updated?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="updated">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/health?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="health">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/commits?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="commits">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/release?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="releases">
</p>

Native desktop cockpit for local and remote SSH project discovery, browsing, and editor handoff. egui/eframe (wgpu), zero webview.

## Stack

| Layer            | Choice                                                    |
| ---------------- | --------------------------------------------------------- |
| Language         | Rust (edition 2024)                                       |
| GUI              | egui 0.31 (immediate-mode), eframe (wgpu backend)         |
| Config           | TOML via serde, platform config dir (`directories` crate) |
| Cache            | TOML, versioned, at platform cache dir                    |
| Local walking    | `ignore` crate (.gitignore-aware, replaces raw `walkdir`) |
| Remote transport | System `ssh` binary (BatchMode, stdin-piped scripts)      |
| SVG rendering    | `resvg` (rasterize echopoint SVGs to egui textures)       |
| Image loading    | Custom `EchopointSvgLoader` (egui `ImageLoader` trait)    |
| HTTP             | egui_extras http transport                                |
| Font             | MonaspaceNeon-Regular.otf embedded via `include_bytes!`   |
| Logging          | `tracing` + `tracing-subscriber` (env-filter)             |
| Error handling   | `anyhow`                                                  |

### Dependencies

```toml
[dependencies]
eframe = { version = "0.31", default-features = false, features = ["wgpu"] }
egui_extras = { version = "0.31", default-features = false, features = ["http"] }
resvg = { version = "0.37", default-features = false, features = ["text", "system-fonts"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
ignore = "0.4"
directories = "6"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
open = "5"
rfd = "0.14"
```

## Build

```sh
cargo run --release
cargo build --release
cargo clippy -- -D warnings
```

## Architecture

```bash
src/
├── main.rs            // eframe entry, window config, config load
├── app.rs             // devhub state, sources overlay, file tree and picker
├── config.rs          // serde TOML config, scan dirs, remote hosts, editors, theme, appearance
├── discovery/
│   ├── mod.rs
│   └── scanner.rs     // scanning, marker-file detection, git state, remote SSH piped find
├── workspace.rs       // file tree listing, file read, remote SSH grep
├── editor.rs
├── cache.rs           // TOML-serialized project cache (versioned)
└── ui/
    └── theme.rs       // 5 themes, monaspace neon font, thread-local colors
```

### State

1. All state lives in the `DevHub` struct (eframe `App`). 
2. No retained widget tree and egui recalculates every frame.
3. Key fields: `config: Config`, `projects: Vec<Project>`, `selected: Option<usize>`, `scan_status: ScanStatus`, `show_sources: bool`, `picker_state: PickerState`, `workbench: WorkbenchState`

### Scanning

1. Background thread runs `discovery::scan_directories` + per-host `scan_remote_host`
2. Results sent via `mpsc::channel`
3. Main thread picks up via `try_recv()` in `update()`
4. On success: replaces `self.projects`, caches to disk (TOML, versioned)
5. On startup: loads cache if `onboarding_complete && config_has_sources`

### Local project discovery

1. Marker files detected: `Cargo.toml`, `package.json`, `go.mod`, `requirements.txt`, `Makefile`, `CMakeLists.txt`, `.sln`, `build.gradle`, `pom.xml`, `.asm`.
2. Git state: reads `.git/config` for remote origin URL. Marks dirty/clean via `git status --porcelain`.

```rust
ignore::WalkBuilder::new(dir)
    .max_depth(max_depth)
    .git_ignore(true)
    .git_global(true)
    .git_exclude(true)
    .build()
```

### Remote project discovery

1. SSH piped script that runs `find` + `stat` + `git config` on the remote host. 
2. Project markers matched via `basename` filtering. Output parsed line-by-line.
3. Config: `RemoteHostConfig { host, roots: Vec<PathBuf>, max_depth }`. 
4. Requires BatchMode key-based SSH (no password prompting).

```bash
find <root> -maxdepth <n> \( <marker_flags> \) -printf '%h\0' 2>/dev/null
```

### File and Search

1. Local: `ignore::WalkBuilder` (respects .gitignore), cap 500 entries
2. Remote: SSH `find` with prune, cap 500 entries
3. UI: collapsible directories via `HashSet<PathBuf>`, painted full-width rows
4. Local: `std::fs::read`, max 512KB
5. Remote: SSH `head -c 204800`, max 200KB
6. Local: `ignore::WalkBuilder` + case-insensitive line scan
7. Remote: SSH `grep -RIn` with `--include` filters
8. Cap: 200 hits, binary files skipped

### Default Configuration

1. Default path: platform config dir / `config.toml`.
2. `apply_defaults()`: fills missing editors, normalizes SSH hosts, deduplicates remote roots.

```toml
theme = "CatppuccinMocha"
appearance = "System"
scan_dirs = ["~/projects"]
max_depth = 3
onboarding_complete = false
default_editor = "code"

[remote_hosts]
hosts = [
  { host = "user@server", roots = ["/home/user/projects"], max_depth = 3 }
]

[editors]
# template-based, see Editor launch below

[disabled_local_dirs]
[disabled_remote_hosts]
```

### Editor launch Configuration

```toml
[editors.code]
command = "code {path}"
remote = "vscode-remote://ssh-remote+{host}{path}"

[editors.zed]
command = "zed {path}"
```

### Telemetry (Echopoint)

1. SVG badges/cards fetched from `echopoint.ujjwalvivek.com`
2. Alias resolved from git remote → GitHub path → tracked repo list fallback.
3. Rendered via custom `EchopointSvgLoader` (egui `ImageLoader` trait) that uses `resvg` to rasterize SVGs. 
4. Cached in a `Mutex<HashMap<(Uri, SizeHint), ImageResult>>`.

```bash
https://echopoint.ujjwalvivek.com/svg/badges/<kind>?repo=<alias>&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4
```

### UI layout

1. Sources overlay (full-screen semi-transparent `egui::Area`) replaces the onboarding page flow. 
2. Contains in-app directory picker with drive navigation (Windows), remote host management.

```
┌─────────────────────────────────────────────────┐
│ hamburger | brand | filter |      theme | - □ X │
├──────────┬──────────┬───────────────────────────┤
│ Projects │ Details  │ File Tree / Search        │
│ Cards    │ Stats    │ / Preview                 │
│          │ Badges   │                           │
│          │ Editor   │                           │
│          │ Buttons  │                           │
│          │          │                           │
├──────────┴──────────┴───────────────────────────┤
│ Status bar: project path                        │
└─────────────────────────────────────────────────┘
```

### Themes

1. 5 palettes, each with auto-generated light variant (5-color interpolation)
2. Each palette: `bg_dark`, `bg_panel`, `bg_card`, `bg_card_hover`, `bg_card_active`, `bg_field`, `text_primary`, `text_dim`, `text_muted`, `accent`, `accent_dim`, `border`, `git_badge`.
3. Active colors stored in thread-local `Cell<(Colors, Colors)>` (dark + light). Accessed via `theme::text_primary()`, `theme::bg_dark()`, etc.
4. `AppearanceMode`: `System` (follows OS), `Dark`, `Light`.

### Known Issues

- remote SSH requires BatchMode key-based authentication; no password/agent prompting
- file tree capped at 500 entries per project
- content search on remote hosts may be slow on large directory trees
- native file dialog (rfd) dependency (windows) present but unused in current UI paths
- incremental scan not supported; rescans everything on each scan trigger
- selected project index may become stale after rescans (corrected on next scan)

## Where to from here?

- Per-source scan status and errors
- Manual refresh per root or host
- Pin/favorite projects
- Hide/archive projects
- Better empty states
- Optional project notes
- SSH config parsing (`~/.ssh/config`) via `ssh_config` crate
- Remote shell/editor templates
- Build/test command shortcuts
- Project insight (git status on demand, line counts)
- Keyboard-first launcher overlay
- Recent/fuzzy project filter
- GitHub OAuth or sync using `oauth-device-flows` Turnkey
- `tokei`/code statistics (defer until manually requested)
- Global full-text search index (`grep`)
- `tokei` (library) + `grep` hybrid JSON over SSH
- `SkimV2` algorithm for fuzzy search
- `keyring-core` for credential storage
- `ignore` for `.gitignore`-style pattern matching
