use super::Failures;
use crate::git::Repo;
use crate::markdown;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};

const RULES_BUDGET: usize = 8_000;
const TASK_PAGE_BUDGET: usize = 16_000;
const DOC_BUDGET: usize = 32_000;

fn is_active_doc(path: &str) -> bool {
    path.ends_with(".md")
        && !path.split('/').any(|part| part == "archive")
        && !path.starts_with(".agents/")
        && !path.starts_with(".claude/")
        && !path.starts_with("target/")
}

fn budget_for(path: &str) -> Option<(usize, &'static str)> {
    let name = path.rsplit('/').next().unwrap_or(path);
    if path == ".rules" {
        Some((RULES_BUDGET, ".rules"))
    } else if (path.starts_with("docs/agents/") && path.ends_with(".md")) || name == "RULES.md" {
        Some((TASK_PAGE_BUDGET, "task pages and domain RULES.md"))
    } else if is_active_doc(path)
        && !path.starts_with("docs/adr/")
        && !path.starts_with("docs/research/")
    {
        Some((DOC_BUDGET, "active docs"))
    } else {
        None
    }
}

pub(super) fn doc_size(repo: &Repo) -> Result<Failures> {
    let mut failures = Failures::new();
    for path in repo.files()? {
        let Some((limit, tier)) = budget_for(&path) else {
            continue;
        };
        let size = std::fs::metadata(repo.path(&path))?.len() as usize;
        if size >= limit {
            failures.push(format!(
                "{path} is {size} bytes; the budget for {tier} is {limit}. Move domain detail into a domain folder or split into an index and topic guides; never raise the budget."
            ));
        }
    }
    Ok(failures)
}

fn needs_index(path: &str) -> bool {
    if !is_active_doc(path) || path.starts_with("docs/adr/") || path.starts_with("docs/research/") {
        return false;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    path.starts_with("docs/agents/")
        || ["GLOSSARY.md", "HOW-TO.md", "STATE.md", "RULES.md"].contains(&name)
        || path.contains("/how-to/")
        || path.contains("/state/")
        || path.starts_with("how-to/")
        || path.starts_with("state/")
}

fn normalize(base: &Path, target: &str) -> Option<String> {
    let target = target.split('#').next().unwrap_or_default();
    if target.is_empty() || target.contains("://") || target.starts_with("mailto:") {
        return None;
    }
    let joined: PathBuf = if let Some(stripped) = target.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        base.parent().unwrap_or(Path::new("")).join(target)
    };
    let mut parts: Vec<String> = Vec::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                if parts.last().is_none_or(|last| last == "..") {
                    parts.push("..".to_owned());
                } else {
                    parts.pop();
                }
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    Some(parts.join("/"))
}

pub(super) fn doc_index(repo: &Repo) -> Result<Failures> {
    let files: BTreeSet<String> = repo.files()?.into_iter().collect();
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::from([".rules".to_owned()]);
    while let Some(current) = queue.pop_front() {
        if !reached.insert(current.clone()) {
            continue;
        }
        let is_markdown = current == ".rules" || current.ends_with(".md");
        if !is_markdown || !files.contains(&current) {
            continue;
        }
        let content = repo.read(&current)?;
        for link in markdown::links(&content) {
            if let Some(target) = normalize(Path::new(&current), &link.target)
                && files.contains(&target)
                && !reached.contains(&target)
            {
                queue.push_back(target);
            }
        }
    }
    Ok(files
        .iter()
        .filter(|path| needs_index(path) && !reached.contains(*path))
        .map(|path| {
            format!(
                "{path} is not transitively reachable from .rules. Link it with a read trigger from .rules, MAP.md, GLOSSARY-MAP.md, or a reachable parent HOW-TO.md / STATE.md / docs/agents/ page, or move obsolete docs to archive/."
            )
        })
        .collect())
}

fn link_scope(path: &str) -> bool {
    path == ".rules" || is_active_doc(path)
}

pub(super) fn doc_links(repo: &Repo) -> Result<Failures> {
    let files: BTreeSet<String> = repo.files()?.into_iter().collect();
    let mut anchors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut failures = Failures::new();
    for path in files.iter().filter(|p| link_scope(p)) {
        let content = repo.read(path)?;
        for link in markdown::links(&content) {
            let fragment = link.target.split_once('#').map(|(_, f)| f.to_owned());
            let target = if link.target.starts_with('#') {
                Some(path.clone())
            } else {
                normalize(Path::new(path), &link.target)
            };
            let Some(target) = target else {
                continue;
            };
            let is_dir = repo.path(&target).is_dir();
            if !files.contains(&target) && !is_dir {
                failures.push(format!(
                    "{path}:{} links to {target}, which does not exist. Fix the link or restore the target.",
                    link.line
                ));
                continue;
            }
            let Some(fragment) = fragment.filter(|f| !f.is_empty()) else {
                continue;
            };
            if !target.ends_with(".md") {
                continue;
            }
            if !anchors.contains_key(&target) {
                anchors.insert(
                    target.clone(),
                    markdown::heading_anchors(&repo.read(&target)?),
                );
            }
            let known = anchors
                .get(&target)
                .is_some_and(|list| list.contains(&fragment));
            if !known {
                failures.push(format!(
                    "{path}:{} links to {target}#{fragment}, which does not exist. Fix the link or restore the target.",
                    link.line
                ));
            }
        }
    }
    Ok(failures)
}

