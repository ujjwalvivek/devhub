use std::path::PathBuf;
use std::process::Command;

use crate::config::EditorConfig;
use crate::discovery::{Project, ProjectSource};

pub fn open_project(editor: &EditorConfig, project: &Project) {
    let Some(template) = template_for(editor, project) else {
        tracing::warn!(
            editor = %editor.name,
            source = %project.source.label(),
            "no editor template for project source"
        );
        return;
    };

    let parts = parse_template(template, project);
    if parts.is_empty() {
        tracing::error!(editor = %editor.name, "empty editor command");
        return;
    }

    let program = &parts[0];
    let args = &parts[1..];
    tracing::info!(
        editor = %editor.name,
        program = %program,
        args = ?args,
        "launching editor"
    );

    match spawn_command(program, args) {
        Ok(_) => tracing::info!("editor launched"),
        Err(error) => tracing::error!(%error, "failed to launch editor"),
    }
}

fn spawn_command(program: &str, args: &[String]) -> std::io::Result<std::process::Child> {
    match Command::new(program).args(args).spawn() {
        Ok(child) => Ok(child),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && program.eq_ignore_ascii_case("code") =>
        {
            spawn_vscode(args).or(Err(error))
        }
        Err(error) => Err(error),
    }
}

fn spawn_vscode(args: &[String]) -> std::io::Result<std::process::Child> {
    for candidate in vscode_candidates() {
        if candidate.exists() {
            if candidate
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("cmd"))
            {
                return Command::new("cmd")
                    .arg("/C")
                    .arg(candidate)
                    .args(args)
                    .spawn();
            }
            return Command::new(candidate).args(args).spawn();
        }
    }

    Command::new("cmd").arg("/C").arg("code").args(args).spawn()
}

fn vscode_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let base = PathBuf::from(local_app_data)
            .join("Programs")
            .join("Microsoft VS Code");
        candidates.push(base.join("Code.exe"));
        candidates.push(base.join("bin").join("code.cmd"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let base = PathBuf::from(program_files).join("Microsoft VS Code");
        candidates.push(base.join("Code.exe"));
        candidates.push(base.join("bin").join("code.cmd"));
    }
    candidates
}

pub fn can_open_project(editor: &EditorConfig, project: &Project) -> bool {
    template_for(editor, project).is_some()
}

fn template_for<'a>(editor: &'a EditorConfig, project: &Project) -> Option<&'a str> {
    match &project.source {
        ProjectSource::Local => editor.local.as_deref().or(editor.command.as_deref()),
        ProjectSource::Remote { .. } => editor.remote.as_deref(),
    }
    .filter(|template| !template.trim().is_empty())
}

fn parse_template(template: &str, project: &Project) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;

    for ch in template.chars() {
        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    parts.push(expand_placeholders(&current, project));
                    current.clear();
                }
            }
            None => current.push(ch),
        }
    }

    if !current.is_empty() {
        parts.push(expand_placeholders(&current, project));
    }

    parts
}

fn expand_placeholders(input: &str, project: &Project) -> String {
    let path = project.path.to_string_lossy();
    let host = project.source.host().unwrap_or_default();
    let remote_uri = if host.is_empty() {
        path.to_string()
    } else {
        format!("vscode-remote://ssh-remote+{host}{path}")
    };

    input
        .replace("{path}", &path)
        .replace("{file}", &path)
        .replace("{line}", "1")
        .replace("{host}", host)
        .replace("{remote_uri}", &remote_uri)
}
