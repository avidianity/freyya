use super::{Args, Failures};
use crate::git::{ChangeKind, Repo, Revision};
use crate::policy::{self, TierMap};
use crate::rust_source::{self, Facts};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

struct Finding {
    tier: u8,
    path: String,
    why: String,
}

fn approval_rule(tier: u8) -> &'static str {
    match tier {
        0..=2 => "green CI and a complete Evidence bundle; at Gate 0 the Owner merges",
        3 => "an Independent reviewer's PASS, then the Owner's yes on a Risk summary",
        _ => "the Owner's yes on a Risk summary, and the Owner starts or watches it",
    }
}

struct Crate {
    dir: String,
    depends_on: Vec<String>,
}

fn workspace_crates(repo: &Repo) -> Result<BTreeMap<String, Crate>> {
    let output = Command::new("cargo")
        .current_dir(&repo.root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .context("running cargo metadata")?;
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("reading cargo metadata")?;
    let root = repo.root.to_string_lossy().replace('\\', "/");
    let mut crates = BTreeMap::new();
    for package in metadata["packages"].as_array().cloned().unwrap_or_default() {
        let name = package["name"].as_str().unwrap_or_default().to_owned();
        let manifest = package["manifest_path"]
            .as_str()
            .unwrap_or_default()
            .replace('\\', "/");
        let dir = manifest
            .trim_end_matches("/Cargo.toml")
            .trim_start_matches(&root)
            .trim_start_matches('/')
            .to_owned();
        let depends_on = package["dependencies"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter(|d| d["path"].is_string())
            .filter_map(|d| d["name"].as_str().map(str::to_owned))
            .collect();
        crates.insert(name, Crate { dir, depends_on });
    }
    Ok(crates)
}

fn dependents(crates: &BTreeMap<String, Crate>, of: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut frontier = vec![of.to_owned()];
    while let Some(current) = frontier.pop() {
        for (name, krate) in crates {
            if krate.depends_on.contains(&current) && found.insert(name.clone()) {
                frontier.push(name.clone());
            }
        }
    }
    found
}

fn reverse_dependency_findings(
    path: &str,
    crates: &BTreeMap<String, Crate>,
    map: &TierMap,
) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let owner = crates
        .iter()
        .filter(|(_, k)| !k.dir.is_empty() && path.starts_with(&format!("{}/", k.dir)))
        .max_by_key(|(_, k)| k.dir.len());
    let Some((owner_name, _)) = owner else {
        return Ok(findings);
    };
    for dependent in dependents(crates, owner_name) {
        let dir = &crates[&dependent].dir;
        let (tier, _) = map.tier_of(&format!("{dir}/src/lib.rs"))?;
        if tier >= 3 {
            findings.push(Finding {
                tier,
                path: path.to_owned(),
                why: format!("reverse dependency from {dir}"),
            });
        }
    }
    Ok(findings)
}

fn facts_at(repo: &Repo, revision: &Revision, path: &str) -> Result<Option<Facts>> {
    match repo.show(revision, path)? {
        Some(source) => Ok(rust_source::facts(&source).ok()),
        None => Ok(None),
    }
}

fn signal_findings(path: &str, before: &Facts, after: &Facts) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut add = |tier: u8, why: &str| {
        findings.push(Finding {
            tier,
            path: path.to_owned(),
            why: format!("diff signal {why}"),
        });
    };
    if after.network_endpoints > before.network_endpoints {
        add(3, "new external network endpoint");
    }
    if after.secret_reads > before.secret_reads {
        add(3, "new environment or secret read");
    }
    if after.disk_writes > before.disk_writes {
        add(3, "new file write or delete on disk");
    }
    if after.bulk_deletes > before.bulk_deletes {
        add(4, "bulk or irreversible data deletion");
    }
    if after.tests.len() < before.tests.len() {
        add(3, "weakens tests (a test was deleted)");
    }
    if after.ignored_tests > before.ignored_tests {
        add(3, "weakens tests (a test was skipped)");
    }
    if after.assertions < before.assertions {
        add(3, "weakens tests (an assertion was removed)");
    }
    if after.suppressions.len() > before.suppressions.len() {
        add(3, "adds a lint suppression");
    }
    findings
}

