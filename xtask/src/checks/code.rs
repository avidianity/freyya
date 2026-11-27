use super::{Args, Failures};
use crate::git::{Repo, Revision};
use crate::policy;
use crate::rust_source::{self, Comment};
use anyhow::{Context, Result, bail};
use std::process::Command;

fn rust_files(repo: &Repo) -> Result<Vec<String>> {
    Ok(repo
        .files()?
        .into_iter()
        .filter(|p| p.ends_with(".rs") && !p.starts_with("target/"))
        .collect())
}

pub fn no_suppressions(repo: &Repo) -> Result<Failures> {
    let approved = policy::suppressions(repo)?.approved;
    let mut failures = Failures::new();
    for path in rust_files(repo)? {
        let facts = rust_source::facts(&repo.read(&path)?).with_context(|| format!("in {path}"))?;
        for found in facts.suppressions {
            let allowed = approved
                .iter()
                .any(|entry| entry.path == path && found.lint.contains(&entry.lint));
            if !allowed {
                failures.push(format!(
                    "{path}:{} suppresses {}. Fix the cause, or change the rule for the whole Project through an ADR.",
                    found.line,
                    if found.lint.is_empty() { "a lint" } else { &found.lint }
                ));
            }
        }
    }
    Ok(failures)
}

const COVERAGE_CONFIGS: &[&str] = &[
    "tarpaulin.toml",
    ".tarpaulin.toml",
    "codecov.yml",
    ".codecov.yml",
    "mutants.toml",
    ".cargo/mutants.toml",
];

pub fn test_scope(repo: &Repo) -> Result<Failures> {
    let layout = policy::layout(repo)?.tests;
    let app_source = policy::glob_set(&layout.app_source)?;
    let app_tests = policy::glob_set(&layout.app_tests)?;
    let script_tests = policy::glob_set(&layout.script_tests)?;
    let script_path = layout.script_tests.join(", ");
    let mut failures = Failures::new();
    for path in rust_files(repo)? {
        let facts = rust_source::facts(&repo.read(&path)?).with_context(|| format!("in {path}"))?;
        if facts.tests.is_empty() {
            continue;
        }
        let in_app = app_source.is_match(&path) || app_tests.is_match(&path);
        if !in_app && !script_tests.is_match(&path) {
            failures.push(format!(
                "{path} is outside the declared application test paths. Do not add automated tests for declarative tool configs or low-risk helper scripts; put custom check-script or high-risk ops-script tests in {script_path}. See testing.md."
            ));
            continue;
        }
        if in_app {
            for text in &facts.test_strings {
                if let Some(config) = layout
                    .config_names
                    .iter()
                    .find(|name| text.contains(name.as_str()))
                {
                    failures.push(format!(
                        "{path} tests the tool config or repository file {config}. Do not add automated tests for declarative tool configs or low-risk helper scripts; validate them by running the tool. See testing.md."
                    ));
                }
            }
        }
    }
    for config in COVERAGE_CONFIGS {
        if repo.path(config).exists() {
            failures.push(format!(
                "Coverage or mutation config {config} includes non-application paths (no allow-list is approved). Collect only declared application source paths and exclude configs, scripts, generated files, migrations, type-only files, and test helpers; see testing.md."
            ));
        }
    }
    Ok(failures)
}

