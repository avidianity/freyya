use anyhow::{Context, Result};
use proc_macro2::{TokenStream, TokenTree};
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, ExprCall, ExprMacro, ItemFn, ItemMod, LitStr, Macro, Meta, StmtMacro};

#[derive(Default, Debug)]
pub struct Facts {
    pub tests: BTreeMap<String, String>,
    pub ignored_tests: usize,
    pub assertions: usize,
    pub suppressions: Vec<Suppression>,
    pub test_strings: Vec<String>,
    pub network_endpoints: usize,
    pub secret_reads: usize,
    pub disk_writes: usize,
    pub bulk_deletes: usize,
}

#[derive(Debug, Clone)]
pub struct Suppression {
    pub line: usize,
    pub lint: String,
}

#[derive(Debug)]
pub struct Comment {
    pub line: usize,
    pub text: String,
}

const DISK_WRITES: &[&[&str]] = &[
    &["fs", "write"],
    &["fs", "remove_file"],
    &["fs", "remove_dir"],
    &["fs", "remove_dir_all"],
    &["fs", "rename"],
    &["fs", "copy"],
    &["fs", "create_dir"],
    &["fs", "create_dir_all"],
    &["File", "create"],
    &["File", "create_new"],
    &["OpenOptions", "new"],
];

const SECRET_READS: &[&[&str]] = &[
    &["env", "var"],
    &["env", "var_os"],
    &["env", "vars"],
    &["env", "vars_os"],
];

const LINT_LEVELS: &[&str] = &["allow", "expect", "warn"];

fn path_ends_with(path: &syn::Path, suffix: &[&str]) -> bool {
    let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= suffix.len()
        && segments[segments.len() - suffix.len()..]
            .iter()
            .zip(suffix)
            .all(|(a, b)| a == b)
}

fn is_test_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test")
}

fn attr_tokens(attr: &Attribute) -> Option<&TokenStream> {
    match &attr.meta {
        Meta::List(list) => Some(&list.tokens),
        _ => None,
    }
}

fn stream_mentions(tokens: &TokenStream, word: &str) -> bool {
    tokens.clone().into_iter().any(|tree| match tree {
        TokenTree::Ident(ident) => ident == word,
        TokenTree::Group(group) => stream_mentions(&group.stream(), word),
        _ => false,
    })
}

fn is_cfg_test(attr: &Attribute) -> bool {
    attr.path().is_ident("cfg") && attr_tokens(attr).is_some_and(|t| stream_mentions(t, "test"))
}

fn suppressed_lint(attr: &Attribute) -> Option<String> {
    let path = attr.path();
    let tokens = attr_tokens(attr);
    let direct = LINT_LEVELS.iter().any(|level| path.is_ident(level));
    let conditional = path.is_ident("cfg_attr")
        && tokens.is_some_and(|t| LINT_LEVELS.iter().any(|level| stream_mentions(t, level)));
    if direct || conditional {
        Some(tokens.map(ToString::to_string).unwrap_or_default())
    } else {
        None
    }
}

struct Collector<'src> {
    source: &'src str,
    facts: Facts,
    test_depth: usize,
}

