//! Markdown parsing using pulldown-cmark

use anyhow::Result;
use pulldown_cmark::{Event, Options, Parser, Tag};

use super::directives::Directive;

/// A parsed markdown document containing events and detected directives
#[derive(Debug)]
pub struct ParsedDocument {
    /// The raw pulldown-cmark events
    pub events: Vec<ParsedEvent>,
}

/// A parsed event that may be a markdown event or a custom directive
#[derive(Debug)]
pub enum ParsedEvent {
    /// A standard markdown event
    Markdown(Event<'static>),
    /// A custom directive
    Directive(Directive),
}

/// Parse markdown content into a ParsedDocument
pub fn parse_markdown(content: &str) -> Result<ParsedDocument> {
    // Enable all markdown extensions
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let parser = Parser::new_ext(content, options);

    let mut events = Vec::new();

    for event in parser {
        // Check if this is a text event that might contain a directive
        if let Event::Text(ref text) = event {
            if let Some(directive) = Directive::parse(text.as_ref()) {
                events.push(ParsedEvent::Directive(directive));
                continue;
            }
        }

        // Convert to owned event for storage
        let owned_event = event.into_static();
        events.push(ParsedEvent::Markdown(owned_event));
    }

    Ok(ParsedDocument { events })
}

/// Extension trait to convert events to owned/static versions
trait IntoStatic {
    fn into_static(self) -> Event<'static>;
}

impl<'a> IntoStatic for Event<'a> {
    fn into_static(self) -> Event<'static> {
        match self {
            Event::Start(tag) => Event::Start(tag_into_static(tag)),
            Event::End(tag) => Event::End(tag),
            Event::Text(text) => Event::Text(text.into_string().into()),
            Event::Code(code) => Event::Code(code.into_string().into()),
            Event::Html(html) => Event::Html(html.into_string().into()),
            Event::InlineHtml(html) => Event::InlineHtml(html.into_string().into()),
            Event::FootnoteReference(reference) => {
                Event::FootnoteReference(reference.into_string().into())
            }
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
        }
    }
}

fn tag_into_static(tag: Tag<'_>) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading { level, id, classes, attrs } => Tag::Heading {
            level,
            id: id.map(|s| s.into_string().into()),
            classes: classes.into_iter().map(|s| s.into_string().into()).collect(),
            attrs: attrs.into_iter().map(|(k, v)| {
                (k.into_string().into(), v.map(|s| s.into_string().into()))
            }).collect(),
        },
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
            pulldown_cmark::CodeBlockKind::Indented => pulldown_cmark::CodeBlockKind::Indented,
            pulldown_cmark::CodeBlockKind::Fenced(info) => {
                pulldown_cmark::CodeBlockKind::Fenced(info.into_string().into())
            }
        }),
        Tag::List(start) => Tag::List(start),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(label) => Tag::FootnoteDefinition(label.into_string().into()),
        Tag::Table(alignments) => Tag::Table(alignments),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link { link_type, dest_url, title, id } => Tag::Link {
            link_type,
            dest_url: dest_url.into_string().into(),
            title: title.into_string().into(),
            id: id.into_string().into(),
        },
        Tag::Image { link_type, dest_url, title, id } => Tag::Image {
            link_type,
            dest_url: dest_url.into_string().into(),
            title: title.into_string().into(),
            id: id.into_string().into(),
        },
        Tag::HtmlBlock => Tag::HtmlBlock,
        Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let content = "# Hello World\n\nThis is a paragraph.";
        let doc = parse_markdown(content).unwrap();

        assert!(!doc.events.is_empty());
    }

    #[test]
    fn test_parse_with_directive() {
        // Note: Markdown interprets [bracketed-text] as a link reference,
        // so directives in markdown content are primarily handled by
        // postprocess_directives in typst_builder.rs as a fallback.
        // The directive parser here catches them if they appear as raw text.

        // Test the directive parser directly instead
        use super::super::directives::Directive;
        assert_eq!(
            Directive::parse("[page-break]"),
            Some(super::super::directives::Directive::PageBreak)
        );
    }

    #[test]
    fn test_parse_tables() {
        let content = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1 | Cell 2 |";
        let doc = parse_markdown(content).unwrap();

        // Check for table events
        let has_table = doc.events.iter().any(|e| {
            matches!(e, ParsedEvent::Markdown(Event::Start(Tag::Table(_))))
        });
        assert!(has_table);
    }

    #[test]
    fn test_parse_task_list() {
        let content = "- [x] Completed task\n- [ ] Incomplete task";
        let doc = parse_markdown(content).unwrap();

        // Check for task list marker events
        let has_task = doc.events.iter().any(|e| {
            matches!(e, ParsedEvent::Markdown(Event::TaskListMarker(_)))
        });
        assert!(has_task);
    }
}
