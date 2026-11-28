mod agents;
mod code;
mod commits;
mod docs;
mod guard;
mod pull_request;

use crate::git::Repo;
use anyhow::{Result, bail};

pub(crate) type Failures = Vec<String>;

pub(crate) struct Args {
    values: Vec<String>,
}

impl Args {
    pub(crate) fn value(&self, flag: &str) -> Option<String> {
        self.values
            .iter()
            .position(|v| v == flag)
            .and_then(|i| self.values.get(i + 1))
            .cloned()
    }

    pub(crate) fn required(&self, flag: &str) -> Result<String> {
        match self.value(flag) {
            Some(value) => Ok(value),
            None => bail!("missing {flag} <value>"),
        }
    }
}

const REPO_CHECKS: &[&str] = &[
    "agent-entry-links",
    "skills-link",
    "skills-in-owner-guide",
    "doc-size",
    "doc-index",
    "doc-links",
    "no-placeholders",
    "doc-style",
    "record-numbers",
    "no-suppressions",
    "test-scope",
    "dependency-pinning",
    "no-dead-code",
];

const USAGE: &str = "usage: cargo xtask <check> [options]

whole-repository checks:
  agent-entry-links | skills-link | skills-in-owner-guide | doc-size | doc-index | doc-links
  no-placeholders | doc-style | record-numbers | no-suppressions | test-scope
  dependency-pinning | no-dead-code
  repo-checks                  run every whole-repository check (the merged-result check)

change checks:
  no-comments --base <rev> [--head <rev>|:index]
  commit-message --file <path> | --range <base>..<head>
  risk-tier --base <rev> --head <rev> [--github-output <path>]
  evidence-bundle --body-file <path> --head-sha <sha> --tier <n> --repo <owner/name> [--ci-run-url <url>]
  behavior-has-test --base <rev> --head <rev> --body-file <path>

agent command guard:
  guard                        read a Claude Code PreToolUse event on stdin";

fn run_one(repo: &Repo, name: &str, args: &Args) -> Result<Failures> {
    match name {
        "agent-entry-links" => agents::entry_links(repo),
        "skills-link" => agents::skills_link(repo),
        "skills-in-owner-guide" => agents::skills_in_owner_guide(repo),
        "doc-size" => docs::doc_size(repo),
        "doc-index" => docs::doc_index(repo),
        "doc-links" => docs::doc_links(repo),
        "no-placeholders" => docs::no_placeholders(repo),
        "doc-style" => docs::doc_style(repo),
        "record-numbers" => docs::record_numbers(repo),
        "no-suppressions" => code::no_suppressions(repo),
        "test-scope" => code::test_scope(repo),
        "dependency-pinning" => code::dependency_pinning(repo),
        "no-dead-code" => code::no_dead_code(repo),
        "no-comments" => code::no_comments(repo, args),
        "commit-message" => commits::commit_message(repo, args),
        "risk-tier" => pull_request::risk_tier(repo, args),
        "evidence-bundle" => pull_request::evidence_bundle(args),
        "behavior-has-test" => pull_request::behavior_has_test(repo, args),
        "repo-checks" => {
            let mut failures = Failures::new();
            for check in REPO_CHECKS {
                let found = run_one(repo, check, args)?;
                let status = if found.is_empty() { "ok" } else { "FAILED" };
                println!("{check}: {status}");
                failures.extend(found);
            }
            Ok(failures)
        }
        _ => bail!("unknown check \"{name}\"\n\n{USAGE}"),
    }
}

pub(crate) fn run(raw: &[String]) -> Result<Failures> {
    let Some((name, rest)) = raw.split_first() else {
        bail!("{USAGE}");
    };
    if name == "guard" {
        return guard::run();
    }
    let args = Args {
        values: rest.to_vec(),
    };
    let repo = Repo::discover()?;
    run_one(&repo, name, &args)
}
