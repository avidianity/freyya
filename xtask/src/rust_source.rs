use anyhow::{Context, Result, anyhow};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Ident, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemMod, ItemStatic, ItemStruct,
    ItemTrait, ItemType, ItemUnion, LitStr, Macro, Meta, UseTree,
};

#[derive(Default, Debug)]
pub(crate) struct Facts {
    pub tests: BTreeMap<String, String>,
    pub ignored_tests: usize,
    pub assertions: usize,
    pub suppressions: Vec<Suppression>,
    pub test_strings: Vec<String>,
    pub all_strings: Vec<String>,
    pub doc_attribute_lines: Vec<usize>,
    pub network_endpoints: usize,
    pub secret_reads: usize,
    pub disk_writes: usize,
    pub bulk_deletes: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Suppression {
    pub line: usize,
    pub lint: String,
}

#[derive(Debug)]
pub(crate) struct Comment {
    pub line: usize,
    pub text: String,
}

const LINT_LEVELS: &[&str] = &["allow", "expect", "warn"];

const FS_READS: &[&str] = &[
    "read",
    "read_to_string",
    "read_dir",
    "metadata",
    "symlink_metadata",
    "canonicalize",
    "read_link",
    "exists",
];

const NETWORK_CRATES: &[&str] = &[
    "reqwest",
    "ureq",
    "hyper",
    "surf",
    "isahc",
    "attohttpc",
    "curl",
    "tungstenite",
];

const SECRET_FUNCTIONS: &[&str] = &["var", "var_os", "vars", "vars_os"];

fn is_secret_read(segments: &[String]) -> bool {
    segments.windows(2).any(|pair| {
        pair[0] == "env"
            && (SECRET_FUNCTIONS.contains(&pair[1].as_str()) || pair[1] == "*" || pair[1] == "self")
    })
}

fn is_disk_write(segments: &[String]) -> bool {
    let fs_write = segments
        .windows(2)
        .any(|pair| pair[0] == "fs" && pair[1] != "File" && !FS_READS.contains(&pair[1].as_str()));
    let fs_glob = segments.last().is_some_and(|s| s == "*") && segments.iter().any(|s| s == "fs");
    let file_write = segments.windows(2).any(|pair| {
        pair[0] == "File" && ["create", "create_new", "options"].contains(&pair[1].as_str())
    });
    let open_options = segments.iter().any(|s| s == "OpenOptions");
    fs_write || fs_glob || file_write || open_options
}

fn is_network(segments: &[String]) -> bool {
    let first = segments.first().map(String::as_str).unwrap_or_default();
    let std_net = segments
        .windows(2)
        .any(|pair| ["std", "tokio", "async_std"].contains(&pair[0].as_str()) && pair[1] == "net");
    NETWORK_CRATES.contains(&first) || std_net
}

fn is_test_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test")
}

fn stream_mentions(tokens: &TokenStream, word: &str) -> bool {
    tokens.clone().into_iter().any(|tree| match tree {
        TokenTree::Ident(ident) => ident == word,
        TokenTree::Group(group) => stream_mentions(&group.stream(), word),
        _ => false,
    })
}

fn is_cfg_test(attr: &Attribute) -> bool {
    let Meta::List(list) = &attr.meta else {
        return false;
    };
    let only_test = {
        let trees: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
        matches!(trees.as_slice(), [TokenTree::Ident(ident)] if ident == "test")
    };
    attr.path().is_ident("cfg") && only_test
}

fn skips_test(attr: &Attribute) -> bool {
    let path = attr.path();
    let conditional_ignore = path.is_ident("cfg_attr")
        && matches!(&attr.meta, Meta::List(list) if stream_mentions(&list.tokens, "ignore"));
    path.is_ident("ignore") || conditional_ignore || (path.is_ident("cfg") && !is_cfg_test(attr))
}

fn attribute_suppression(body: &TokenStream) -> Option<String> {
    let mut trees = body.clone().into_iter();
    let Some(TokenTree::Ident(name)) = trees.next() else {
        return None;
    };
    let rest: TokenStream = trees.collect();
    let direct = LINT_LEVELS.iter().any(|level| name == level);
    let conditional = name == "cfg_attr"
        && LINT_LEVELS
            .iter()
            .any(|level| stream_mentions(&rest, level));
    (direct || conditional).then(|| format!("{name}{rest}"))
}

fn is_doc_attribute(body: &TokenStream) -> bool {
    match body.clone().into_iter().next() {
        Some(TokenTree::Ident(name)) if name == "doc" => true,
        Some(TokenTree::Ident(name)) if name == "cfg_attr" => stream_mentions(body, "doc"),
        _ => false,
    }
}

