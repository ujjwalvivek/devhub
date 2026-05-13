use std::path::PathBuf;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::discovery::{Project, sort_projects};

const CACHE_VERSION: u32 = 4;

#[derive(Debug, Serialize, Deserialize)]
struct ProjectCache {
    version: u32,
    projects: Vec<Project>,
}

pub fn load_projects() -> Result<Vec<Project>> {
    let path = cache_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut cache: ProjectCache =
        toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    if cache.version != CACHE_VERSION {
        return Ok(Vec::new());
    }

    for project in &mut cache.projects {
        project.refresh_search_key();
    }
    sort_projects(&mut cache.projects);

    Ok(cache.projects)
}

pub fn save_projects(projects: &[Project]) -> Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating cache dir {}", parent.display()))?;
    }

    let cache = ProjectCache {
        version: CACHE_VERSION,
        projects: projects.to_vec(),
    };
    let raw = toml::to_string(&cache).context("serializing project cache")?;
    std::fs::write(&path, raw).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn cache_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "devhub")
        .context("cannot determine cache directory")?;
    Ok(dirs.cache_dir().join("projects.toml"))
}
