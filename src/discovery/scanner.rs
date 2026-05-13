use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::config::RemoteHostConfig;

const MARKERS: &[(&str, ProjectType)] = &[
    ("Cargo.toml", ProjectType::Rust),
    ("package.json", ProjectType::Node),
    ("go.mod", ProjectType::Go),
    ("pyproject.toml", ProjectType::Python),
    ("requirements.txt", ProjectType::Python),
    ("Makefile", ProjectType::Make),
    ("CMakeLists.txt", ProjectType::CMake),
    ("*.asm", ProjectType::Assembly),
    ("*.sln", ProjectType::DotNet),
    ("build.gradle", ProjectType::Java),
    ("pom.xml", ProjectType::Java),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Node,
    Go,
    Python,
    Make,
    CMake,
    Assembly,
    DotNet,
    Java,
    Unknown,
}

impl ProjectType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Node => "Node",
            Self::Go => "Go",
            Self::Python => "Python",
            Self::Make => "Make",
            Self::CMake => "CMake",
            Self::Assembly => "ASM",
            Self::DotNet => ".NET",
            Self::Java => "Java",
            Self::Unknown => "???",
        }
    }

    fn from_label(label: &str) -> Self {
        match label {
            "Rust" => Self::Rust,
            "Node" => Self::Node,
            "Go" => Self::Go,
            "Python" => Self::Python,
            "Make" => Self::Make,
            "CMake" => Self::CMake,
            "ASM" | "Assembly" => Self::Assembly,
            ".NET" | "DotNet" => Self::DotNet,
            "Java" => Self::Java,
            _ => Self::Unknown,
        }
    }

    pub fn accent(self) -> eframe::egui::Color32 {
        use eframe::egui::Color32;
        match self {
            Self::Rust => Color32::from_rgb(222, 165, 95),
            Self::Node => Color32::from_rgb(130, 190, 100),
            Self::Go => Color32::from_rgb(100, 200, 220),
            Self::Python => Color32::from_rgb(180, 160, 220),
            Self::Assembly => Color32::from_rgb(200, 200, 200),
            _ => Color32::from_rgb(140, 140, 140),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectSource {
    #[default]
    Local,
    Remote {
        name: String,
        host: String,
    },
}

impl ProjectSource {
    pub fn label(&self) -> &str {
        match self {
            Self::Local => "local",
            Self::Remote { name, host } => {
                if name.is_empty() {
                    host
                } else {
                    name
                }
            }
        }
    }

    pub fn host(&self) -> Option<&str> {
        match self {
            Self::Local => None,
            Self::Remote { host, .. } => Some(host),
        }
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub source: ProjectSource,
    pub project_type: ProjectType,
    pub has_git: bool,
    pub git_remote: Option<String>,
    pub markers_found: Vec<String>,
    pub last_modified: Option<u64>,
    #[serde(default)]
    pub search_key: String,
}

impl Project {
    pub fn refresh_search_key(&mut self) {
        self.search_key = format!(
            "{} {} {} {} {}",
            self.name,
            self.path.display(),
            self.source.label(),
            self.project_type.label(),
            self.git_remote.as_deref().unwrap_or_default()
        )
        .to_lowercase();
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Done { count: usize },
    Error(String),
}

pub fn scan_directories(dirs: &[PathBuf], max_depth: usize) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in dirs {
        if !dir.exists() {
            tracing::warn!(path = %dir.display(), "scan dir does not exist, skipping");
            continue;
        }
        scan_root(dir, max_depth, &mut projects, &mut seen);
    }

    sort_projects(&mut projects);
    projects
}

pub fn scan_remote_host(host: &RemoteHostConfig) -> Result<Vec<Project>> {
    if host.roots.is_empty() {
        return Ok(Vec::new());
    }

    let ssh_target = normalize_ssh_target(&host.host);
    let script = build_remote_scan_script(&host.roots, host.max_depth);
    let mut child = Command::new("ssh")
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("ConnectTimeout=5")
        .arg(ssh_target)
        .arg("sh")
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("starting ssh scan for {ssh_target}"))?;

    child
        .stdin
        .as_mut()
        .context("opening ssh stdin")?
        .write_all(script.as_bytes())
        .with_context(|| format!("sending scan script to {ssh_target}"))?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("waiting for ssh scan on {ssh_target}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "ssh scan failed for {}: {}",
            ssh_target,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut projects = stdout
        .lines()
        .filter_map(|line| parse_remote_project(line, host, ssh_target))
        .collect::<Vec<_>>();

    sort_projects(&mut projects);
    Ok(projects)
}

pub fn sort_projects(projects: &mut [Project]) {
    projects.sort_by_cached_key(|project| {
        (
            project.source.label().to_lowercase(),
            project.name.to_lowercase(),
            project.path.to_string_lossy().to_lowercase(),
        )
    });
}

fn scan_root(
    root: &Path,
    max_depth: usize,
    projects: &mut Vec<Project>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    if let Some(project) = detect_project(root) {
        insert_project(project, projects, seen);
        return;
    }

    let walker = ignore::WalkBuilder::new(root)
        .max_depth(Some(1))
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if path == root {
            continue;
        }

        if let Some(project) = detect_project_tree(path, max_depth) {
            insert_project(project, projects, seen);
        }
    }

    let source_project = detect_project_tree(root, max_depth)
        .unwrap_or_else(|| build_local_project(root, ProjectType::Unknown, Vec::new()));
    insert_project(source_project, projects, seen);
}

fn detect_project(dir: &Path) -> Option<Project> {
    let mut markers_found = Vec::new();
    let mut project_type = ProjectType::Unknown;

    scan_markers_in_dir(dir, &mut markers_found, &mut project_type);

    if markers_found.is_empty() {
        return None;
    }

    Some(build_local_project(dir, project_type, markers_found))
}

fn detect_project_tree(root: &Path, max_depth: usize) -> Option<Project> {
    let mut markers_found = Vec::new();
    let mut project_type = ProjectType::Unknown;

    let walker = ignore::WalkBuilder::new(root)
        .max_depth(Some(max_depth))
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_markers_in_dir(path, &mut markers_found, &mut project_type);
        }
    }

    if markers_found.is_empty() {
        return None;
    }

    Some(build_local_project(root, project_type, markers_found))
}

