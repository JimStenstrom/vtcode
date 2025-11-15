//! Markdown parsing and event normalization.
//!
//! This module handles converting markdown text into a normalized event stream
//! that can be processed by the rendering engine.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use tracing::warn;

/// Parse markdown text into normalized events.
pub fn parse_markdown(text: &str) -> Vec<MarkdownEvent> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(text, options);
    collect_markdown_events(parser)
}

/// Normalize pulldown_cmark events to internal format.
fn collect_markdown_events<'a>(parser: Parser<'a>) -> Vec<MarkdownEvent> {
    parser
        .map(|event| {
            #[allow(unreachable_patterns)]
            match event {
                Event::Start(tag) => MarkdownEvent::Start(tag.into()),
                Event::End(tag_end) => MarkdownEvent::End(tag_end.into()),
                Event::Text(text) => MarkdownEvent::Text(text.into_string()),
                Event::Code(code) => MarkdownEvent::Code(code.into_string()),
                Event::Html(html) => MarkdownEvent::Html(html.into_string()),
                Event::FootnoteReference(ref_str) => {
                    MarkdownEvent::FootnoteReference(ref_str.into_string())
                }
                Event::SoftBreak => MarkdownEvent::SoftBreak,
                Event::HardBreak => MarkdownEvent::HardBreak,
                Event::Rule => MarkdownEvent::Rule,
                Event::TaskListMarker(checked) => MarkdownEvent::TaskListMarker(checked),
                Event::InlineHtml(html) => MarkdownEvent::Html(html.into_string()),
                Event::InlineMath(math) => MarkdownEvent::Text(format!("${}$", math.into_string())),
                Event::DisplayMath(math) => {
                    MarkdownEvent::Text(format!("$$\n{}\n$$", math.into_string()))
                }
                other => {
                    warn!(?other, "Unhandled pulldown-cmark event variant");
                    MarkdownEvent::Text(String::new())
                }
            }
        })
        .collect()
}

/// Internal markdown event representation.
#[derive(Clone, Debug)]
pub enum MarkdownEvent {
    Start(MarkdownTag),
    End(MarkdownTag),
    Text(String),
    Code(String),
    Html(String),
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker(bool),
    FootnoteReference(String),
}

/// Markdown tag types.
#[derive(Clone, Debug)]
pub enum MarkdownTag {
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    List(Option<usize>),
    Item,
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    CodeBlock(CodeBlockKind),
    Table,
    TableHead,
    TableRow,
    TableCell,
    FootnoteDefinition,
    HtmlBlock,
}

impl From<Tag<'_>> for MarkdownTag {
    fn from(tag: Tag) -> Self {
        match tag {
            Tag::Paragraph => MarkdownTag::Paragraph,
            Tag::Heading { level, .. } => MarkdownTag::Heading(heading_level_from_u8(level as u8)),
            Tag::BlockQuote(_) => MarkdownTag::BlockQuote,
            Tag::CodeBlock(kind) => MarkdownTag::CodeBlock(kind.into()),
            Tag::HtmlBlock => MarkdownTag::HtmlBlock,
            Tag::List(start) => MarkdownTag::List(start.map(|n| n as usize)),
            Tag::Item => MarkdownTag::Item,
            Tag::FootnoteDefinition(_) => MarkdownTag::FootnoteDefinition,
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {
                MarkdownTag::Paragraph
            }
            Tag::Table(_) => MarkdownTag::Table,
            Tag::TableHead => MarkdownTag::TableHead,
            Tag::TableRow => MarkdownTag::TableRow,
            Tag::TableCell => MarkdownTag::TableCell,
            Tag::Emphasis => MarkdownTag::Emphasis,
            Tag::Strong => MarkdownTag::Strong,
            Tag::Strikethrough => MarkdownTag::Strikethrough,
            Tag::Superscript | Tag::Subscript => MarkdownTag::Emphasis,
            Tag::Link { .. } => MarkdownTag::Link,
            Tag::Image { .. } => MarkdownTag::Image,
            Tag::MetadataBlock(_) => MarkdownTag::Paragraph, // fallback
        }
    }
}

impl From<TagEnd> for MarkdownTag {
    fn from(tag_end: TagEnd) -> Self {
        match tag_end {
            TagEnd::Paragraph => MarkdownTag::Paragraph,
            TagEnd::Heading(level) => MarkdownTag::Heading(heading_level_from_u8(level as u8)),
            TagEnd::BlockQuote(_) => MarkdownTag::BlockQuote,
            TagEnd::CodeBlock => MarkdownTag::CodeBlock(CodeBlockKind::Indented), // doesn't matter for end
            TagEnd::HtmlBlock => MarkdownTag::HtmlBlock,
            TagEnd::List(_) => MarkdownTag::List(None), // doesn't matter for end
            TagEnd::Item => MarkdownTag::Item,
            TagEnd::FootnoteDefinition => MarkdownTag::FootnoteDefinition,
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => MarkdownTag::Paragraph,
            TagEnd::Table => MarkdownTag::Table,
            TagEnd::TableHead => MarkdownTag::TableHead,
            TagEnd::TableRow => MarkdownTag::TableRow,
            TagEnd::TableCell => MarkdownTag::TableCell,
            TagEnd::Emphasis => MarkdownTag::Emphasis,
            TagEnd::Strong => MarkdownTag::Strong,
            TagEnd::Strikethrough => MarkdownTag::Strikethrough,
            TagEnd::Superscript | TagEnd::Subscript => MarkdownTag::Emphasis,
            TagEnd::Link => MarkdownTag::Link,
            TagEnd::Image => MarkdownTag::Image,
            TagEnd::MetadataBlock(_) => MarkdownTag::Paragraph, // fallback
        }
    }
}

/// Code block kind.
#[derive(Clone, Debug)]
pub enum CodeBlockKind {
    Fenced(String),
    Indented,
}

impl From<pulldown_cmark::CodeBlockKind<'_>> for CodeBlockKind {
    fn from(kind: pulldown_cmark::CodeBlockKind) -> Self {
        match kind {
            pulldown_cmark::CodeBlockKind::Fenced(info) => {
                CodeBlockKind::Fenced(info.into_string())
            }
            pulldown_cmark::CodeBlockKind::Indented => CodeBlockKind::Indented,
        }
    }
}

/// Heading level.
#[derive(Clone, Copy, Debug)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

fn heading_level_from_u8(level: u8) -> HeadingLevel {
    match level {
        1 => HeadingLevel::H1,
        2 => HeadingLevel::H2,
        3 => HeadingLevel::H3,
        4 => HeadingLevel::H4,
        5 => HeadingLevel::H5,
        _ => HeadingLevel::H6,
    }
}
