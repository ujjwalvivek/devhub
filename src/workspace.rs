use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};

use crate::discovery::{Project, ProjectSource};

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
}

pub fn list_local_subdirs(path: &Path) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push(DirEntry { name, path: entry.path() });
        }
    }
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

pub fn list_remote_subdirs(host: &str, path: &str) -> Result<Vec<DirEntry>> {
    let script = format!("find {} -maxdepth 1 -type d ! -name '.' 2>/dev/null | sort", shell_quote(path));
    let output = run_ssh_script(host, &script)?;
    let mut entries = Vec::new();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let p = PathBuf::from(line);
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name != "." && !name.is_empty() {
                entries.push(DirEntry { name: name.to_string(), path: p });
            }
        }
    }
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

pub fn list_drives() -> Vec<DirEntry> {
    let mut drives = Vec::new();
    for letter in 'A'..='Z' {
        let path = format!("{letter}:\\");
        if Path::new(&path).exists() {
            drives.push(DirEntry {
                name: format!("{letter}:"),
                path: PathBuf::from(&path),
            });
        }
    }
    drives
}

const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_PREVIEW_BYTES: usize = 200 * 1024;
const MAX_TREE_ENTRIES: usize = 500;
const MAX_SEARCH_HITS: usize = 200;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub path: PathBuf,
    pub line: usize,
    pub preview: String,
}

pub fn list_tree(project: &Project, max_depth: usize) -> Result<Vec<FileEntry>> {
    match &project.source {
        ProjectSource::Local => list_local_tree(&project.path, max_depth),
        ProjectSource::Remote { host, .. } => list_remote_tree(host, &project.path, max_depth),
    }
}

pub fn read_file(project: &Project, path: &Path) -> Result<String> {
    match &project.source {
        ProjectSource::Local => read_local_file(path),
        ProjectSource::Remote { host, .. } => read_remote_file(host, path),
    }
}

pub fn search(project: &Project, query: &str) -> Result<Vec<SearchHit>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    match &project.source {
        ProjectSource::Local => search_local(&project.path, query),
        ProjectSource::Remote { host, .. } => search_remote(host, &project.path, query),
    }
}

fn list_local_tree(root: &Path, max_depth: usize) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .max_depth(Some(max_depth))
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|entry| !is_skipped_path(entry.path()))
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path == root {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap_or(path);
        let depth = relative.components().count().saturating_sub(1);
        let name = relative.to_string_lossy().replace('\\', "/");
        entries.push(FileEntry {
            name,
            path: path.to_path_buf(),
            depth,
            is_dir: path.is_dir(),
        });
        if entries.len() >= MAX_TREE_ENTRIES {
            break;
        }
    }

    sort_entries(&mut entries);
    Ok(entries)
}

fn list_remote_tree(host: &str, root: &Path, max_depth: usize) -> Result<Vec<FileEntry>> {
    let script = format!(
        r#"root={root}
find "$root" -maxdepth {max_depth} \( -name .git -o -name node_modules -o -name target -o -name .next -o -name vendor \) -prune -o -mindepth 1 \( -type d -printf 'd\t%p\n' -o -type f -printf 'f\t%p\n' \) 2>/dev/null | head -n {limit}
"#,
        root = shell_quote(&root.to_string_lossy()),
        max_depth = max_depth,
        limit = MAX_TREE_ENTRIES,
    );
    let output = run_ssh_script(host, &script)?;
    let root_raw = root.to_string_lossy();
    let mut entries = output
        .lines()
        .filter_map(|line| {
            let (kind, path) = line.split_once('\t')?;
            let name = path
                .strip_prefix(root_raw.as_ref())
                .unwrap_or(path)
                .trim_start_matches('/')
                .to_string();
            if name.is_empty() {
                return None;
            }
            let depth = name.split('/').count().saturating_sub(1);
            Some(FileEntry {
                name,
                path: PathBuf::from(path),
                depth,
                is_dir: kind == "d",
            })
        })
        .collect::<Vec<_>>();

    sort_entries(&mut entries);
    Ok(entries)
}

fn read_local_file(path: &Path) -> Result<String> {
    let metadata =
        std::fs::metadata(path).with_context(|| format!("reading {}", path.display()))?;
    if !metadata.is_file() {
        return Err(anyhow!("not a file"));
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(anyhow!("file is larger than {} KiB", MAX_FILE_BYTES / 1024));
    }

    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_remote_file(host: &str, path: &Path) -> Result<String> {
    let script = format!(
        r#"path={path}
if [ ! -f "$path" ]; then
    exit 2
fi
head -c {limit} "$path"
"#,
        path = shell_quote(&path.to_string_lossy()),
        limit = MAX_PREVIEW_BYTES,
    );
    run_ssh_script(host, &script)
}

fn search_local(root: &Path, query: &str) -> Result<Vec<SearchHit>> {
    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|entry| !is_skipped_path(entry.path()))
        .build();

    for entry in walker.flatten() {
        if hits.len() >= MAX_SEARCH_HITS {
            break;
        }

        let path = entry.path();
        if !path.is_file() || is_probably_binary_or_large(path) {
            continue;
        }

        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&bytes);
        for (line_idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                hits.push(SearchHit {
                    path: path.to_path_buf(),
                    line: line_idx + 1,
                    preview: line.trim().chars().take(240).collect(),
                });
                if hits.len() >= MAX_SEARCH_HITS {
                    break;
                }
            }
        }
    }

    Ok(hits)
}

fn search_remote(host: &str, root: &Path, query: &str) -> Result<Vec<SearchHit>> {
    let script = format!(
        r#"root={root}
query={query}
grep -RIn --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.next --binary-files=without-match -- "$query" "$root" 2>/dev/null | head -n {limit} || true
"#,
        root = shell_quote(&root.to_string_lossy()),
        query = shell_quote(query),
        limit = MAX_SEARCH_HITS,
    );
    let output = run_ssh_script(host, &script)?;
    Ok(output
        .lines()
        .filter_map(parse_grep_hit)
        .collect::<Vec<_>>())
}

fn parse_grep_hit(line: &str) -> Option<SearchHit> {
    let (path, rest) = line.split_once(':')?;
    let (line_no, preview) = rest.split_once(':')?;
    Some(SearchHit {
        path: PathBuf::from(path),
        line: line_no.parse().ok()?,
        preview: preview.trim().chars().take(240).collect(),
    })
}

fn run_ssh_script(host: &str, script: &str) -> Result<String> {
    let mut child = Command::new("ssh")
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("ConnectTimeout=5")
        .arg(host)
        .arg("sh")
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("starting ssh command for {host}"))?;

    child
        .stdin
        .as_mut()
        .context("opening ssh stdin")?
        .write_all(script.as_bytes())
        .with_context(|| format!("sending script to {host}"))?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("waiting for ssh command on {host}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ssh command failed for {host}: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn is_skipped_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | "node_modules" | "target" | ".next" | "vendor"
            )
        })
}

fn is_probably_binary_or_large(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return true;
    };
    if metadata.len() > MAX_FILE_BYTES {
        return true;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "exe"
                    | "dll"
                    | "pdb"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "webp"
                    | "ico"
                    | "zip"
                    | "7z"
                    | "tar"
                    | "gz"
                    | "pdf"
                    | "otf"
                    | "ttf"
                    | "woff"
                    | "woff2"
            )
        })
}

fn sort_entries(entries: &mut [FileEntry]) {
    entries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| right.is_dir.cmp(&left.is_dir))
    });
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
