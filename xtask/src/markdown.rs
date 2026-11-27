use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

pub struct Link {
    pub target: String,
}

pub struct Placeholder {
    pub text: String,
    pub line: usize,
}

fn line_of(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())].matches('\n').count() + 1
}

fn parser(content: &str) -> Parser<'_> {
    Parser::new_ext(
        content,
        Options::ENABLE_TABLES | Options::ENABLE_HEADING_ATTRIBUTES,
    )
}

pub fn links(content: &str) -> Vec<Link> {
    let mut found = Vec::new();
    for event in parser(content) {
        if let Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) = event {
            found.push(Link {
                target: dest_url.to_string(),
            });
        }
    }
    found
}

fn scan_placeholders(text: &str, line: usize, found: &mut Vec<Placeholder>) {
    let mut rest = text;
    let mut consumed_lines = 0;
    while let Some(start) = rest.find('<') {
        consumed_lines += rest[..start].matches('\n').count();
        let after = &rest[start + 1..];
        let Some(end) = after.find('>') else {
            break;
        };
        let inner = &after[..end];
        let starts_lowercase = inner.chars().next().is_some_and(|c| c.is_ascii_lowercase());
        let is_url = inner.contains("://") || inner.starts_with("mailto:");
        if starts_lowercase && !is_url && !inner.contains('\n') {
            found.push(Placeholder {
                text: format!("<{inner}>"),
                line: line + consumed_lines,
            });
        }
        rest = &after[end + 1..];
    }
}

pub fn placeholders(content: &str) -> Vec<Placeholder> {
    let mut found = Vec::new();
    let mut template_depth = 0usize;
    for (event, range) in parser(content).into_offset_iter() {
        let line = line_of(content, range.start);
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info)))
                if info.contains("template") =>
            {
                template_depth += 1;
            }
            Event::End(TagEnd::CodeBlock) if template_depth > 0 => {
                template_depth -= 1;
            }
            Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text)
                if template_depth == 0 =>
            {
                scan_placeholders(&text, line, &mut found);
            }
            _ => {}
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{links, placeholders};

    #[test]
    fn finds_placeholder_in_text_and_inline_code() {
        let doc = "Allowed scopes: <scope list>.\n\nRun `<baseline test command>`.\n";
        let found: Vec<String> = placeholders(doc).into_iter().map(|p| p.text).collect();
        assert_eq!(found, vec!["<scope list>", "<baseline test command>"]);
    }

    #[test]
    fn ignores_template_blocks_urls_and_capitalized_tags() {
        let doc = "See <https://example.com>.\n\n```markdown template\n<main branch>\n```\n\nA <Term> here.\n";
        assert!(placeholders(doc).is_empty());
    }

    #[test]
    fn placeholder_in_unmarked_fence_still_fails() {
        let doc = "```text\n<main branch>\n```\n";
        assert_eq!(placeholders(doc).len(), 1);
    }

    #[test]
    fn collects_link_targets() {
        let doc = "# T\n\nSee [a](./a.md#x) and\n[b](b.md).\n";
        let found: Vec<String> = links(doc).into_iter().map(|l| l.target).collect();
        assert_eq!(found, vec!["./a.md#x".to_owned(), "b.md".to_owned()]);
    }
}
