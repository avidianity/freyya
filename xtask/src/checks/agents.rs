use super::Failures;
use crate::git::{Repo, is_symlink};
use crate::policy;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

fn resolves_to(repo: &Repo, link: &str, expected: &str) -> bool {
    let path = repo.path(link);
    if !is_symlink(&path) {
        return false;
    }
    let Ok(target) = std::fs::read_link(&path) else {
        return false;
    };
    let joined = Path::new(link)
        .parent()
        .unwrap_or(Path::new(""))
        .join(target);
    let mut parts: Vec<PathBuf> = Vec::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(part) => parts.push(PathBuf::from(part)),
            _ => {}
        }
    }
    let resolved: PathBuf = parts.iter().collect();
    resolved == Path::new(expected) && repo.path(expected).exists()
}

pub fn entry_links(repo: &Repo) -> Result<Failures> {
    let layout = policy::layout(repo)?;
    let mut failures = Failures::new();
    if !repo.path(".rules").is_file() {
        failures
            .push(".rules is missing. Restore it; every agent entry file links to it.".to_owned());
    }
    for file in &layout.agents.entry_files {
        if !resolves_to(repo, file, ".rules") {
            failures.push(format!(
                "{file} must be a link to .rules. Edit .rules only, then recreate the link."
            ));
        }
    }
    Ok(failures)
}

fn skill_names(repo: &Repo, dir: &str) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(repo.path(dir)).with_context(|| format!("reading {dir}"))? {
        let entry = entry?;
        if entry.path().join("SKILL.md").is_file() {
            names.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(names)
}

fn pinned_skills(repo: &Repo) -> Result<BTreeSet<String>> {
    let lock: serde_json::Value = serde_json::from_str(&repo.read("skills-lock.json")?)
        .context("parsing skills-lock.json")?;
    Ok(lock
        .get("skills")
        .and_then(serde_json::Value::as_object)
        .map(|skills| skills.keys().cloned().collect())
        .unwrap_or_default())
}

pub fn skills_link(repo: &Repo) -> Result<Failures> {
    let layout = policy::layout(repo)?;
    let skills_dir = &layout.agents.skills_dir;
    let mut failures = Failures::new();
    for link in &layout.agents.skills_links {
        if !resolves_to(repo, link, skills_dir) {
            failures.push(format!("{link} must be a link to {skills_dir}."));
        }
    }
    let pinned = pinned_skills(repo)?;
    let project: BTreeSet<&String> = layout.agents.project_skills.iter().collect();
    for skill in skill_names(repo, skills_dir)? {
        if !pinned.contains(&skill) && !project.contains(&skill) {
            failures.push(format!(
                "{skill} is not pinned in the skills lock file. Install it with the skills installer."
            ));
        }
    }
    Ok(failures)
}

pub fn skills_in_owner_guide(repo: &Repo) -> Result<Failures> {
    let layout = policy::layout(repo)?;
    let guide = repo.read("docs/owner-guide.md")?;
    Ok(skill_names(repo, &layout.agents.skills_dir)?
        .into_iter()
        .filter(|skill| !guide.contains(&format!("`{skill}`")))
        .map(|skill| {
            format!(
                "{skill} is not in docs/owner-guide.md. Add when the Owner starts it, what to type, and what comes back, or list it as a helper."
            )
        })
        .collect())
}
