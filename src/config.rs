use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub onboarding_complete: bool,

    #[serde(default)]
    pub scan_dirs: Vec<PathBuf>,

    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    #[serde(default)]
    pub remote_hosts: Vec<RemoteHostConfig>,

    #[serde(default = "default_editors")]
    pub editors: HashMap<String, EditorConfig>,

    #[serde(default = "default_editor_key")]
    pub default_editor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteHostConfig {
    #[serde(default)]
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub roots: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

impl RemoteHostConfig {
    pub fn label(&self) -> &str {
        if self.name.is_empty() {
            &self.host
        } else {
            &self.name
        }
    }
}

fn default_max_depth() -> usize {
    3
}

fn default_editor_key() -> String {
    "code".into()
}

fn default_editors() -> HashMap<String, EditorConfig> {
    let mut m = HashMap::new();
    m.insert(
        "code".into(),
        EditorConfig {
            name: "VS Code".into(),
            command: None,
            local: Some("code {path}".into()),
            remote: Some("code --remote ssh-remote+{host} {path}".into()),
        },
    );
    m.insert(
        "zed".into(),
        EditorConfig {
            name: "Zed".into(),
            command: None,
            local: Some("zed {path}".into()),
            remote: None,
        },
    );
    m
}

impl Default for Config {
    fn default() -> Self {
        Self {
            onboarding_complete: false,
            scan_dirs: Vec::new(),
            max_depth: default_max_depth(),
            remote_hosts: Vec::new(),
            editors: default_editors(),
            default_editor: default_editor_key(),
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("reading config at {}", path.display()))?;
            let mut cfg: Config = toml::from_str(&raw).with_context(|| "parsing config.toml")?;
            cfg.apply_defaults();
            Ok(cfg)
        } else {
            let cfg = Config::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let serialized = toml::to_string_pretty(&cfg)?;
            std::fs::write(&path, &serialized)
                .with_context(|| format!("writing default config to {}", path.display()))?;
            tracing::info!(path = %path.display(), "created default config");
            Ok(cfg)
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("", "", "devhub")
            .context("cannot determine config directory")?;
        Ok(dirs.config_dir().join("config.toml"))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        let serialized = toml::to_string_pretty(self).context("serializing config")?;
        std::fs::write(&path, serialized)
            .with_context(|| format!("writing config to {}", path.display()))
    }

    fn apply_defaults(&mut self) {
        if !self.onboarding_complete
            && self.remote_hosts.is_empty()
            && is_legacy_home_scan(&self.scan_dirs)
        {
            self.scan_dirs.clear();
        }

        let defaults = default_editors();
        for (key, default_editor) in &defaults {
            self.editors
                .entry(key.clone())
                .or_insert_with(|| default_editor.clone());
        }

        for (key, editor) in &mut self.editors {
            if editor.local.is_none() {
                editor.local = editor.command.clone();
            }
            if editor.remote.is_none() {
                editor.remote = defaults.get(key).and_then(|default| default.remote.clone());
            }
        }

        self.merge_remote_hosts();
    }

    fn merge_remote_hosts(&mut self) {
        let mut merged: Vec<RemoteHostConfig> = Vec::new();

        for mut remote in self.remote_hosts.drain(..) {
            remote.host = normalize_ssh_host(&remote.host);
            remote.roots.retain(|root| !root.trim().is_empty());

            if remote.host.is_empty() || remote.roots.is_empty() {
                continue;
            }

            if let Some(existing) = merged.iter_mut().find(|item| item.host == remote.host) {
                if existing.name.is_empty() {
                    existing.name = remote.name;
                }
                for root in remote.roots {
                    if !existing.roots.iter().any(|seen| seen == &root) {
                        existing.roots.push(root);
                    }
                }
            } else {
                merged.push(remote);
            }
        }

        self.remote_hosts = merged;
    }
}

fn dirs_home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().to_path_buf())
}

fn is_legacy_home_scan(scan_dirs: &[PathBuf]) -> bool {
    scan_dirs.len() == 1 && dirs_home().is_some_and(|home| scan_dirs[0] == home)
}

fn normalize_ssh_host(raw: &str) -> String {
    raw.trim()
        .strip_prefix("ssh ")
        .map(str::trim)
        .unwrap_or_else(|| raw.trim())
        .to_string()
}
