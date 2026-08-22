//! A minimal JSON syntax highlighter: colors tokens (keys, strings,
//! numbers, `true`/`false`/`null`, punctuation) without re-parsing or
//! reformatting the document — every source line maps 1:1 to one
//! `RenderedLine`, exactly like `markdown_view::render_plain_text`, so
//! scrolling and cursor addressing stay as simple as the plain-text path.
//! Gated behind `[ui] json_highlight` (see
//! `crates/tui/src/input.rs::refresh_rendered`) — off by default, so a
//! `.json` file renders as plain text unless the user opts in via
//! `config.toml` or the Settings window.
//!
//! Deliberately line-local rather than a real JSON parser: a JSON string
//! can't legally contain a raw newline, so tokenizing one line at a time
//! is enough for well-formed JSON. Malformed input (e.g. an unterminated
//! string spanning a line break) just degrades to slightly-off coloring on
//! the affected lines — every byte always lands in *some* token, so there
//! is nothing to error out on.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::markdown_view::RenderedLine;
use crate::theme::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    Key,
    String,
    Number,
    Literal, // true / false / null
    Punct,
    Other,
}

/// Tokenizes one line into `(Tok, &str)` slices that, concatenated in
/// order, reproduce `line` exactly (whitespace included) — so styling
/// never drops or duplicates a character.
fn tokenize_line(line: &str) -> Vec<(Tok, &str)> {
    let mut out = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        if c == '"' {
            chars.next();
            let mut end = start + c.len_utf8();
            loop {
                match chars.peek().copied() {
                    None => break,
                    Some((idx, '\\')) => {
                        chars.next();
                        end = idx + '\\'.len_utf8();
                        if let Some((idx2, ch2)) = chars.peek().copied() {
                            chars.next();
                            end = idx2 + ch2.len_utf8();
                        }
                    }
                    Some((idx, ch)) => {
                        chars.next();
                        end = idx + ch.len_utf8();
                        if ch == '"' {
                            break;
                        }
                    }
                }
            }
            let text = &line[start..end];
            // A key if the next non-whitespace char (still on this line)
            // is ':' — checked without consuming, so the colon itself is
            // tokenized normally on the next loop iteration.
            let mut lookahead = chars.clone();
            let mut is_key = false;
            while let Some(&(_, wc)) = lookahead.peek() {
                if wc.is_whitespace() {
                    lookahead.next();
                    continue;
                }
                is_key = wc == ':';
                break;
            }
            out.push((if is_key { Tok::Key } else { Tok::String }, text));
        } else if c.is_whitespace() {
            let mut end = start;
            while let Some(&(idx, ch)) = chars.peek() {
                if !ch.is_whitespace() {
                    break;
                }
                chars.next();
                end = idx + ch.len_utf8();
            }
            out.push((Tok::Other, &line[start..end]));
        } else if "{}[]:,".contains(c) {
            chars.next();
            out.push((Tok::Punct, &line[start..start + c.len_utf8()]));
        } else if line[start..].starts_with("true") {
            for _ in 0..4 {
                chars.next();
            }
            out.push((Tok::Literal, &line[start..start + 4]));
        } else if line[start..].starts_with("false") {
            for _ in 0..5 {
                chars.next();
            }
            out.push((Tok::Literal, &line[start..start + 5]));
        } else if line[start..].starts_with("null") {
            for _ in 0..4 {
                chars.next();
            }
            out.push((Tok::Literal, &line[start..start + 4]));
        } else if c.is_ascii_digit() || c == '-' {
            let mut end = start;
            while let Some(&(idx, ch)) = chars.peek() {
                if ch.is_ascii_digit() || matches!(ch, '.' | 'e' | 'E' | '+' | '-') {
                    chars.next();
                    end = idx + ch.len_utf8();
                } else {
                    break;
                }
            }
            out.push((Tok::Number, &line[start..end]));
        } else {
            chars.next();
            out.push((Tok::Other, &line[start..start + c.len_utf8()]));
        }
    }

    out
}

