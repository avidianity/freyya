use super::Failures;
use anyhow::{Context, Result};
use std::io::Read;

fn words(command: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in command.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => quote = Some(c),
            (None, c) if c.is_whitespace() || c == ';' || c == '&' || c == '|' => {
                if !current.is_empty() {
                    found.push(std::mem::take(&mut current));
                }
                if !c.is_whitespace() {
                    found.push(c.to_string());
                }
            }
            (None, c) => current.push(c),
        }
    }
    if !current.is_empty() {
        found.push(current);
    }
    found
}

fn commit_short_no_verify(words: &[String]) -> bool {
    let mut in_commit = false;
    let mut skip_value = false;
    for word in words {
        if [";", "&", "|"].contains(&word.as_str()) {
            in_commit = false;
            continue;
        }
        if word == "commit" {
            in_commit = true;
            continue;
        }
        if !in_commit {
            continue;
        }
        if skip_value {
            skip_value = false;
            continue;
        }
        if let Some(flags) = word.strip_prefix('-').filter(|f| !f.starts_with('-')) {
            if flags.contains('n') {
                return true;
            }
            if flags.ends_with('m')
                || flags.ends_with('F')
                || flags.ends_with('C')
                || flags.ends_with('c')
            {
                skip_value = true;
            }
        }
    }
    false
}

fn is_message_flag(word: &str) -> bool {
    word == "-m"
        || word == "--message"
        || word == "-F"
        || word == "--file"
        || (word.starts_with('-') && !word.starts_with("--") && word.ends_with('m'))
}

fn no_verify_outside_message(split: &[String]) -> bool {
    split.iter().enumerate().any(|(index, word)| {
        let after_message_flag = index > 0 && is_message_flag(&split[index - 1]);
        word.contains("--no-v") && !after_message_flag
    })
}

pub(super) fn bypass_reason(command: &str) -> Option<&'static str> {
    let split = words(command);
    let lower = command.to_lowercase();
    let squashed: String = lower
        .chars()
        .filter(|c| !matches!(c, '"' | '\'' | '`' | '^'))
        .map(|c| if c == '\\' { '/' } else { c })
        .collect();
    if no_verify_outside_message(&split) {
        return Some("--no-verify skips Git hooks");
    }
    if commit_short_no_verify(&split) {
        return Some("git commit -n skips Git hooks");
    }
    if squashed.contains("core.hookspath") || squashed.contains("hookspath") {
        return Some("overriding core.hooksPath disables Git hooks");
    }
    let lower_words: Vec<String> = split
        .iter()
        .map(|w| w.to_lowercase().replace('\\', "/"))
        .collect();
    let lefthook_removed = lower_words
        .windows(2)
        .any(|pair| pair[0] == "lefthook" && ["uninstall", "remove"].contains(&pair[1].as_str()));
    if lefthook_removed {
        return Some("uninstalling lefthook removes the Git hooks");
    }
    let touches_git_dir = lower_words
        .iter()
        .any(|w| w == ".git" || w.ends_with("/.git"));
    let names_hooks = lower_words
        .iter()
        .any(|w| w == "hooks" || w.ends_with("/hooks"));
    if squashed.contains(".git/hooks") || (touches_git_dir && names_hooks) {
        return Some("editing .git/hooks bypasses the managed hooks");
    }
    let lefthook_off = split.iter().any(|w| {
        let word = w.to_lowercase();
        word.split_once("lefthook=")
            .is_some_and(|(_, value)| !["1", "true"].contains(&value))
            || word.starts_with("lefthook_exclude")
            || word.starts_with("lefthook_skip")
    }) || [
        "lefthook_exclude",
        "lefthook_skip",
        "lefthook=0",
        "lefthook=false",
    ]
    .iter()
    .any(|m| lower.contains(m));
    if lefthook_off {
        return Some("the LEFTHOOK environment overrides skip Git hooks");
    }
    None
}

pub(super) fn run() -> Result<Failures> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let event: serde_json::Value =
        serde_json::from_str(&input).context("reading the hook event")?;
    let command = event["tool_input"]["command"].as_str().unwrap_or_default();
    match bypass_reason(command) {
        Some(reason) => {
            eprintln!(
                "Blocked: {reason}. Never skip hooks; fix the failure and commit again. See docs/checks/no-hook-bypass.md."
            );
            std::process::exit(2);
        }
        None => Ok(Failures::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::bypass_reason;

    #[test]
    fn blocks_hook_bypasses() {
        assert!(bypass_reason("git commit --no-verify -m \"x\"").is_some());
        assert!(bypass_reason("git commit -nm \"fix(app): x\"").is_some());
        assert!(bypass_reason("git -c core.hooksPath=/dev/null commit -m x").is_some());
        assert!(bypass_reason("LEFTHOOK=0 git commit -m x").is_some());
        assert!(bypass_reason("git push --no-verify").is_some());
        assert!(bypass_reason("git commit --no-verif -m x").is_some());
        assert!(bypass_reason("LEFTHOOK=false git commit -m x").is_some());
        assert!(bypass_reason("$env:LEFTHOOK='0'; git commit -m x").is_some());
        assert!(bypass_reason("Remove-Item .git\\hooks\\pre-commit").is_some());
        assert!(bypass_reason("del .git\\hooks\\pre-commit").is_some());
        assert!(bypass_reason("cd .git && rm -rf hooks").is_some());
        assert!(bypass_reason("git -c core.hooks\"\"Path=/dev/null commit -m x").is_some());
        assert!(bypass_reason("git config core.hooks`Path NUL").is_some());
        assert!(bypass_reason("git -c alias.c='commit --no-verify' c -m x").is_some());
        assert!(bypass_reason("lefthook uninstall").is_some());
        assert!(bypass_reason("rm .git/hooks/pre-commit").is_some());
    }

    #[test]
    fn allows_ordinary_commands_and_quoted_text() {
        assert!(
            bypass_reason(
                "git commit -m \"docs(rules): explain why -n and --no-verify are refused\""
            )
            .is_none()
        );
        assert!(bypass_reason("git commit -am \"fix(app): x\"").is_none());
        assert!(bypass_reason("cargo test -p freyya").is_none());
        assert!(bypass_reason("grep -n foo src/main.rs").is_none());
    }
}
