use std::iter::Peekable;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser as CmarkParser, Tag};

use crate::ast::{Alignment, Block, Inline, ListItem};

/// Parses Markdown source into a [`Block`] tree. Task list checkboxes are
/// numbered in document order via `task_index`, matching the order used by
/// [`crate::checkbox::toggle`] so the TUI can toggle-by-index safely.
pub fn parse(source: &str) -> Vec<Block> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut events = CmarkParser::new_ext(source, options).peekable();
    let mut task_counter = 0usize;
    parse_blocks(&mut events, &mut task_counter)
}

type Events<'a, I> = Peekable<I>;

fn is_block_start(ev: &Event) -> bool {
    matches!(
        ev,
        Event::Start(Tag::Heading { .. })
            | Event::Start(Tag::Paragraph)
            | Event::Start(Tag::BlockQuote)
            | Event::Start(Tag::CodeBlock(_))
            | Event::Start(Tag::List(_))
            | Event::Start(Tag::Table(_))
    )
}

fn parse_blocks<'a, I: Iterator<Item = Event<'a>>>(
    events: &mut Events<'a, I>,
    task_counter: &mut usize,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    loop {
        match events.peek() {
            Some(ev) if is_block_start(ev) => {
                if let Some(Event::Start(tag)) = events.next() {
                    if let Some(block) = parse_block(tag, events, task_counter) {
                        blocks.push(block);
                    }
                }
            }
            Some(Event::Rule) => {
                events.next();
                blocks.push(Block::ThematicBreak);
            }
            Some(Event::End(_)) => {
                events.next();
                break;
            }
            Some(Event::Html(_))
            | Some(Event::InlineHtml(_))
            | Some(Event::FootnoteReference(_)) => {
                events.next();
            }
            Some(_) => {
                let (inlines, consumed_end) = parse_inlines(events);
                let was_empty = inlines.is_empty();
                if !was_empty {
                    blocks.push(paragraph_or_image(inlines));
                }
                if consumed_end {
                    // The End event closing *this* container (e.g. a tight
                    // list item) was consumed while collecting the bare
                    // inline run above; our own job is done too.
                    break;
                }
                if was_empty {
                    // Nothing understood was consumed and no End either;
                    // drop the event to avoid looping forever.
                    if events.next().is_none() {
                        break;
                    }
                }
            }
            None => break,
        }
    }
    blocks
}

/// A paragraph whose only content is a single image is promoted to a
/// block-level `Block::Image` so the TUI can render it as its own element.
fn paragraph_or_image(inlines: Vec<Inline>) -> Block {
    if inlines.len() == 1 {
        if let Inline::Image { alt, url, title } = &inlines[0] {
            return Block::Image {
                alt: alt.clone(),
                url: url.clone(),
                title: title.clone(),
            };
        }
    }
    Block::Paragraph(inlines)
}

