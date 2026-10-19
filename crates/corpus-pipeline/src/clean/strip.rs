//! Removal of control and invisible characters that NFC leaves behind and that
//! poison shingling and hashing. See docs/CLEANING_PLAN.md (Clean stage).

/// Keep a character: drop control characters (except newline and tab) and the
/// common invisible web-junk code points.
fn keep(c: char) -> bool {
    match c {
        '\n' | '\t' => true,
        // Zero-width, BOM/ZWNBSP, soft hyphen.
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{00AD}' => false,
        // Bidirectional controls.
        '\u{202A}'..='\u{202E}' => false,
        c if c.is_control() => false,
        _ => true,
    }
}

/// Strip control and invisible characters. Line endings are normalized first: `\r\n`
/// becomes `\n`, and a lone `\r` (old Mac endings) becomes `\n` instead of vanishing
/// and gluing the neighbouring words together.
pub fn strip_invisibles(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    out.push('\n');
                }
            }
            c if keep(c) => out.push(c),
            _ => {}
        }
    }
    out
}

/// Remove isolated U+FFFD replacement characters from otherwise kept documents.
pub fn strip_stray_ufffd(text: &str) -> String {
    text.replace('\u{FFFD}', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_newline_and_tab() {
        assert_eq!(strip_invisibles("a\nb\tc"), "a\nb\tc");
    }

    #[test]
    fn removes_zero_width_and_bom() {
        assert_eq!(strip_invisibles("a\u{200B}b\u{FEFF}c"), "abc");
    }

    #[test]
    fn removes_carriage_return_and_controls() {
        assert_eq!(strip_invisibles("a\r\nb\u{0007}c"), "a\nbc");
    }

    #[test]
    fn lone_carriage_return_keeps_line_boundary() {
        assert_eq!(strip_invisibles("wanaagsan\rDadku"), "wanaagsan\nDadku");
    }

    #[test]
    fn removes_soft_hyphen_and_bidi() {
        assert_eq!(strip_invisibles("a\u{00AD}b\u{202E}c"), "abc");
    }
}