fn build_local_project(
    dir: &Path,
    project_type: ProjectType,
    markers_found: Vec<String>,
) -> Project {
    let has_git = dir.join(".git").exists();
    let git_remote = if has_git { read_git_remote(dir) } else { None };

    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());

    let last_modified = dir
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(system_time_to_unix);

    let mut project = Project {
        name,
        path: dir.to_path_buf(),
        source: ProjectSource::Local,
        project_type,
        has_git,
        git_remote,
        markers_found,
        last_modified,
        search_key: String::new(),
    };
    project.refresh_search_key();
    project
}

fn insert_project(
    project: Project,
    projects: &mut Vec<Project>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    let canonical = match project.path.canonicalize() {
        Ok(c) => c,
        Err(_) => project.path.clone(),
    };
    if seen.insert(canonical) {
        projects.push(project);
    }
}

fn scan_markers_in_dir(
    dir: &Path,
    markers_found: &mut Vec<String>,
    project_type: &mut ProjectType,
) {
    for (marker, ptype) in MARKERS {
        let found = if let Some(ext) = marker.strip_prefix('*') {
            has_file_with_ext(dir, ext)
        } else {
            dir.join(marker).exists()
        };

        if found {
            if !markers_found.iter().any(|seen| seen == marker) {
                markers_found.push(marker.to_string());
            }
            if *project_type == ProjectType::Unknown {
                *project_type = *ptype;
            }
        }
    }
}

fn has_file_with_ext(dir: &Path, ext: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let expected = ext.strip_prefix('.').unwrap_or(ext);
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
    })
}

fn read_git_remote(dir: &Path) -> Option<String> {
    let config_path = dir.join(".git").join("config");
    let content = std::fs::read_to_string(config_path).ok()?;

    let mut in_origin = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed.contains("remote") && trimmed.contains("origin");
        } else if in_origin
            && trimmed.starts_with("url")
            && let Some((_, url)) = trimmed.split_once('=')
        {
            return Some(url.trim().to_string());
        }
    }
    None
}