fn style_for(tok: Tok, palette: &Palette) -> Style {
    match tok {
        Tok::Key => Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD),
        Tok::String => Style::default().fg(palette.success),
        Tok::Number => Style::default().fg(palette.warning),
        Tok::Literal => Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::ITALIC),
        Tok::Punct => Style::default().fg(palette.muted),
        Tok::Other => Style::default().fg(palette.fg),
    }
}

/// Renders `content` (assumed to be JSON) with one `RenderedLine` per
/// source line, tokens colored by `style_for`. Never fails — content that
/// isn't valid JSON just renders with best-effort/approximate coloring
/// rather than an error, the same tolerant spirit as the Markdown parser.
pub fn render(content: &str, palette: &Palette) -> Vec<RenderedLine> {
    content
        .lines()
        .map(|line| RenderedLine {
            line: Line::from(
                tokenize_line(line)
                    .into_iter()
                    .map(|(tok, text)| Span::styled(text.to_string(), style_for(tok, palette)))
                    .collect::<Vec<_>>(),
            ),
            task_index: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str) -> Vec<Tok> {
        tokenize_line(line).into_iter().map(|(t, _)| t).collect()
    }

    fn reconstruct(line: &str) -> String {
        tokenize_line(line)
            .into_iter()
            .map(|(_, t)| t)
            .collect::<String>()
    }

    #[test]
    fn tokens_reconstruct_the_original_line_exactly() {
        for line in [
            r#"{ "name": "Ada", "age": 36, "active": true, "score": -1.5e3 }"#,
            r#"  "escaped \"quote\" and \\backslash": null,"#,
            "not really json at all, just [text]",
            "",
        ] {
            assert_eq!(reconstruct(line), line);
        }
    }

    #[test]
    fn a_quoted_string_before_a_colon_is_a_key() {
        let toks = tokenize_line(r#""name": "Ada""#);
        assert_eq!(toks[0].0, Tok::Key);
        assert_eq!(toks[0].1, r#""name""#);
    }

    #[test]
    fn a_quoted_string_not_before_a_colon_is_a_plain_string() {
        let toks = tokenize_line(r#"["Ada", "Grace"]"#);
        let strings: Vec<_> = toks.iter().filter(|(t, _)| *t == Tok::String).collect();
        assert_eq!(strings.len(), 2);
        assert!(toks.iter().all(|(t, _)| *t != Tok::Key));
    }

    #[test]
    fn key_lookahead_skips_whitespace_before_the_colon() {
        let toks = tokenize_line(r#""name"   : "Ada""#);
        assert_eq!(toks[0].0, Tok::Key);
    }

    #[test]
    fn numbers_and_literals_are_classified() {
        let toks = kinds("-1.5e3 true false null 42");
        assert_eq!(
            toks,
            vec![
                Tok::Number,
                Tok::Other,
                Tok::Literal,
                Tok::Other,
                Tok::Literal,
                Tok::Other,
                Tok::Literal,
                Tok::Other,
                Tok::Number,
            ]
        );
    }

    #[test]
    fn escaped_quote_does_not_end_the_string_early() {
        let toks = tokenize_line(r#""a \"b\" c""#);
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].0, Tok::String);
        assert_eq!(toks[0].1, r#""a \"b\" c""#);
    }

    #[test]
    fn unterminated_string_consumes_to_end_of_line_without_panicking() {
        let toks = tokenize_line(r#""never closes"#);
        assert_eq!(reconstruct(r#""never closes"#), r#""never closes"#);
        assert_eq!(toks.len(), 1);
    }

    #[test]
    fn render_produces_one_line_per_source_line() {
        let pal = Palette::from_theme(&tmr_core::theme::Theme::dark());
        let lines = render("{\n  \"a\": 1\n}\n", &pal);
        assert_eq!(lines.len(), 3);
    }
}