fn parse_block<'a, I: Iterator<Item = Event<'a>>>(
    tag: Tag<'a>,
    events: &mut Events<'a, I>,
    task_counter: &mut usize,
) -> Option<Block> {
    match tag {
        Tag::Heading { level, .. } => {
            let inlines = parse_inlines(events).0;
            Some(Block::Heading {
                level: level as u8,
                inlines,
            })
        }
        Tag::Paragraph => {
            let inlines = parse_inlines(events).0;
            Some(paragraph_or_image(inlines))
        }
        Tag::BlockQuote => {
            let inner = parse_blocks(events, task_counter);
            Some(Block::Blockquote(inner))
        }
        Tag::CodeBlock(kind) => {
            let lang = match kind {
                CodeBlockKind::Fenced(s) if !s.is_empty() => Some(s.to_string()),
                _ => None,
            };
            let mut code = String::new();
            loop {
                match events.next() {
                    Some(Event::Text(t)) => code.push_str(&t),
                    Some(Event::End(_)) | None => break,
                    _ => {}
                }
            }
            Some(Block::CodeBlock { lang, code })
        }
        Tag::List(start) => {
            let ordered = start.is_some();
            let mut items = Vec::new();
            loop {
                match events.peek() {
                    Some(Event::Start(Tag::Item)) => {
                        events.next();
                        let mut checked = None;
                        if let Some(Event::TaskListMarker(c)) = events.peek() {
                            checked = Some(*c);
                            events.next();
                        }
                        let content = parse_blocks(events, task_counter);
                        let task_index = if checked.is_some() {
                            let idx = *task_counter;
                            *task_counter += 1;
                            Some(idx)
                        } else {
                            None
                        };
                        items.push(ListItem {
                            content,
                            checked,
                            task_index,
                        });
                    }
                    Some(Event::End(_)) => {
                        events.next();
                        break;
                    }
                    Some(_) => {
                        events.next();
                    }
                    None => break,
                }
            }
            Some(Block::List {
                ordered,
                start,
                items,
            })
        }
        Tag::Table(alignments) => {
            let alignments = alignments.into_iter().map(convert_alignment).collect();
            let mut headers = Vec::new();
            let mut rows = Vec::new();
            loop {
                match events.peek() {
                    Some(Event::Start(Tag::TableHead)) => {
                        events.next();
                        headers = parse_table_row(events);
                    }
                    Some(Event::Start(Tag::TableRow)) => {
                        events.next();
                        rows.push(parse_table_row(events));
                    }
                    Some(Event::End(_)) => {
                        events.next();
                        break;
                    }
                    Some(_) => {
                        events.next();
                    }
                    None => break,
                }
            }
            Some(Block::Table {
                alignments,
                headers,
                rows,
            })
        }
        _ => {
            skip_unknown(events);
            None
        }
    }
}

fn parse_table_row<'a, I: Iterator<Item = Event<'a>>>(
    events: &mut Events<'a, I>,
) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();
    loop {
        match events.peek() {
            Some(Event::Start(Tag::TableCell)) => {
                events.next();
                cells.push(parse_inlines(events).0);
            }
            Some(Event::End(_)) => {
                events.next();
                break;
            }
            Some(_) => {
                events.next();
            }
            None => break,
        }
    }
    cells
}

fn convert_alignment(a: pulldown_cmark::Alignment) -> Alignment {
    match a {
        pulldown_cmark::Alignment::None => Alignment::None,
        pulldown_cmark::Alignment::Left => Alignment::Left,
        pulldown_cmark::Alignment::Center => Alignment::Center,
        pulldown_cmark::Alignment::Right => Alignment::Right,
    }
}

/// Skips a subtree we don't render (e.g. footnote definitions, HTML
/// blocks-as-tags, metadata blocks), tracking nesting depth so we don't stop
/// early on a nested container's `End`.
fn skip_unknown<'a, I: Iterator<Item = Event<'a>>>(events: &mut Events<'a, I>) {
    let mut depth = 1;
    while depth > 0 {
        match events.next() {
            Some(Event::Start(_)) => depth += 1,
            Some(Event::End(_)) => depth -= 1,
            Some(_) => {}
            None => break,
        }
    }
}