impl Collector<'_> {
    fn scan_macro(&mut self, mac: &Macro) {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if name.starts_with("assert")
            || name.starts_with("debug_assert")
            || name.starts_with("prop_assert")
        {
            self.facts.assertions += 1;
        }
        if name == "env" || name == "option_env" {
            self.facts.secret_reads += 1;
        }
        self.scan_literals(&mac.tokens);
    }

    fn scan_literals(&mut self, tokens: &TokenStream) {
        for tree in tokens.clone() {
            match tree {
                TokenTree::Literal(literal) => {
                    if let Ok(lit) = syn::parse_str::<LitStr>(&literal.to_string()) {
                        self.record_string(&lit.value());
                    }
                }
                TokenTree::Group(group) => self.scan_literals(&group.stream()),
                _ => {}
            }
        }
    }

    fn record_string(&mut self, value: &str) {
        let lower = value.to_lowercase();
        if ["http://", "https://", "ws://", "wss://"]
            .iter()
            .any(|scheme| lower.starts_with(scheme))
        {
            self.facts.network_endpoints += 1;
        }
        let compact = lower.split_whitespace().collect::<Vec<_>>().join(" ");
        if ["drop table", "truncate table", "delete from"]
            .iter()
            .any(|op| compact.contains(op))
        {
            self.facts.bulk_deletes += 1;
        }
        if self.test_depth > 0 {
            self.facts.test_strings.push(value.to_owned());
        }
    }

    fn text_of(&self, node: &impl Spanned) -> String {
        let range = node.span().byte_range();
        self.source.get(range).unwrap_or_default().to_owned()
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if let Some(lint) = suppressed_lint(attr) {
            self.facts.suppressions.push(Suppression {
                line: attr.span().start().line,
                lint,
            });
        }
        visit::visit_attribute(self, attr);
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if item.attrs.iter().any(is_test_attr) {
            let body = self.text_of(item);
            self.facts.tests.insert(item.sig.ident.to_string(), body);
            if item.attrs.iter().any(|a| a.path().is_ident("ignore")) {
                self.facts.ignored_tests += 1;
            }
            self.test_depth += 1;
            visit::visit_item_fn(self, item);
            self.test_depth -= 1;
        } else {
            visit::visit_item_fn(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let is_test_mod = item.attrs.iter().any(is_cfg_test);
        self.test_depth += usize::from(is_test_mod);
        visit::visit_item_mod(self, item);
        self.test_depth -= usize::from(is_test_mod);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            if DISK_WRITES
                .iter()
                .any(|suffix| path_ends_with(&path.path, suffix))
            {
                self.facts.disk_writes += 1;
            }
            if SECRET_READS
                .iter()
                .any(|suffix| path_ends_with(&path.path, suffix))
            {
                self.facts.secret_reads += 1;
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_macro(&mut self, mac: &'ast ExprMacro) {
        self.scan_macro(&mac.mac);
        visit::visit_expr_macro(self, mac);
    }

    fn visit_stmt_macro(&mut self, mac: &'ast StmtMacro) {
        self.scan_macro(&mac.mac);
        visit::visit_stmt_macro(self, mac);
    }

    fn visit_lit_str(&mut self, lit: &'ast LitStr) {
        self.record_string(&lit.value());
        visit::visit_lit_str(self, lit);
    }
}

pub fn facts(source: &str) -> Result<Facts> {
    let file = syn::parse_file(source).context("parsing Rust source")?;
    let mut collector = Collector {
        source,
        facts: Facts::default(),
        test_depth: 0,
    };
    collector.visit_file(&file);
    Ok(collector.facts)
}

pub fn comments(source: &str) -> Vec<Comment> {
    let skip = ra_ap_rustc_lexer::strip_shebang(source).unwrap_or(0);
    let mut found = Vec::new();
    let mut offset = skip;
    for token in tokenize(&source[skip..], FrontmatterAllowed::Yes) {
        let len = token.len as usize;
        if matches!(
            token.kind,
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            found.push(Comment {
                line: source[..offset].matches('\n').count() + 1,
                text: source[offset..offset + len].to_owned(),
            });
        }
        offset += len;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{comments, facts};

    #[test]
    fn counts_tests_ignores_and_assertions() {
        let src = "#[cfg(test)]\nmod t {\n    #[test]\n    fn a() { assert_eq!(1, 1); }\n    #[test]\n    #[ignore]\n    fn b() { assert!(true); }\n}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.tests.len(), 2);
        assert_eq!(found.ignored_tests, 1);
        assert_eq!(found.assertions, 2);
    }

    #[test]
    fn finds_inner_outer_and_conditional_suppressions() {
        let src = "#![allow(dead_code)]\n#[expect(clippy::all)]\nfn a() {}\n#[cfg_attr(test, allow(unused))]\nfn b() {}\n";
        let lines: Vec<usize> = facts(src)
            .unwrap_or_default()
            .suppressions
            .iter()
            .map(|s| s.line)
            .collect();
        assert_eq!(lines, vec![1, 2, 4]);
    }

    #[test]
    fn deny_is_not_a_suppression() {
        let src = "#![deny(warnings)]\nfn a() {}\n";
        assert!(facts(src).unwrap_or_default().suppressions.is_empty());
    }

    #[test]
    fn detects_risk_signals() {
        let src = "fn a() {\n    let _ = std::env::var(\"KEY\");\n    let _ = std::fs::write(\"p\", \"x\");\n    let _ = format!(\"https://api.example.com/{}\", 1);\n    let _ = \"DELETE FROM snapshots\";\n}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.secret_reads, 1);
        assert_eq!(found.disk_writes, 1);
        assert_eq!(found.network_endpoints, 1);
        assert_eq!(found.bulk_deletes, 1);
    }

    #[test]
    fn comments_come_from_the_lexer_not_from_strings() {
        let src = "fn a() {\n    let url = \"https://x.dev/*not*/ // nope\";\n    // real\n    let _ = url; /* block */\n}\n";
        let found: Vec<(usize, String)> = comments(src)
            .into_iter()
            .map(|c| (c.line, c.text))
            .collect();
        assert_eq!(
            found,
            vec![(3, "// real".to_owned()), (4, "/* block */".to_owned())]
        );
    }
}
