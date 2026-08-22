use ratatui::style::Color;
use tmr_core::theme::Theme;

/// The core [`Theme`] resolved into ratatui [`Color`]s. Kept separate from
/// `tmr-core` so the core crate never has to depend on ratatui.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
    pub border: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Palette {
    pub fn from_theme(theme: &Theme) -> Self {
        Palette {
            fg: parse_color(&theme.foreground),
            bg: parse_color(&theme.background),
            accent: parse_color(&theme.accent),
            border: parse_color(&theme.border),
            muted: parse_color(&theme.muted),
            success: parse_color(&theme.success),
            warning: parse_color(&theme.warning),
            error: parse_color(&theme.error),
        }
    }
}

/// Parses `"#rrggbb"` or a common ANSI color name into a ratatui [`Color`].
/// Never fails outright — unrecognized input falls back to `Color::Reset`
/// (the terminal's default) so a typo in a theme file can't crash tmr.
pub fn parse_color(spec: &str) -> Color {
    let spec = spec.trim();
    if let Some(hex) = spec.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        return Color::Reset;
    }
    match spec.to_ascii_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_color() {
        assert_eq!(parse_color("#ff00aa"), Color::Rgb(0xff, 0x00, 0xaa));
    }

    #[test]
    fn parses_named_color() {
        assert_eq!(parse_color("blue"), Color::Blue);
    }

    #[test]
    fn falls_back_on_garbage() {
        assert_eq!(parse_color("not-a-color"), Color::Reset);
    }
}
