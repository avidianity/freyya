use crate::git::{Repo, Revision};
use anyhow::{Context, Result};
use globset::{GlobSet, GlobSetBuilder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Layout {
    pub tests: TestLayout,
    pub agents: AgentLayout,
    pub commits: CommitRules,
}

#[derive(Deserialize)]
pub struct TestLayout {
    pub app_source: Vec<String>,
    pub app_tests: Vec<String>,
    pub script_tests: Vec<String>,
    pub config_names: Vec<String>,
}

#[derive(Deserialize)]
pub struct AgentLayout {
    pub entry_files: Vec<String>,
    pub skills_dir: String,
    pub skills_links: Vec<String>,
    pub project_skills: Vec<String>,
}

#[derive(Deserialize)]
pub struct CommitRules {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
pub struct TierMap {
    pub default: u8,
    pub rule: Vec<TierRule>,
}

#[derive(Deserialize)]
pub struct TierRule {
    pub tier: u8,
    pub reason: String,
    pub paths: Vec<String>,
}

#[derive(Deserialize)]
pub struct Suppressions {
    pub approved: Vec<ApprovedSuppression>,
}

#[derive(Deserialize)]
pub struct ApprovedSuppression {
    pub path: String,
    pub lint: String,
}

fn load<T: for<'de> Deserialize<'de>>(repo: &Repo, name: &str) -> Result<T> {
    let text = repo.read(name)?;
    toml::from_str(&text).with_context(|| format!("parsing {name}"))
}

fn load_at<T: for<'de> Deserialize<'de>>(repo: &Repo, base: &str, name: &str) -> Result<T> {
    match repo.show(&Revision::Commit(base.to_owned()), name)? {
        Some(text) => toml::from_str(&text).with_context(|| format!("parsing {name} at {base}")),
        None => load(repo, name),
    }
}

pub fn layout(repo: &Repo) -> Result<Layout> {
    load(repo, "policy/layout.toml")
}

pub fn layout_at(repo: &Repo, base: &str) -> Result<Layout> {
    load_at(repo, base, "policy/layout.toml")
}

#[derive(Deserialize)]
pub struct Gate {
    pub active_gate: u8,
}

pub fn gate_at(repo: &Repo, base: &str) -> Result<Gate> {
    load_at(repo, base, "policy/gate.toml")
}

pub fn tier_map_at(repo: &Repo, base: &str) -> Result<TierMap> {
    load_at(repo, base, "policy/tier-map.toml")
}

pub fn suppressions(repo: &Repo) -> Result<Suppressions> {
    load(repo, "policy/suppressions.toml")
}

pub fn glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = globset::GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .with_context(|| format!("bad glob {pattern}"))?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

impl TierMap {
    pub fn tier_of(&self, path: &str) -> Result<(u8, String)> {
        let mut best: Option<(u8, String)> = None;
        for rule in &self.rule {
            if glob_set(&rule.paths)?.is_match(path) {
                let better = best.as_ref().is_none_or(|(tier, _)| rule.tier > *tier);
                if better {
                    best = Some((rule.tier, format!("tier map rule \"{}\"", rule.reason)));
                }
            }
        }
        Ok(best.unwrap_or((self.default, "no tier map rule (default)".to_owned())))
    }
}

#[cfg(test)]
mod tests {
    use super::{TierMap, TierRule};

    fn map() -> TierMap {
        TierMap {
            default: 2,
            rule: vec![
                TierRule {
                    tier: 0,
                    reason: "docs".to_owned(),
                    paths: vec!["**/*.md".to_owned()],
                },
                TierRule {
                    tier: 3,
                    reason: "agent rules".to_owned(),
                    paths: vec!["docs/agents/**".to_owned()],
                },
            ],
        }
    }

    #[test]
    fn highest_matching_rule_wins() {
        let (tier, _) = map()
            .tier_of("docs/agents/testing.md")
            .unwrap_or((9, String::new()));
        assert_eq!(tier, 3);
    }

    #[test]
    fn lower_rule_applies_when_only_it_matches() {
        let (tier, _) = map()
            .tier_of("docs/research/x.md")
            .unwrap_or((9, String::new()));
        assert_eq!(tier, 0);
    }

    #[test]
    fn unmatched_path_takes_default() {
        let (tier, _) = map()
            .tier_of("crates/freyya/src/main.rs")
            .unwrap_or((9, String::new()));
        assert_eq!(tier, 2);
    }
}
