use std::path::{Path, PathBuf};

/// The format of a document. Only `Markdown` is fully rendered today; the
/// others exist so the rest of the engine (document model, save/open flow,
/// commands) never has to special-case "just markdown" and can grow new
/// formats without touching this type's callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Markdown,
    PlainText,
    Unknown,
}

impl DocumentFormat {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("md") | Some("markdown") => DocumentFormat::Markdown,
            Some("txt") => DocumentFormat::PlainText,
            _ => DocumentFormat::Unknown,
        }
    }
}

/// The currently open document: its path, raw source text and dirty state.
///
/// `Document` intentionally holds only raw text. Parsing into a renderable
/// structure is the job of a format-specific renderer crate (e.g.
/// `tmr-markdown`), keeping the core ignorant of any particular markup.
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub format: DocumentFormat,
    pub content: String,
    dirty: bool,
}

impl Document {
    pub fn new(path: PathBuf, content: String) -> Self {
        let format = DocumentFormat::from_path(&path);
        Document {
            path,
            format,
            content,
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn set_content(&mut self, content: String) {
        if content != self.content {
            self.content = content;
            self.dirty = true;
        }
    }

    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_markdown_format() {
        assert_eq!(
            DocumentFormat::from_path(Path::new("notes/todo.md")),
            DocumentFormat::Markdown
        );
    }

    #[test]
    fn detects_unknown_format() {
        assert_eq!(
            DocumentFormat::from_path(Path::new("data.bin")),
            DocumentFormat::Unknown
        );
    }

    #[test]
    fn set_content_marks_dirty_only_on_change() {
        let mut doc = Document::new(PathBuf::from("a.md"), "hello".into());
        assert!(!doc.is_dirty());
        doc.set_content("hello".into());
        assert!(!doc.is_dirty());
        doc.set_content("hello world".into());
        assert!(doc.is_dirty());
    }
}