fn is_macro_built_attribute(body: &TokenStream) -> bool {
    matches!(body.clone().into_iter().next(), Some(TokenTree::Punct(p)) if p.as_char() == '$')
}

fn scan_token_attributes(tokens: TokenStream, found: &mut Vec<Suppression>, docs: &mut Vec<usize>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        match tree {
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                let mut next = index + 1;
                if matches!(trees.get(next), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                    next += 1;
                }
                let Some(TokenTree::Group(group)) = trees.get(next) else {
                    continue;
                };
                if group.delimiter() != Delimiter::Bracket {
                    continue;
                }
                let line = punct.span().start().line;
                if let Some(lint) = attribute_suppression(&group.stream()) {
                    found.push(Suppression { line, lint });
                }
                if is_macro_built_attribute(&group.stream()) {
                    found.push(Suppression {
                        line,
                        lint: "an attribute built from a macro argument".to_owned(),
                    });
                }
                if is_doc_attribute(&group.stream()) {
                    docs.push(line);
                }
            }
            TokenTree::Group(group) => scan_token_attributes(group.stream(), found, docs),
            _ => {}
        }
    }
}

fn use_paths(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            use_paths(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let mut full = prefix.clone();
            full.push(name.ident.to_string());
            out.push(full);
        }
        UseTree::Rename(rename) => {
            let mut full = prefix.clone();
            full.push(rename.ident.to_string());
            out.push(full);
        }
        UseTree::Glob(_) => {
            let mut full = prefix.clone();
            full.push("*".to_owned());
            out.push(full);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                use_paths(item, prefix, out);
            }
        }
    }
}

struct Collector<'src> {
    source: &'src str,
    facts: Facts,
    test_depth: usize,
}