fn source_findings(
    repo: &Repo,
    base: &str,
    head: &Revision,
    path: &str,
    kind: &ChangeKind,
) -> Result<Vec<Finding>> {
    let base_rev = Revision::Commit(base.to_owned());
    let before = if *kind == ChangeKind::Added {
        Facts::default()
    } else {
        facts_at(repo, &base_rev, path)?.unwrap_or_default()
    };
    if *kind == ChangeKind::Deleted {
        let mut findings = Vec::new();
        if !before.tests.is_empty() {
            findings.push(Finding {
                tier: 3,
                path: path.to_owned(),
                why: "diff signal weakens tests (a file with tests was deleted)".to_owned(),
            });
        }
        return Ok(findings);
    }
    match facts_at(repo, head, path)? {
        Some(after) => Ok(signal_findings(path, &before, &after)),
        None => Ok(vec![Finding {
            tier: 3,
            path: path.to_owned(),
            why: "diff signal unparseable Rust source".to_owned(),
        }]),
    }
}

pub(super) fn risk_tier(repo: &Repo, args: &Args) -> Result<Failures> {
    let base = args.required("--base")?;
    let head = Revision::parse(&args.required("--head")?);
    let map = policy::tier_map_at(repo, &base)?;
    let script_paths = policy::glob_set(&policy::layout_at(repo, &base)?.tests.script_tests)?;
    let gate = policy::gate_at(repo, &base)?;
    let crates = workspace_crates(repo)?;
    let base_rev = Revision::Commit(base.clone());
    let mut findings = Vec::new();
    for change in repo.changes(&base, &head)? {
        let (tier, why) = map.tier_of(&change.path)?;
        findings.push(Finding {
            tier,
            path: change.path.clone(),
            why,
        });
        let name = change.path.rsplit('/').next().unwrap_or_default();
        if name == "Cargo.toml" || name == "Cargo.lock" {
            findings.push(Finding {
                tier: 2,
                path: change.path.clone(),
                why: "diff signal dependency manifest or lockfile change".to_owned(),
            });
        }
        if name == "Cargo.toml" {
            let lints_of = |text: Option<String>| {
                text.and_then(|t| t.parse::<toml::Table>().ok()).map(|m| {
                    let own = m.get("lints").cloned();
                    let shared = m.get("workspace").and_then(|w| w.get("lints")).cloned();
                    (own, shared)
                })
            };
            let before = lints_of(repo.show(&base_rev, &change.path)?);
            let after = lints_of(repo.show(&head, &change.path)?);
            if before != after {
                findings.push(Finding {
                    tier: 3,
                    path: change.path.clone(),
                    why: "diff signal changes lint configuration".to_owned(),
                });
            }
        }
        findings.extend(reverse_dependency_findings(&change.path, &crates, &map)?);
        if change.path.ends_with(".rs") && !script_paths.is_match(&change.path) {
            findings.extend(source_findings(
                repo,
                &base,
                &head,
                &change.path,
                &change.kind,
            )?);
        }
    }
    let tier = findings.iter().map(|f| f.tier).max().unwrap_or(0);
    println!("Tier {tier}");
    for finding in findings.iter().filter(|f| f.tier == tier) {
        println!(
            "Tier {}: {} matched {}. This Tier needs {}.",
            finding.tier,
            finding.path,
            finding.why,
            approval_rule(finding.tier)
        );
    }
    let merger = if gate.agent_may_merge(tier) {
        "an agent may merge under the merge contract"
    } else {
        "the Owner merges"
    };
    println!(
        "Autonomy gate {} (policy/gate.toml): {merger}.",
        gate.active_gate
    );
    if let Some(output) = args.value("--github-output") {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output)?;
        writeln!(file, "tier={tier}")?;
    }
    Ok(Failures::new())
}