fn placeholder_scope(path: &str) -> bool {
    path == ".rules"
        || path == "docs/owner-guide.md"
        || (path.starts_with("docs/agents/") && path.ends_with(".md"))
        || (path.starts_with("docs/checks/") && path.ends_with(".md"))
}

pub(super) fn no_placeholders(repo: &Repo) -> Result<Failures> {
    let mut failures = Failures::new();
    for path in repo.files()?.into_iter().filter(|p| placeholder_scope(p)) {
        for found in markdown::placeholders(&repo.read(&path)?) {
            failures.push(format!(
                "{path}:{} still holds the placeholder {}. Replace it with the Project value.",
                found.line, found.text
            ));
        }
    }
    Ok(failures)
}

pub(super) fn doc_style(repo: &Repo) -> Result<Failures> {
    let mut failures = Failures::new();
    for path in repo.files()? {
        if !(path == ".rules" || is_active_doc(&path)) {
            continue;
        }
        for (index, line) in repo.read(&path)?.lines().enumerate() {
            if line.contains('\u{2014}') {
                failures.push(format!(
                    "{path}:{} uses an em dash. Use a plain dash \"-\" or split the sentence.",
                    index + 1
                ));
            }
        }
    }
    Ok(failures)
}

pub(super) fn record_numbers(repo: &Repo) -> Result<Failures> {
    let mut by_number: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for path in repo.files()? {
        let Some(name) = path.strip_prefix("docs/adr/") else {
            continue;
        };
        if name.contains('/') || !name.ends_with(".md") {
            continue;
        }
        let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(number) = digits.parse::<u32>() {
            by_number.entry(number).or_default().push(path.clone());
        }
    }
    let mut failures = Failures::new();
    let next_free = by_number.keys().max().map_or(1, |max| max + 1);
    for (number, paths) in &by_number {
        if let [first, second, ..] = paths.as_slice() {
            failures.push(format!(
                "Record number {number:04} is used by {first} and {second}. Renumber the newer one to {next_free:04}."
            ));
        }
    }
    for (expected, number) in (1u32..).zip(by_number.keys()) {
        if *number != expected {
            failures.push(format!(
                "Record numbers have a gap: {expected:04} is missing before {number:04}. Renumber so that records run 0001, 0002, and so on."
            ));
            break;
        }
    }
    Ok(failures)
}

#[cfg(test)]
mod tests {
    use super::{budget_for, needs_index, normalize};
    use std::path::Path;

    #[test]
    fn budgets_follow_tiers() {
        assert_eq!(budget_for(".rules").map(|b| b.0), Some(8_000));
        assert_eq!(
            budget_for("docs/agents/testing.md").map(|b| b.0),
            Some(16_000)
        );
        assert_eq!(
            budget_for("crates/store/RULES.md").map(|b| b.0),
            Some(16_000)
        );
        assert_eq!(budget_for("GLOSSARY.md").map(|b| b.0), Some(32_000));
        assert_eq!(budget_for("docs/adr/archive/0001-x.md"), None);
    }

    #[test]
    fn index_scope_exempts_adrs_research_and_archive() {
        assert!(needs_index("docs/agents/git.md"));
        assert!(needs_index("crates/store/HOW-TO.md"));
        assert!(!needs_index("docs/adr/0001-x.md"));
        assert!(!needs_index("docs/research/x.md"));
        assert!(!needs_index("docs/agents/archive/old.md"));
    }

    #[test]
    fn links_resolve_relative_to_their_file() {
        assert_eq!(
            normalize(Path::new("docs/agents/git.md"), "../checks/x.md#a"),
            Some("docs/checks/x.md".to_owned())
        );
        assert_eq!(
            normalize(Path::new(".rules"), "docs/agents/git.md"),
            Some("docs/agents/git.md".to_owned())
        );
        assert_eq!(normalize(Path::new(".rules"), "https://x.dev"), None);
        assert_eq!(
            normalize(
                Path::new("docs/agents/git.md"),
                "../../../../RELEASE-NOTES.md"
            ),
            Some("../../RELEASE-NOTES.md".to_owned())
        );
    }
}
