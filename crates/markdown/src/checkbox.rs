//! Toggling a task-list checkbox (`- [ ]` / `- [x]`) directly in the raw
//! Markdown source, by its document-order index — the same index the
//! parser assigns via `ListItem::task_index`.
//!
//! This intentionally works on the raw text rather than re-serializing the
//! AST, so toggling a checkbox can never change unrelated formatting
//! (spacing, line endings, etc.) elsewhere in the file.

/// Returns the byte offset (within `line`, excluding any line terminator)
/// of the `[` starting a task-list marker, if `line` is a list item with a
/// checkbox (`- [ ]`, `* [x]`, `1. [ ]`, ...).
fn checkbox_offset(line: &str) -> Option<usize> {
    let indent = line.len() - line.trim_start().len();
    let rest = line.as_bytes();
    let mut pos = indent;

    match rest.get(pos) {
        Some(b'-') | Some(b'*') | Some(b'+') => pos += 1,
        Some(b) if b.is_ascii_digit() => {
            let mut i = pos;
            while matches!(rest.get(i), Some(b) if b.is_ascii_digit()) {
                i += 1;
            }
            match rest.get(i) {
                Some(b'.') | Some(b')') => pos = i + 1,
                _ => return None,
            }
        }
        _ => return None,
    }

    if rest.get(pos) != Some(&b' ') {
        return None;
    }
    while rest.get(pos) == Some(&b' ') {
        pos += 1;
    }

    if rest.len() >= pos + 3
        && rest[pos] == b'['
        && matches!(rest[pos + 1], b' ' | b'x' | b'X')
        && rest[pos + 2] == b']'
    {
        Some(pos)
    } else {
        None
    }
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(stripped) = line.strip_suffix("\r\n") {
        (stripped, "\r\n")
    } else if let Some(stripped) = line.strip_suffix('\n') {
        (stripped, "\n")
    } else {
        (line, "")
    }
}

/// Flips the `task_index`-th checkbox found in `source` (document order,
/// top to bottom) and returns the updated source. Returns `None` if there
/// are fewer than `task_index + 1` checkboxes.
pub fn toggle(source: &str, task_index: usize) -> Option<String> {
    let mut count = 0usize;
    let mut result = String::with_capacity(source.len());
    let mut done = false;

    for raw_line in source.split_inclusive('\n') {
        let (content, terminator) = split_line_ending(raw_line);
        if !done {
            if let Some(offset) = checkbox_offset(content) {
                if count == task_index {
                    let checked_char = content.as_bytes()[offset + 1];
                    let flipped = if checked_char == b' ' { 'x' } else { ' ' };
                    result.push_str(&content[..offset + 1]);
                    result.push(flipped);
                    result.push_str(&content[offset + 2..]);
                    result.push_str(terminator);
                    done = true;
                    count += 1;
                    continue;
                }
                count += 1;
            }
        }
        result.push_str(raw_line);
    }

    done.then_some(result)
}

/// Counts how many task-list checkboxes exist in `source`, without
/// requiring a full parse — useful for bounds-checking before toggling.
pub fn count(source: &str) -> usize {
    source
        .split_inclusive('\n')
        .filter(|l| checkbox_offset(split_line_ending(l).0).is_some())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_unchecked_to_checked() {
        let src = "- [ ] task one\n- [x] task two\n";
        let out = toggle(src, 0).unwrap();
        assert_eq!(out, "- [x] task one\n- [x] task two\n");
    }

    #[test]
    fn toggles_checked_to_unchecked() {
        let src = "- [ ] task one\n- [x] task two\n";
        let out = toggle(src, 1).unwrap();
        assert_eq!(out, "- [ ] task one\n- [ ] task two\n");
    }

    #[test]
    fn ignores_non_task_lines() {
        let src = "plain paragraph\n- [ ] only task\n";
        let out = toggle(src, 0).unwrap();
        assert_eq!(out, "plain paragraph\n- [x] only task\n");
    }

    #[test]
    fn returns_none_for_out_of_range_index() {
        let src = "- [ ] only task\n";
        assert_eq!(toggle(src, 5), None);
    }

    #[test]
    fn handles_nested_indented_tasks() {
        let src = "- item\n  - [ ] nested\n";
        let out = toggle(src, 0).unwrap();
        assert_eq!(out, "- item\n  - [x] nested\n");
    }

    #[test]
    fn preserves_file_without_trailing_newline() {
        let src = "- [ ] last line no newline";
        let out = toggle(src, 0).unwrap();
        assert_eq!(out, "- [x] last line no newline");
    }

    #[test]
    fn count_matches_number_of_checkboxes() {
        let src = "- [ ] a\n- b\n- [x] c\n";
        assert_eq!(count(src), 2);
    }
}
