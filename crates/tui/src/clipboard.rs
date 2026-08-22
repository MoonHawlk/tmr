//! System clipboard access via the OSC 52 terminal escape sequence
//! (`ESC ] 52 ; c ; <base64> BEL`), rather than a native clipboard crate.
//! OSC 52 is understood by most modern terminal emulators (iTerm2, Kitty,
//! WezTerm, foot, Windows Terminal, xterm with `allowClipboardOperations`)
//! and — unlike a native clipboard API — still works over SSH or inside
//! tmux, where there's no direct path back to the user's desktop clipboard.
//! It's write-only (no read-back), which is all Copy/Cut need. No new
//! dependency: the small base64 encoder below is all OSC 52 needs, and
//! matches this crate's existing "keep the built-in editor dependency-free"
//! stance (see `crates/tui/src/editor.rs`).
use std::io::{self, Write};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Sets the system clipboard to `text` via an OSC 52 escape sequence
/// written directly to stdout. Fire-and-forget: there's no response to
/// wait for (and nothing here blocks on one), matching how
/// `image_backend`'s truecolor detection avoids blocking terminal queries.
pub fn set_clipboard(text: &str) -> io::Result<()> {
    let encoded = base64_encode(text.as_bytes());
    write!(io::stdout(), "\x1b]52;c;{encoded}\x07")?;
    io::stdout().flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_matches_rfc_4648_test_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_encode_handles_utf8() {
        assert_eq!(base64_encode("héllo".as_bytes()), "aMOpbGxv");
    }
}
