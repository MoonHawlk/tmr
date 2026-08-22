//! A renderer-agnostic Markdown document tree. Any frontend (the TUI today,
//! conceivably something else tomorrow) walks this instead of dealing with
//! `pulldown_cmark` events directly.

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link {
        text: Vec<Inline>,
        url: String,
    },
    Image {
        alt: String,
        url: String,
        title: String,
    },
    SoftBreak,
    HardBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

/// One `- [ ] ...` / `- ...` list entry. `checked` is `Some` only for task
/// list items (`None` for a plain list item).
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub content: Vec<Block>,
    pub checked: Option<bool>,
    /// Index into the document's flat task-item order (see
    /// [`crate::checkbox`]), set only when `checked.is_some()`.
    pub task_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    List {
        ordered: bool,
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    Blockquote(Vec<Block>),
    Table {
        alignments: Vec<Alignment>,
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Image {
        alt: String,
        url: String,
        title: String,
    },
    ThematicBreak,
}

/// Flattens an inline sequence down to plain text (used e.g. for image alt
/// text, which the AST stores as a string rather than nested inlines).
pub fn flatten_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for i in inlines {
        match i {
            Inline::Text(t) | Inline::Code(t) => out.push_str(t),
            Inline::Bold(inner) | Inline::Italic(inner) | Inline::Strikethrough(inner) => {
                out.push_str(&flatten_text(inner))
            }
            Inline::Link { text, .. } => out.push_str(&flatten_text(text)),
            Inline::Image { alt, .. } => out.push_str(alt),
            Inline::SoftBreak => out.push(' '),
            Inline::HardBreak => out.push('\n'),
        }
    }
    out
}