const SECTIONS: &[&str] = &[
    "Tier",
    "Task",
    "Fail-first test",
    "Checks",
    "Safety proof",
    "Runtime evidence",
    "Gaps",
];

fn sections(body: &str) -> BTreeMap<String, String> {
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in body.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            current = Some(title.trim().to_owned());
            found.entry(title.trim().to_owned()).or_default();
        } else if let Some(title) = &current {
            let entry = found.entry(title.clone()).or_default();
            entry.push_str(line);
            entry.push('\n');
        }
    }
    found
}

pub(super) fn visible_text(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => rest = "",
        }
    }
    out.push_str(rest);
    out
}

fn has_template_line(body: &str) -> bool {
    body.lines().any(|line| {
        let mut rest = line;
        while let Some(start) = rest.find('<') {
            let after = &rest[start + 1..];
            let Some(end) = after.find('>') else {
                return false;
            };
            let inner = &after[..end];
            let sentence = inner.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && inner.contains(' ')
                && inner.chars().any(|c| c.is_ascii_lowercase());
            if sentence {
                return true;
            }
            rest = &after[end + 1..];
        }
        false
    })
}

fn is_run_url(token: &str, repo: &str) -> bool {
    let Some(rest) = token.strip_prefix(&format!("https://github.com/{repo}/actions/runs/")) else {
        return false;
    };
    let run = rest.split('/').next().unwrap_or_default();
    !run.is_empty() && run.chars().all(|c| c.is_ascii_digit())
}

fn has_run_artifact(text: &str, head_sha: &str, repo: &str) -> bool {
    let short = &head_sha[..head_sha.len().min(7)];
    let has_url = text
        .split(|c: char| c.is_whitespace() || "()<>[]`\"'".contains(c))
        .any(|token| is_run_url(token, repo));
    has_url && short.len() == 7 && text.contains(short)
}

fn not_applicable(text: &str) -> bool {
    text.trim_start()
        .strip_prefix("Not applicable:")
        .is_some_and(|reason| reason.trim().len() >= 3)
}

pub(super) fn bundle_problems(body: &str, head_sha: &str, tier: u8, repo: &str) -> Vec<String> {
    let visible = visible_text(body);
    let body = visible.as_str();
    let found = sections(body);
    let mut problems = Vec::new();
    for section in SECTIONS {
        match found.get(*section) {
            Some(text) if !text.trim().is_empty() => {}
            _ => problems.push(format!("lacks {section}")),
        }
    }
    let revision = body
        .lines()
        .find_map(|line| line.trim().strip_prefix("Revision:"))
        .map(|rest| rest.trim().trim_matches('`').trim().to_owned())
        .unwrap_or_default();
    if revision != head_sha {
        let shown = if revision.is_empty() {
            "none"
        } else {
            revision.as_str()
        };
        problems.push(format!(
            "cites stale revision {shown} instead of head {head_sha}"
        ));
    }
    if crate::markdown::placeholders(body)
        .iter()
        .any(|p| !p.text.starts_with("<!--"))
    {
        problems.push("lacks content: it still holds template placeholders".to_owned());
    }
    if has_template_line(body) {
        problems.push("lacks content: it still holds template lines".to_owned());
    }
    if tier >= 2 {
        let safety = found.get("Safety proof").cloned().unwrap_or_default();
        let gaps = found
            .get("Gaps")
            .cloned()
            .unwrap_or_default()
            .to_lowercase();
        let unproven_ok = tier == 2
            && safety.to_lowercase().contains("unproven")
            && (gaps.contains("unproven") || gaps.contains("safety"));
        if !has_run_artifact(&safety, head_sha, repo) && !unproven_ok {
            problems.push("lacks a revision-stamped artifact for the Safety proof (a GitHub Actions run URL and the head SHA)".to_owned());
        }
        let runtime = found.get("Runtime evidence").cloned().unwrap_or_default();
        if !has_run_artifact(&runtime, head_sha, repo) && !not_applicable(&runtime) {
            problems.push("lacks a revision-stamped artifact for the Runtime evidence".to_owned());
        }
    }
    problems
}

