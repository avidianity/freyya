use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) enum Revision {
    Commit(String),
    Index,
}

impl Revision {
    pub(crate) fn parse(text: &str) -> Self {
        if text == ":index" {
            Self::Index
        } else {
            Self::Commit(text.to_owned())
        }
    }
}

pub(crate) struct Repo {
    pub root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Clone, Debug)]
pub(crate) struct Change {
    pub path: String,
    pub kind: ChangeKind,
}

impl Repo {
    pub(crate) fn discover() -> Result<Self> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("running git rev-parse")?;
        if !output.status.success() {
            bail!("not inside a git repository");
        }
        let root = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok(Self {
            root: PathBuf::from(root),
        })
    }

    pub(crate) fn git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["-c", "core.quotepath=off"])
            .args(args)
            .output()
            .with_context(|| format!("running git {}", args.join(" ")))?;
        if !output.status.success() {
            bail!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    pub(crate) fn files(&self) -> Result<Vec<String>> {
        let listing = self.git(&["ls-files", "--cached", "--others", "--exclude-standard"])?;
        let mut files: Vec<String> = listing
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| self.root.join(line).exists())
            .map(str::to_owned)
            .collect();
        files.sort();
        files.dedup();
        Ok(files)
    }

    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub(crate) fn read(&self, relative: &str) -> Result<String> {
        let path = self.path(relative);
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    pub(crate) fn show(&self, revision: &Revision, path: &str) -> Result<Option<String>> {
        let spec = match revision {
            Revision::Commit(commit) => format!("{commit}:{path}"),
            Revision::Index => format!(":{path}"),
        };
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["show", &spec])
            .output()
            .context("running git show")?;
        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
        } else {
            Ok(None)
        }
    }

    fn diff_args(base: &str, head: &Revision) -> Vec<String> {
        match head {
            Revision::Commit(commit) => vec![format!("{base}...{commit}")],
            Revision::Index => vec!["--cached".to_owned(), base.to_owned()],
        }
    }

    pub(crate) fn changes(&self, base: &str, head: &Revision) -> Result<Vec<Change>> {
        let mut args = vec![
            "diff".to_owned(),
            "--name-status".to_owned(),
            "--no-renames".to_owned(),
            "-z".to_owned(),
        ];
        args.extend(Self::diff_args(base, head));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        Ok(parse_name_status(&self.git(&refs)?))
    }

    pub(crate) fn added_lines(
        &self,
        base: &str,
        head: &Revision,
    ) -> Result<BTreeMap<String, BTreeSet<usize>>> {
        let mut args = vec![
            "diff".to_owned(),
            "--unified=0".to_owned(),
            "--no-renames".to_owned(),
        ];
        args.extend(Self::diff_args(base, head));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let patch = self.git(&refs)?;
        Ok(parse_added_lines(&patch))
    }
}

pub(crate) fn parse_name_status(listing: &str) -> Vec<Change> {
    let fields: Vec<&str> = listing.split('\0').collect();
    fields
        .chunks(2)
        .filter_map(|pair| match pair {
            [status, path] if !status.is_empty() && !path.is_empty() => {
                let kind = match status.chars().next() {
                    Some('A') => ChangeKind::Added,
                    Some('D') => ChangeKind::Deleted,
                    _ => ChangeKind::Modified,
                };
                Some(Change {
                    path: (*path).to_owned(),
                    kind,
                })
            }
            _ => None,
        })
        .collect()
}

fn diff_path(raw: &str) -> Option<String> {
    let raw = raw.trim_end_matches('\t');
    let unquoted = match raw.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        Some(inner) => {
            let mut out = String::new();
            let mut chars = inner.chars();
            while let Some(c) = chars.next() {
                if c != '\\' {
                    out.push(c);
                    continue;
                }
                match chars.next() {
                    Some('t') => out.push('\t'),
                    Some('n') => out.push('\n'),
                    Some(other) => out.push(other),
                    None => {}
                }
            }
            out
        }
        None => raw.to_owned(),
    };
    unquoted.strip_prefix("b/").map(str::to_owned)
}

pub(crate) fn parse_added_lines(patch: &str) -> BTreeMap<String, BTreeSet<usize>> {
    let mut added: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in patch.lines() {
        if let Some(path) = line.strip_prefix("+++ ") {
            current = diff_path(path);
            continue;
        }
        let Some(hunk) = line.strip_prefix("@@ ") else {
            continue;
        };
        let Some(file) = current.clone() else {
            continue;
        };
        let Some(new_range) = hunk.split_whitespace().find(|part| part.starts_with('+')) else {
            continue;
        };
        let mut numbers = new_range.trim_start_matches('+').splitn(2, ',');
        let start: usize = numbers.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        let count: usize = numbers.next().and_then(|n| n.parse().ok()).unwrap_or(1);
        let lines = added.entry(file).or_default();
        for offset in 0..count {
            lines.insert(start + offset);
        }
    }
    added
}

pub(crate) fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{ChangeKind, parse_added_lines, parse_name_status};

    #[test]
    fn name_status_keeps_quotes_and_tabs_in_paths() {
        let listing = "A\0.github/workflows/a\"b.yml\0M\0policy/tier\tmap.toml\0D\0old.rs\0";
        let changes = parse_name_status(listing);
        let paths: Vec<&str> = changes.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                ".github/workflows/a\"b.yml",
                "policy/tier\tmap.toml",
                "old.rs"
            ]
        );
        assert_eq!(changes[2].kind, ChangeKind::Deleted);
    }

    #[test]
    fn added_lines_follow_hunk_headers() {
        let patch = "diff --git a/x.rs b/x.rs\n--- a/x.rs\n+++ b/x.rs\n@@ -1,0 +2,2 @@\n+a\n+b\n@@ -9 +12 @@\n-c\n+d\n";
        let added = parse_added_lines(patch);
        let lines: Vec<usize> = added["x.rs"].iter().copied().collect();
        assert_eq!(lines, vec![2, 3, 12]);
    }

    #[test]
    fn paths_with_spaces_and_quotes_keep_their_name() {
        let patch = "+++ b/crates/freyya/src/bin/my tool.rs\t\n@@ -0,0 +1 @@\n+// x\n+++ \"b/a\\\"b.rs\"\n@@ -0,0 +1 @@\n+// y\n";
        let added = parse_added_lines(patch);
        assert!(added.contains_key("crates/freyya/src/bin/my tool.rs"));
        assert!(added.contains_key("a\"b.rs"));
    }

    #[test]
    fn deleted_files_have_no_added_lines() {
        let patch = "--- a/gone.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-a\n-b\n";
        assert!(parse_added_lines(patch).is_empty());
    }
}
