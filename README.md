# Devhub

<p>
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/stars?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="stars">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/updated?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="updated">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/health?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="health">
  <img src="https://echopoint.ujjwalvivek.com/svg/badges/release?repo=devhub&logo=github&bg=111111&badgeColor=2b2b2b&textColor=e8e8e8&border=555555&borderWidth=2&rx=0&px=6&py=4" height="24" alt="releases">
</p>

Native egui/eframe desktop app that scans filesystems and SSH hosts for software projects (Cargo.toml, package.json, go.mod, etc.), indexes them, and presents a three-pane browser with file tree, content search, git state, and telemetry.

```sh
cargo run --release # from the project root, requires rust 2024
```

### Dependencies

| eframe 0.31 (wgpu) | egui_extras 0.31 | serde 1 + toml 0.8               | resvg 0.37 | open 5   |
| ------------------ | ---------------- | -------------------------------- | ---------- | -------- |
| ignore 0.4         | directories 6    | tracing 0.1 + tracing-subscriber | anyhow 1   | rfd 0.14 |

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

## Config.toml Configuration

```toml
theme = "CatppuccinMocha"
appearance = "System"
scan_dirs = ["~/projects"]
max_depth = 3
remote_hosts = [
  { host = "user@server", roots = ["/home/user/projects"], max_depth = 3 }
]
default_editor = "code"
[editors.code]
command = "code {path}"
remote = "vscode-remote://ssh-remote+{host}{path}"
```

## What's in it?

- local (`ignore::WalkBuilder`, respects .gitignore) and remote (SSH `find` + `stat` + `git config`)
- types detected include `Rust (Cargo.toml)`, `Node (package.json)`, `Go (go.mod)`, `Python (requirements.txt)`, `Make`, `CMake`, `Assembly (.asm)`, `.NET (.sln)`, `Java (Gradle/Maven)`
- collapsible directories, file icons, cap at 500 entries
- local searches via `ignore` crate, remote searches via SSH `grep -RIn`, 200-hit cap
- SSH-over-pipe, BatchMode, configurable max depth per host
- SVG rendered via `resvg` from echopoint.ujjwalvivek.com
- launcher defaults include VS Code (local + SSH remote URI), Zed (local)
- Catppuccin Mocha, Rose Pine Moon, Tokyo Night Storm, Horizon Bold, Monochrome Zero + light variants; follows OS theme
- background thread scanning → mpsc channel → cache to disk (TOML, versioned)
- TOML at platform config dir `(~/.config/devhub/config.toml or %APPDATA%/devhub/config.toml)`