pub(super) fn evidence_bundle(args: &Args) -> Result<Failures> {
    let body = std::fs::read_to_string(args.required("--body-file")?)?;
    let head_sha = args.required("--head-sha")?;
    let tier: u8 = args
        .required("--tier")?
        .parse()
        .context("--tier must be 0 to 4")?;
    let repo = args.required("--repo")?;
    Ok(bundle_problems(&body, &head_sha, tier, &repo)
        .into_iter()
        .map(|problem| format!("The pull request Evidence bundle {problem}. Use templates/pr-evidence.md (see docs/agents/review-and-merge.md)."))
        .collect())
}

fn states_no_behavior_change(body: &str) -> bool {
    body.lines().any(|line| {
        line.trim()
            .strip_prefix("No behavior change")
            .is_some_and(|rest| rest.trim_start_matches([':', ' ']).len() >= 3)
    })
}

pub(super) fn behavior_has_test(repo: &Repo, args: &Args) -> Result<Failures> {
    let base = args.required("--base")?;
    let head = Revision::parse(&args.required("--head")?);
    let body = std::fs::read_to_string(args.required("--body-file")?)?;
    let layout = policy::layout_at(repo, &base)?.tests;
    let app_source = policy::glob_set(&layout.app_source)?;
    let app_tests = policy::glob_set(&layout.app_tests)?;
    let changes = repo.changes(&base, &head)?;
    let source_changes: Vec<_> = changes
        .iter()
        .filter(|c| app_source.is_match(&c.path))
        .collect();
    if source_changes.is_empty() || states_no_behavior_change(&visible_text(&body)) {
        return Ok(Failures::new());
    }
    let base_rev = Revision::Commit(base.clone());
    let candidates = changes.iter().filter(|c| {
        c.kind != ChangeKind::Deleted
            && c.path.ends_with(".rs")
            && (app_source.is_match(&c.path) || app_tests.is_match(&c.path))
    });
    for change in candidates {
        let before = facts_at(repo, &base_rev, &change.path)?.unwrap_or_default();
        let after = facts_at(repo, &head, &change.path)?.unwrap_or_default();
        let touched = after
            .tests
            .iter()
            .any(|(name, text)| before.tests.get(name) != Some(text));
        if touched {
            return Ok(Failures::new());
        }
    }
    let paths = layout.app_source.join(", ");
    Ok(vec![format!(
        "This pull request changes application source files under {paths} but no tests. Add a fail-first test, or state \"No behavior change\" with a reason."
    )])
}

#[cfg(test)]
mod tests {
    use super::{bundle_problems, signal_findings, states_no_behavior_change, visible_text};
    use crate::rust_source::facts;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const REPO: &str = "avidianity/freyya";

    fn body(revision: &str, safety: &str) -> String {
        format!(
            "## Summary\n\nx\n\n## Tier\n\nTier 1 from the tier check.\nRevision: `{revision}`\n\n## Task\n\nt\n\n## Fail-first test\n\nf\n\n## Checks\n\nc\n\n## Safety proof\n\n{safety}\n\n## Runtime evidence\n\nNot applicable: no user-visible change.\n\n## Gaps\n\n- Deferred coverage: none\n"
        )
    }

    #[test]
    fn complete_bundle_passes() {
        assert!(
            bundle_problems(&body(SHA, "Not applicable: Tier 0 or 1."), SHA, 1, REPO).is_empty()
        );
    }

    #[test]
    fn summary_only_body_fails() {
        assert!(bundle_problems("## Summary\n\nDid things.\n", SHA, 1, REPO).len() >= 7);
    }