impl Collector<'_> {
    fn in_test(&self) -> bool {
        self.test_depth > 0
    }

    fn record_segments(&mut self, segments: &[String]) {
        if self.in_test() {
            return;
        }
        if is_secret_read(segments) {
            self.facts.secret_reads += 1;
        }
        if is_disk_write(segments) {
            self.facts.disk_writes += 1;
        }
        if is_network(segments) {
            self.facts.network_endpoints += 1;
        }
    }

    fn record_string(&mut self, value: &str) {
        self.facts.all_strings.push(value.to_owned());
        if self.in_test() {
            self.facts.test_strings.push(value.to_owned());
            return;
        }
        let lower = value.to_lowercase();
        let scheme_only = ["http", "https", "ws", "wss"].contains(&lower.trim());
        if lower.contains("://") || scheme_only {
            self.facts.network_endpoints += 1;
        }
        let compact = lower.split_whitespace().collect::<Vec<_>>().join(" ");
        if ["drop table", "truncate table", "delete from"]
            .iter()
            .any(|op| compact.contains(op))
        {
            self.facts.bulk_deletes += 1;
        }
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

    fn record_name(&mut self, ident: &Ident) {
        let name = ident.to_string();
        if name.starts_with('_') && name != "_" {
            self.facts.suppressions.push(Suppression {
                line: ident.span().start().line,
                lint: format!("dead_code by the leading underscore in `{name}`"),
            });
        }
    }

    fn text_of(&self, node: &impl Spanned) -> String {
        let range = node.span().byte_range();
        self.source.get(range).unwrap_or_default().to_owned()
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.record_name(&item.sig.ident);
        if item.attrs.iter().any(is_test_attr) {
            let body = self.text_of(item);
            self.facts.tests.insert(item.sig.ident.to_string(), body);
            if item.attrs.iter().any(skips_test) {
                self.facts.ignored_tests += 1;
            }
            self.test_depth += 1;
            visit::visit_item_fn(self, item);
            self.test_depth -= 1;
        } else {
            visit::visit_item_fn(self, item);
        }
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.record_name(&item.sig.ident);
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.record_name(&item.ident);
        visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.record_name(&item.ident);
        visit::visit_item_static(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.record_name(&item.ident);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.record_name(&item.ident);
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.record_name(&item.ident);
        visit::visit_item_union(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.record_name(&item.ident);
        visit::visit_item_trait(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.record_name(&item.ident);
        visit::visit_item_type(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let is_test_mod = item.attrs.iter().any(is_cfg_test);
        self.test_depth += usize::from(is_test_mod);
        visit::visit_item_mod(self, item);
        self.test_depth -= usize::from(is_test_mod);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        let mut paths = Vec::new();
        use_paths(&item.tree, &mut Vec::new(), &mut paths);
        for path in paths {
            self.record_segments(&path);
        }
        visit::visit_item_use(self, item);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        self.record_segments(&segments);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if ["assert", "debug_assert", "prop_assert"]
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            self.facts.assertions += 1;
        }
        if (name == "env" || name == "option_env") && !self.in_test() {
            self.facts.secret_reads += 1;
        }
        self.scan_literals(&mac.tokens);
        visit::visit_macro(self, mac);
    }

    fn visit_lit_str(&mut self, lit: &'ast LitStr) {
        self.record_string(&lit.value());
        visit::visit_lit_str(self, lit);
    }
}

pub(crate) fn facts(source: &str) -> Result<Facts> {
    let file = syn::parse_file(source).context("parsing Rust source")?;
    let mut collector = Collector {
        source,
        facts: Facts::default(),
        test_depth: 0,
    };
    collector.visit_file(&file);
    let tokens: TokenStream = source
        .parse()
        .map_err(|error| anyhow!("tokenizing Rust source: {error}"))?;
    let mut suppressions = Vec::new();
    let mut doc_lines = Vec::new();
    scan_token_attributes(tokens, &mut suppressions, &mut doc_lines);
    collector.facts.doc_attribute_lines = doc_lines;
    suppressions.extend(collector.facts.suppressions);
    suppressions.sort_by_key(|s| s.line);
    collector.facts.suppressions = suppressions;
    Ok(collector.facts)
}

pub(crate) fn comments(source: &str) -> Vec<Comment> {
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

    fn lines(src: &str) -> Vec<usize> {
        facts(src)
            .unwrap_or_default()
            .suppressions
            .iter()
            .map(|s| s.line)
            .collect()
    }

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
        assert_eq!(lines(src), vec![1, 2, 4]);
    }

    #[test]
    fn finds_suppressions_inside_macros() {
        let src = "macro_rules! quiet {\n    ($i:item) => {\n        #[allow(dead_code)]\n        $i\n    };\n}\nquiet! { fn never_called() {} }\n";
        assert_eq!(lines(src), vec![3]);
    }

    #[test]
    fn cfg_not_test_is_production_code() {
        let src = "#[cfg(not(test))]\nmod m {\n    pub(crate) fn f() -> bool {\n        std::env::var(\"K\").is_ok()\n    }\n}\n#[cfg(any(test, unix))]\nfn g() {\n    let _ = std::fs::remove_file(\"x\");\n}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.secret_reads, 1);
        assert_eq!(found.disk_writes, 1);
    }

    #[test]
    fn conditional_skips_count_as_skipped_tests() {
        let src = "#[test]\n#[cfg_attr(all(), ignore)]\nfn a() {}\n#[test]\n#[cfg(any())]\nfn b() {}\n#[test]\nfn c() {}\n";
        assert_eq!(facts(src).unwrap_or_default().ignored_tests, 2);
    }

    #[test]
    fn macro_built_attributes_and_conditional_docs_are_found() {
        let src = "macro_rules! lvl {\n    ($l:ident) => {\n        #[$l(dead_code)]\n        fn z() {}\n    };\n}\nlvl!(allow);\n#[cfg_attr(all(), doc = \"narration\")]\nfn y() {}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.suppressions.len(), 1);
        assert_eq!(found.doc_attribute_lines, vec![8]);
    }

    #[test]
    fn env_globs_and_self_aliases_count() {
        let found =
            facts("use std::env::*;\nuse std::env::{self as e};\nfn a() {}\n").unwrap_or_default();
        assert_eq!(found.secret_reads, 2);
    }

    #[test]
    fn leading_underscore_names_count_as_suppressions() {
        assert_eq!(lines("fn _never_called() {}\nconst _: () = ();\n"), vec![1]);
    }

    #[test]
    fn deny_is_not_a_suppression() {
        assert!(lines("#![deny(warnings)]\nfn a() {}\n").is_empty());
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
    fn aliases_and_builders_still_count() {
        let src = "use std::env::var as read_setting;\nuse std::net::TcpStream;\nfn a() {\n    let _ = read_setting(\"API_TOKEN\");\n    let _ = std::fs::File::options().write(true).open(\"x\");\n    let _ = format!(\"{}://{}\", \"https\", \"api.example.com\");\n}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.secret_reads, 1);
        assert_eq!(found.disk_writes, 1);
        assert!(found.network_endpoints >= 2);
    }

    #[test]
    fn reads_and_test_code_raise_no_signal() {
        let src = "fn a() {\n    let _ = std::fs::read_to_string(\"save.v3\");\n}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn b() {\n        let _ = std::env::var(\"X\");\n        let _ = \"DELETE FROM snapshots\";\n    }\n}\n";
        let found = facts(src).unwrap_or_default();
        assert_eq!(found.disk_writes, 0);
        assert_eq!(found.secret_reads, 0);
        assert_eq!(found.bulk_deletes, 0);
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
