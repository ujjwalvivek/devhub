## Version 1.0.0 - Devhub

Initial release. Desktop tool for scanning, indexing, and browsing software projects across local filesystems and remote SSH hosts.

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

### Known Issues

- remote SSH requires BatchMode key-based authentication; no password/agent prompting
- file tree capped at 500 entries per project
- content search on remote hosts may be slow on large directory trees
- native file dialog (rfd) dependency (windows) present but unused in current UI paths
- incremental scan not supported; rescans everything on each scan trigger
