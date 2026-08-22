//! Terminal image rendering, with a capability-gated real backend and an
//! always-safe text fallback.
//!
//! Actual terminal graphics protocols (Kitty, iTerm2, Sixel) are real
//! protocol implementations, not something to bolt on speculatively — so
//! this version ships one concrete backend that works nearly everywhere
//! truecolor is available (a Unicode half-block renderer, using only the
//! `image` crate to decode/resize), plus capability detection and a text
//! fallback for anything else. The `ImageBackend` trait is the seam a
//! Kitty/Sixel backend could plug into later without touching call sites.

use std::path::Path;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// What we've detected the terminal can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCapability {
    /// Truecolor is available; we can approximate images with colored
    /// half-block characters.
    Halfblocks,
    /// No usable image rendering; always fall back to a text placeholder.
    None,
}

/// Detects capability from environment variables only — no blocking writes
/// to the terminal (those risk hanging in odd environments), so this is
/// always fast and safe to call at startup.
pub fn detect_capability() -> ImageCapability {
    let truecolor = std::env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false);
    if truecolor {
        return ImageCapability::Halfblocks;
    }
    let term = std::env::var("TERM").unwrap_or_default();
    if term.contains("256color") || term.contains("kitty") || term.contains("alacritty") {
        return ImageCapability::Halfblocks;
    }
    ImageCapability::None
}

/// Renders `path` as terminal lines: real half-block pixels when `cap`
/// allows and the file decodes cleanly, otherwise an elegant text
/// placeholder. Never panics or propagates a hard error — a broken/missing
/// image is not allowed to take down the whole document view.
pub fn render(
    path: &Path,
    cap: ImageCapability,
    max_width: u16,
    max_height: u16,
) -> Vec<Line<'static>> {
    if cap == ImageCapability::Halfblocks {
        if let Some(lines) = render_halfblocks(path, max_width, max_height) {
            return lines;
        }
    }
    placeholder(path)
}

fn placeholder(path: &Path) -> Vec<Line<'static>> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    vec![Line::from(Span::styled(
        format!("[image: {name}]"),
        Style::default().add_modifier(ratatui::style::Modifier::ITALIC),
    ))]
}

/// Downsamples the image to `width x height*2` pixels (two vertical pixel
/// samples per terminal cell) and draws each cell as a `▀` glyph whose
/// foreground is the top pixel and background is the bottom pixel.
fn render_halfblocks(path: &Path, max_width: u16, max_height: u16) -> Option<Vec<Line<'static>>> {
    let img = image::open(path).ok()?.into_rgba8();
    let width = max_width.max(1) as u32;
    let height = (max_height.max(1) as u32) * 2;
    let (orig_w, orig_h) = (img.width(), img.height());
    if orig_w == 0 || orig_h == 0 {
        return None;
    }
    let scale = f64::min(width as f64 / orig_w as f64, height as f64 / orig_h as f64).min(1.0);
    let target_w = ((orig_w as f64 * scale).round() as u32).max(1).min(width);
    let target_h = ((orig_h as f64 * scale).round() as u32).max(2).min(height);
    let resized = image::imageops::resize(
        &img,
        target_w,
        target_h,
        image::imageops::FilterType::Triangle,
    );

    let mut lines = Vec::new();
    let mut y = 0;
    while y < resized.height() {
        let mut spans = Vec::with_capacity(resized.width() as usize);
        for x in 0..resized.width() {
            let top = resized.get_pixel(x, y);
            let bottom = if y + 1 < resized.height() {
                resized.get_pixel(x, y + 1)
            } else {
                top
            };
            let fg = Color::Rgb(top[0], top[1], top[2]);
            let bg = Color::Rgb(bottom[0], bottom[1], bottom[2]);
            spans.push(Span::styled("\u{2580}", Style::default().fg(fg).bg(bg)));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    Some(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_shown_for_missing_file() {
        let lines = render(
            Path::new("does-not-exist.png"),
            ImageCapability::None,
            40,
            10,
        );
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn halfblocks_fall_back_gracefully_on_bad_path() {
        let lines = render(
            Path::new("does-not-exist.png"),
            ImageCapability::Halfblocks,
            40,
            10,
        );
        // Decoding fails, so we still get the text placeholder, not a panic.
        assert_eq!(lines.len(), 1);
    }
}