    #[test]
    fn stale_revision_fails() {
        let problems = bundle_problems(
            &body("deadbeef", "Not applicable: Tier 0 or 1."),
            SHA,
            1,
            REPO,
        );
        assert!(problems.iter().any(|p| p.contains("stale revision")));
    }

    #[test]
    fn tier_three_needs_run_artifact() {
        let prose = bundle_problems(&body(SHA, "I checked it."), SHA, 3, REPO);
        assert!(prose.iter().any(|p| p.contains("Safety proof")));
        let linked = body(
            SHA,
            "Proof for 0123456: https://github.com/avidianity/freyya/actions/runs/1",
        );
        assert!(bundle_problems(&linked, SHA, 3, REPO).is_empty());
    }

    #[test]
    fn a_bare_path_or_a_link_without_the_head_sha_is_not_proof() {
        let bare = body(SHA, "Trust me, I will add /actions/runs/ later.");
        assert!(!bundle_problems(&bare, SHA, 3, REPO).is_empty());
        let unstamped = body(SHA, "https://github.com/avidianity/freyya/actions/runs/1");
        assert!(!bundle_problems(&unstamped, SHA, 3, REPO).is_empty());
    }

    #[test]
    fn runtime_not_applicable_needs_the_exact_form() {
        let proof = "Proof for 0123456: https://github.com/avidianity/freyya/actions/runs/1";
        let vague = body(SHA, proof).replace(
            "Not applicable: no user-visible change.",
            "It is Not applicable-ish",
        );
        assert!(!bundle_problems(&vague, SHA, 3, REPO).is_empty());
    }

    #[test]
    fn no_behavior_change_needs_a_reason() {
        assert!(states_no_behavior_change(
            "x\nNo behavior change: rename only.\n"
        ));
        assert!(!states_no_behavior_change("No behavior change\n"));
    }

    #[test]
    fn weakened_test_raises_tier() {
        let before = facts("#[test]\nfn a() { assert_eq!(1, 1); }\n#[test]\nfn b() {}\n")
            .unwrap_or_default();
        let after = facts("#[test]\nfn a() { }\n").unwrap_or_default();
        let found = signal_findings("x.rs", &before, &after);
        assert!(found.iter().all(|f| f.tier == 3));
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn hidden_html_comments_are_not_content() {
        let hidden = "## Tier\n\n<!-- todo -->\n\n## Task\n\n<!-- todo -->\n";
        assert!(
            bundle_problems(hidden, SHA, 1, REPO)
                .iter()
                .any(|p| p.contains("lacks Tier"))
        );
        let proof =
            "<!-- Proof for 0123456: https://github.com/avidianity/freyya/actions/runs/1 -->";
        assert!(!bundle_problems(&body(SHA, proof), SHA, 3, REPO).is_empty());
        assert!(!states_no_behavior_change(&visible_text(
            "<!--\nNo behavior change: hidden\n-->\n"
        )));
    }

    #[test]
    fn foreign_run_urls_and_template_lines_fail() {
        let foreign = body(
            SHA,
            "Proof for 0123456: https://github.com/a/b/actions/runs/1",
        );
        assert!(!bundle_problems(&foreign, SHA, 3, REPO).is_empty());
        let template = body(SHA, "Not applicable: Tier 0 or 1.").replace(
            "t\n",
            "<Link to the task, and its acceptance criteria as a checklist.>\n",
        );
        assert!(!bundle_problems(&template, SHA, 1, REPO).is_empty());
    }

    #[test]
    fn tier_two_unproven_needs_a_gaps_entry() {
        let unproven = body(SHA, "Safety fact: unproven.");
        assert!(!bundle_problems(&unproven, SHA, 2, REPO).is_empty());
        let listed = unproven.replace(
            "- Deferred coverage: none",
            "- Unproven Safety fact: see above",
        );
        assert!(bundle_problems(&listed, SHA, 2, REPO).is_empty());
    }
}