fn parse_remote_project(line: &str, host: &RemoteHostConfig, ssh_target: &str) -> Option<Project> {
    let mut parts = line.split('\t');
    let name = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let project_type = ProjectType::from_label(parts.next()?);
    let has_git = parts.next()? == "true";
    let git_remote = parts
        .next()
        .filter(|remote| !remote.is_empty())
        .map(str::to_string);
    let markers_found = parts
        .next()?
        .split(',')
        .filter(|marker| !marker.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let last_modified = parts
        .next()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|ts| *ts > 0);

    let mut project = Project {
        name,
        path: PathBuf::from(path),
        source: ProjectSource::Remote {
            name: host.label().to_string(),
            host: ssh_target.to_string(),
        },
        project_type,
        has_git,
        git_remote,
        markers_found,
        last_modified,
        search_key: String::new(),
    };
    project.refresh_search_key();
    Some(project)
}

fn build_remote_scan_script(roots: &[String], max_depth: usize) -> String {
    let roots = roots
        .iter()
        .map(|root| shell_quote(root))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r#"add_marker() {{
    markers="${{markers}}$1,"
    if [ -z "$ptype" ]; then
        ptype="$2"
    fi
}}

scan_current() {{
    d="$1"
    markers=""
    ptype=""

    [ -f "$d/Cargo.toml" ] && add_marker "Cargo.toml" "Rust"
    [ -f "$d/package.json" ] && add_marker "package.json" "Node"
    [ -f "$d/go.mod" ] && add_marker "go.mod" "Go"
    [ -f "$d/pyproject.toml" ] && add_marker "pyproject.toml" "Python"
    [ -f "$d/requirements.txt" ] && add_marker "requirements.txt" "Python"
    [ -f "$d/Makefile" ] && add_marker "Makefile" "Make"
    [ -f "$d/CMakeLists.txt" ] && add_marker "CMakeLists.txt" "CMake"
    set -- "$d"/*.asm
    [ -e "$1" ] && add_marker "*.asm" "ASM"
    set -- "$d"/*.sln
    [ -e "$1" ] && add_marker "*.sln" ".NET"
    [ -f "$d/build.gradle" ] && add_marker "build.gradle" "Java"
    [ -f "$d/pom.xml" ] && add_marker "pom.xml" "Java"

    if [ -z "$markers" ]; then
        ptype="Unknown"
    fi

    has_git=false
    remote=""
    if [ -d "$d/.git" ]; then
        has_git=true
        remote="$(git -C "$d" config --get remote.origin.url 2>/dev/null || true)"
    fi

    modified="$(stat -c %Y "$d" 2>/dev/null || echo 0)"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(basename "$d")" "$d" "$ptype" "$has_git" "$remote" "$markers" "$modified"
}}

has_file() {{
    d="$1"
    pattern="$2"
    [ -n "$(find "$d" -maxdepth {max_depth} \( -name .git -o -name node_modules -o -name target -o -name .next -o -name vendor \) -prune -o -type f -name "$pattern" -print -quit 2>/dev/null)" ]
}}

scan_tree() {{
    d="$1"
    markers=""
    ptype=""

    has_file "$d" "Cargo.toml" && add_marker "Cargo.toml" "Rust"
    has_file "$d" "package.json" && add_marker "package.json" "Node"
    has_file "$d" "go.mod" && add_marker "go.mod" "Go"
    has_file "$d" "pyproject.toml" && add_marker "pyproject.toml" "Python"
    has_file "$d" "requirements.txt" && add_marker "requirements.txt" "Python"
    has_file "$d" "Makefile" && add_marker "Makefile" "Make"
    has_file "$d" "CMakeLists.txt" && add_marker "CMakeLists.txt" "CMake"
    has_file "$d" "*.asm" && add_marker "*.asm" "ASM"
    has_file "$d" "*.sln" && add_marker "*.sln" ".NET"
    has_file "$d" "build.gradle" && add_marker "build.gradle" "Java"
    has_file "$d" "pom.xml" && add_marker "pom.xml" "Java"

    if [ -z "$markers" ]; then
        ptype="Unknown"
    fi

    has_git=false
    remote=""
    if [ -d "$d/.git" ]; then
        has_git=true
        remote="$(git -C "$d" config --get remote.origin.url 2>/dev/null || true)"
    fi

    modified="$(stat -c %Y "$d" 2>/dev/null || echo 0)"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(basename "$d")" "$d" "$ptype" "$has_git" "$remote" "$markers" "$modified"
}}

for root in {roots}; do
    [ -d "$root" ] || continue
    scan_tree "$root"
done
"#
    )
}

fn normalize_ssh_target(raw: &str) -> &str {
    raw.trim()
        .strip_prefix("ssh ")
        .map(str::trim)
        .unwrap_or_else(|| raw.trim())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn system_time_to_unix(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}
