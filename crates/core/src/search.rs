use crate::workspace::Entry;

/// A single line match found while searching within a document's content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMatch {
    pub line_number: usize,
    pub line: String,
}

/// Filters directory entries by a case-insensitive substring match on name.
/// Prepared to be reused by a future recursive/global search without
/// changing its signature.
pub fn search_filenames<'a>(entries: &'a [Entry], query: &str) -> Vec<&'a Entry> {
    if query.is_empty() {
        return entries.iter().collect();
    }
    let query = query.to_lowercase();
    entries
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&query))
        .collect()
}

/// Finds every line in `content` containing `query` (case-insensitive).
pub fn search_in_text(content: &str, query: &str) -> Vec<LineMatch> {
    if query.is_empty() {
        return Vec::new();
    }
    let query = query.to_lowercase();
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&query))
        .map(|(idx, line)| LineMatch {
            line_number: idx + 1,
            line: line.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn filename_search_is_case_insensitive() {
        let entries = vec![
            Entry {
                name: "TODO.md".into(),
                path: PathBuf::from("TODO.md"),
                is_dir: false,
            },
            Entry {
                name: "readme.md".into(),
                path: PathBuf::from("readme.md"),
                is_dir: false,
            },
        ];
        let results = search_filenames(&entries, "todo");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "TODO.md");
    }

    #[test]
    fn text_search_returns_line_numbers() {
        let content = "line one\nsecond LINE\nthird";
        let results = search_in_text(content, "line");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 2);
    }
}