/// Collects a run of inline events. Returns the parsed inlines plus whether
/// parsing stopped because an `End` event was consumed (as opposed to
/// stopping because the next event isn't inline content at all, e.g. a
/// sibling block starting) — callers need to know which happened, since a
/// consumed `End` may belong to their *own* enclosing container (this
/// happens for "tight" list items, which have no `Paragraph` wrapper).
fn parse_inlines<'a, I: Iterator<Item = Event<'a>>>(
    events: &mut Events<'a, I>,
) -> (Vec<Inline>, bool) {
    let mut out = Vec::new();
    let mut consumed_end = false;
    loop {
        match events.peek() {
            Some(Event::Text(_)) => {
                if let Some(Event::Text(t)) = events.next() {
                    out.push(Inline::Text(t.to_string()));
                }
            }
            Some(Event::Code(_)) => {
                if let Some(Event::Code(t)) = events.next() {
                    out.push(Inline::Code(t.to_string()));
                }
            }
            Some(Event::SoftBreak) => {
                events.next();
                out.push(Inline::SoftBreak);
            }
            Some(Event::HardBreak) => {
                events.next();
                out.push(Inline::HardBreak);
            }
            Some(Event::Start(Tag::Emphasis)) => {
                events.next();
                out.push(Inline::Italic(parse_inlines(events).0));
            }
            Some(Event::Start(Tag::Strong)) => {
                events.next();
                out.push(Inline::Bold(parse_inlines(events).0));
            }
            Some(Event::Start(Tag::Strikethrough)) => {
                events.next();
                out.push(Inline::Strikethrough(parse_inlines(events).0));
            }
            Some(Event::Start(Tag::Link { .. })) => {
                if let Some(Event::Start(Tag::Link { dest_url, .. })) = events.next() {
                    let text = parse_inlines(events).0;
                    out.push(Inline::Link {
                        text,
                        url: dest_url.to_string(),
                    });
                }
            }
            Some(Event::Start(Tag::Image { .. })) => {
                if let Some(Event::Start(Tag::Image {
                    dest_url, title, ..
                })) = events.next()
                {
                    let alt_inlines = parse_inlines(events).0;
                    out.push(Inline::Image {
                        alt: crate::ast::flatten_text(&alt_inlines),
                        url: dest_url.to_string(),
                        title: title.to_string(),
                    });
                }
            }
            Some(Event::End(_)) => {
                events.next();
                consumed_end = true;
                break;
            }
            _ => break,
        }
    }
    (out, consumed_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_and_paragraph() {
        let blocks = parse("# Title\n\nHello *world*.");
        assert_eq!(
            blocks[0],
            Block::Heading {
                level: 1,
                inlines: vec![Inline::Text("Title".into())]
            }
        );
        match &blocks[1] {
            Block::Paragraph(inlines) => {
                assert_eq!(inlines[0], Inline::Text("Hello ".into()));
                assert_eq!(
                    inlines[1],
                    Inline::Italic(vec![Inline::Text("world".into())])
                );
            }
            other => panic!("expected paragraph, got {other:?}"),
        }
    }

    #[test]
    fn parses_task_list_with_sequential_indices() {
        let blocks = parse("- [ ] first\n- [x] second\n- plain\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("expected list");
        };
        assert_eq!(items[0].checked, Some(false));
        assert_eq!(items[0].task_index, Some(0));
        assert_eq!(items[1].checked, Some(true));
        assert_eq!(items[1].task_index, Some(1));
        assert_eq!(items[2].checked, None);
        assert_eq!(items[2].task_index, None);
    }

    #[test]
    fn parses_nested_list_under_tight_item() {
        let blocks = parse("- item\n  - [ ] nested task\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 1);
        // item content: implicit paragraph "item" + nested List
        assert!(items[0]
            .content
            .iter()
            .any(|b| matches!(b, Block::List { .. })));
    }

    #[test]
    fn parses_code_block_with_language() {
        let blocks = parse("```rust\nfn main() {}\n```\n");
        assert_eq!(
            blocks[0],
            Block::CodeBlock {
                lang: Some("rust".into()),
                code: "fn main() {}\n".into()
            }
        );
    }

    #[test]
    fn parses_blockquote() {
        let blocks = parse("> quoted text\n");
        match &blocks[0] {
            Block::Blockquote(inner) => {
                assert_eq!(inner.len(), 1);
            }
            other => panic!("expected blockquote, got {other:?}"),
        }
    }

    #[test]
    fn parses_thematic_break() {
        let blocks = parse("above\n\n---\n\nbelow\n");
        assert!(blocks.contains(&Block::ThematicBreak));
    }

    #[test]
    fn parses_link_and_inline_code() {
        let blocks = parse("See [docs](https://example.com) and `code`.");
        let Block::Paragraph(inlines) = &blocks[0] else {
            panic!("expected paragraph");
        };
        assert!(inlines
            .iter()
            .any(|i| matches!(i, Inline::Link { url, .. } if url == "https://example.com")));
        assert!(inlines
            .iter()
            .any(|i| matches!(i, Inline::Code(c) if c == "code")));
    }

    #[test]
    fn parses_standalone_image_as_block() {
        let blocks = parse("![alt text](pic.png)\n");
        assert_eq!(
            blocks[0],
            Block::Image {
                alt: "alt text".into(),
                url: "pic.png".into(),
                title: "".into(),
            }
        );
    }

    #[test]
    fn parses_table() {
        let blocks = parse("| a | b |\n|---|---|\n| 1 | 2 |\n");
        let Block::Table { headers, rows, .. } = &blocks[0] else {
            panic!("expected table");
        };
        assert_eq!(headers.len(), 2);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    }
}
