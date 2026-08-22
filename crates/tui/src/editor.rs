//! A minimal, dependency-free multi-line text buffer for the Edit mode.
//!
//! This is deliberately small (insert, delete, arrow navigation) rather
//! than a full editor — tmr is not trying to be Vim. It exists so the MVP
//! can edit Markdown in place; see the README roadmap for "external editor"
//! as a possible future alternative for power users.
pub struct Editor {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    scroll: usize,
    tab_width: usize,
    dirty: bool,
    /// Where a Shift+navigation selection started (row, col), if one is in
    /// progress — the other end is always the live cursor position. `None`
    /// means no selection.
    selection_anchor: Option<(usize, usize)>,
}

fn char_to_byte(line: &str, char_idx: usize) -> usize {
    line.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(line.len())
}

impl Editor {
    pub fn new(content: &str, tab_width: usize) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.split('\n').map(String::from).collect()
        };
        Editor {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll: 0,
            tab_width: tab_width.max(1),
            dirty: false,
            selection_anchor: None,
        }
    }

    pub fn to_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// The selected range as `(start, end)` (row, col) pairs, normalized so
    /// `start <= end` regardless of which direction the selection was
    /// extended in. `None` if there's no anchor, or the anchor and cursor
    /// coincide (Shift was pressed but the cursor hasn't actually moved).
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = (self.cursor_row, self.cursor_col);
        if anchor == cursor {
            return None;
        }
        Some(if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        })
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    /// Starts a selection at the current cursor position if one isn't
    /// already in progress — called on the *first* Shift+navigation key, so
    /// later ones in the same drag just move the cursor and extend it.
    pub fn start_or_keep_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Selects the entire buffer: anchor at the very start, cursor at the
    /// very end, the same range shape Shift+navigation would produce if
    /// dragged across the whole document.
    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.current_line_len();
    }

    /// Moves the cursor to the start of `row` (clamped to the last valid
    /// row), with no selection side effects — used to seed the editor when
    /// entering Edit mode from a known source-line position (what the
    /// Normal-mode view was showing) instead of always starting at (0, 0).
    pub fn set_cursor_row(&mut self, row: usize) {
        self.cursor_row = row.min(self.lines.len().saturating_sub(1));
        self.cursor_col = 0;
    }

    /// Deletes the selected text (if any), moving the cursor to where the
    /// selection started. Returns whether anything was deleted, so callers
    /// (e.g. "typing replaces the selection") know whether to skip their
    /// own default handling of the key that triggered this.
    pub fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        self.selection_anchor = None;
        let (start_row, start_col) = start;
        let (end_row, end_col) = end;
        if start_row == end_row {
            let line = &mut self.lines[start_row];
            let from = char_to_byte(line, start_col);
            let to = char_to_byte(line, end_col);
            line.replace_range(from..to, "");
        } else {
            let end_line = self.lines[end_row].clone();
            let end_byte = char_to_byte(&end_line, end_col);
            let tail = end_line[end_byte..].to_string();
            let start_byte = char_to_byte(&self.lines[start_row], start_col);
            self.lines[start_row].truncate(start_byte);
            self.lines[start_row].push_str(&tail);
            self.lines.drain(start_row + 1..=end_row);
        }
        self.cursor_row = start_row;
        self.cursor_col = start_col;
        self.dirty = true;
        true
    }

    fn current_line_len(&self) -> usize {
        self.lines[self.cursor_row].chars().count()
    }

    fn clamp_col(&mut self) {
        self.cursor_col = self.cursor_col.min(self.current_line_len());
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_idx = char_to_byte(&self.lines[self.cursor_row], self.cursor_col);
        self.lines[self.cursor_row].insert(byte_idx, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..self.tab_width {
            self.insert_char(' ');
        }
    }

    pub fn insert_newline(&mut self) {
        let line = self.lines[self.cursor_row].clone();
        let byte_idx = char_to_byte(&line, self.cursor_col);
        let (left, right) = line.split_at(byte_idx);
        self.lines[self.cursor_row] = left.to_string();
        self.lines.insert(self.cursor_row + 1, right.to_string());
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let from = char_to_byte(line, self.cursor_col - 1);
            let to = char_to_byte(line, self.cursor_col);
            line.replace_range(from..to, "");
            self.cursor_col -= 1;
            self.dirty = true;
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
            self.lines[self.cursor_row].push_str(&current);
            self.dirty = true;
        }
    }

    pub fn delete_forward(&mut self) {
        let len = self.current_line_len();
        if self.cursor_col < len {
            let line = &mut self.lines[self.cursor_row];
            let from = char_to_byte(line, self.cursor_col);
            let to = char_to_byte(line, self.cursor_col + 1);
            line.replace_range(from..to, "");
            self.dirty = true;
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
            self.dirty = true;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_col < self.current_line_len() {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.current_line_len();
    }

    /// Keeps the cursor within a viewport of `height` visible lines.
    pub fn ensure_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.cursor_row < self.scroll {
            self.scroll = self.cursor_row;
        } else if self.cursor_row >= self.scroll + height {
            self.scroll = self.cursor_row - height + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_serialize_roundtrip() {
        let mut ed = Editor::new("hello", 4);
        ed.move_end();
        ed.insert_char('!');
        assert_eq!(ed.to_content(), "hello!");
    }

    #[test]
    fn newline_splits_line() {
        let mut ed = Editor::new("abcdef", 4);
        for _ in 0..3 {
            ed.move_right();
        }
        ed.insert_newline();
        assert_eq!(ed.lines(), &["abc".to_string(), "def".to_string()]);
        assert_eq!(ed.cursor(), (1, 0));
    }

    #[test]
    fn backspace_joins_lines_at_start() {
        let mut ed = Editor::new("abc\ndef", 4);
        ed.move_down();
        ed.move_home();
        ed.backspace();
        assert_eq!(ed.to_content(), "abcdef");
        assert_eq!(ed.cursor(), (0, 3));
    }

    #[test]
    fn backspace_within_line() {
        let mut ed = Editor::new("abc", 4);
        ed.move_end();
        ed.backspace();
        assert_eq!(ed.to_content(), "ab");
    }

    #[test]
    fn delete_forward_joins_next_line() {
        let mut ed = Editor::new("abc\ndef", 4);
        ed.move_end();
        ed.delete_forward();
        assert_eq!(ed.to_content(), "abcdef");
    }

    #[test]
    fn handles_utf8_correctly() {
        let mut ed = Editor::new("héllo", 4);
        ed.move_end();
        ed.insert_char('!');
        assert_eq!(ed.to_content(), "héllo!");
    }

    #[test]
    fn ensure_visible_scrolls_down_when_cursor_leaves_viewport() {
        let mut ed = Editor::new(&"line\n".repeat(20), 4);
        for _ in 0..15 {
            ed.move_down();
        }
        ed.ensure_visible(10);
        assert!(ed.scroll() > 0);
        assert!(ed.cursor().0 >= ed.scroll());
        assert!(ed.cursor().0 < ed.scroll() + 10);
    }

    #[test]
    fn selection_extends_across_repeated_shift_moves() {
        let mut ed = Editor::new("hello world", 4);
        ed.start_or_keep_selection();
        ed.move_right();
        ed.move_right();
        ed.start_or_keep_selection(); // no-op: anchor already set
        ed.move_right();
        assert_eq!(ed.selection_range(), Some(((0, 0), (0, 3))));
    }

    #[test]
    fn selection_is_none_when_anchor_equals_cursor() {
        let mut ed = Editor::new("hello", 4);
        ed.start_or_keep_selection();
        assert_eq!(ed.selection_range(), None);
        assert!(!ed.has_selection());
    }

    #[test]
    fn selection_normalizes_backward_drag() {
        let mut ed = Editor::new("hello", 4);
        ed.move_end();
        ed.start_or_keep_selection();
        ed.move_left();
        ed.move_left();
        assert_eq!(ed.selection_range(), Some(((0, 3), (0, 5))));
    }

    #[test]
    fn delete_selection_removes_text_on_one_line() {
        let mut ed = Editor::new("hello world", 4);
        ed.start_or_keep_selection();
        for _ in 0..5 {
            ed.move_right();
        }
        assert!(ed.delete_selection());
        assert_eq!(ed.to_content(), " world");
        assert_eq!(ed.cursor(), (0, 0));
        assert!(!ed.has_selection());
    }

    #[test]
    fn delete_selection_joins_across_lines() {
        let mut ed = Editor::new("abc\ndef\nghi", 4);
        ed.start_or_keep_selection();
        ed.move_down();
        ed.move_down();
        ed.move_right();
        ed.delete_selection();
        assert_eq!(ed.to_content(), "hi");
        assert_eq!(ed.cursor(), (0, 0));
    }

    #[test]
    fn delete_selection_on_empty_selection_is_noop() {
        let mut ed = Editor::new("hello", 4);
        assert!(!ed.delete_selection());
        assert_eq!(ed.to_content(), "hello");
    }

    #[test]
    fn select_all_spans_the_whole_buffer() {
        let mut ed = Editor::new("abc\ndef\nghi", 4);
        ed.move_right();
        ed.select_all();
        assert_eq!(ed.selection_range(), Some(((0, 0), (2, 3))));
        assert_eq!(ed.cursor(), (2, 3));
    }

    #[test]
    fn select_all_then_delete_clears_the_buffer() {
        let mut ed = Editor::new("abc\ndef", 4);
        ed.select_all();
        assert!(ed.delete_selection());
        assert_eq!(ed.to_content(), "");
        assert_eq!(ed.cursor(), (0, 0));
    }

    #[test]
    fn set_cursor_row_moves_to_the_start_of_that_row() {
        let mut ed = Editor::new("abc\ndef\nghi", 4);
        ed.move_end();
        ed.set_cursor_row(2);
        assert_eq!(ed.cursor(), (2, 0));
    }

    #[test]
    fn set_cursor_row_clamps_past_the_last_line() {
        let mut ed = Editor::new("abc\ndef", 4);
        ed.set_cursor_row(50);
        assert_eq!(ed.cursor(), (1, 0));
    }
}