pub fn dependency_pinning(repo: &Repo) -> Result<Failures> {
    let output = Command::new("cargo")
        .current_dir(&repo.root)
        .args(["metadata", "--format-version", "1", "--locked"])
        .output()
        .context("running cargo metadata")?;
    if !output.status.success() {
        return Ok(vec![format!(
            "Cargo.lock is stale or missing: {}. Pin it and update the lockfile.",
            String::from_utf8_lossy(&output.stderr).trim()
        )]);
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let members: Vec<String> = metadata["workspace_members"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|id| id.as_str().map(str::to_owned))
        .collect();
    let mut failures = Failures::new();
    let packages = metadata["packages"].as_array().cloned().unwrap_or_default();
    let workspace_packages = packages.iter().filter(|p| {
        p["id"]
            .as_str()
            .is_some_and(|id| members.iter().any(|m| m == id))
    });
    for package in workspace_packages {
        let owner = package["name"].as_str().unwrap_or("?");
        for dependency in package["dependencies"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if dependency["path"].is_string() {
                continue;
            }
            let name = dependency["name"].as_str().unwrap_or("?");
            let requirement = dependency["req"].as_str().unwrap_or("");
            let exact = requirement.starts_with('=') && !requirement.contains(',');
            if !exact {
                failures.push(format!(
                    "{name} (in {owner}) is not pinned to an exact version ({requirement}), or the lockfile is stale. Pin it and update the lockfile."
                ));
            }
        }
    }
    Ok(failures)
}

fn tag_ok(text: &str) -> bool {
    let body = text
        .trim_start_matches("//")
        .trim_start_matches("/*")
        .trim_end_matches("*/")
        .trim();
    if body.starts_with("INVARIANT: ") || body.starts_with("SAFETY: ") {
        return true;
    }
    let Some(rest) = body.strip_prefix("WORKAROUND(") else {
        return false;
    };
    let Some((reference, why)) = rest.split_once("):") else {
        return false;
    };
    let reference = reference.trim();
    let has_ref = reference.starts_with('#')
        || reference.starts_with("ADR")
        || reference.starts_with("http://")
        || reference.starts_with("https://");
    has_ref && !why.trim().is_empty()
}

fn is_plain_line_comment(text: &str) -> bool {
    text.starts_with("//") && !text.starts_with("///") && !text.starts_with("//!")
}

pub fn disallowed_comments(comments: &[Comment]) -> Vec<&Comment> {
    let mut bad = Vec::new();
    let mut previous_tagged_line: Option<usize> = None;
    for comment in comments {
        let continuation = is_plain_line_comment(&comment.text)
            && previous_tagged_line.is_some_and(|line| line + 1 == comment.line);
        if tag_ok(&comment.text) || continuation {
            previous_tagged_line = Some(comment.line);
        } else {
            previous_tagged_line = None;
            bad.push(comment);
        }
    }
    bad
}

pub fn no_comments(repo: &Repo, args: &Args) -> Result<Failures> {
    let Some(base) = args.value("--base") else {
        bail!("no-comments needs --base <rev>");
    };
    let head = Revision::parse(&args.value("--head").unwrap_or_else(|| ":index".to_owned()));
    let mut failures = Failures::new();
    for (path, added) in repo.added_lines(&base, &head)? {
        if !path.ends_with(".rs") || path.contains("/generated/") {
            continue;
        }
        let Some(source) = repo.show(&head, &path)? else {
            continue;
        };
        if source.contains("@generated") {
            continue;
        }
        let comments = rust_source::comments(&source);
        for comment in disallowed_comments(&comments) {
            let span = comment.text.matches('\n').count();
            let touched = (comment.line..=comment.line + span).any(|line| added.contains(&line));
            if touched {
                failures.push(format!(
                    "{path}:{} adds an untagged comment. Delete it, put the reason in a name, ADR, test, commit message, or issue, or tag a non-obvious constraint with INVARIANT:, SAFETY:, or WORKAROUND(<ref>):; see engineering.md.",
                    comment.line
                ));
            }
        }
    }
    Ok(failures)
}

#[cfg(test)]
mod tests {
    use super::{disallowed_comments, tag_ok};
    use crate::rust_source::comments;

    #[test]
    fn tagged_constraints_pass() {
        assert!(tag_ok("// SAFETY: the buffer outlives the call"));
        assert!(tag_ok("// INVARIANT: ids are sorted before merge"));
        assert!(tag_ok("// WORKAROUND(#412): flush buffer before close"));
        assert!(tag_ok("/* WORKAROUND(https://x.dev/1): retry once */"));
    }

    #[test]
    fn narration_doc_and_unreferenced_workarounds_fail() {
        assert!(!tag_ok("// increment the counter"));
        assert!(!tag_ok("/// Opens the window."));
        assert!(!tag_ok("// WORKAROUND: flaky"));
        assert!(!tag_ok("// TODO: later"));
    }

    #[test]
    fn continuation_lines_follow_a_tag() {
        let src = "fn a() {\n    // SAFETY: the pointer is valid\n    // for the whole call.\n    let x = 1;\n    // stray narration\n    let _ = x;\n}\n";
        let found = comments(src);
        let bad: Vec<usize> = disallowed_comments(&found).iter().map(|c| c.line).collect();
        assert_eq!(bad, vec![5]);
    }
}
