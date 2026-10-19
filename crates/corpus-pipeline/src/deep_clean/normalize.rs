//! Source-aware P0 normalization for the deep-clean stage.

use std::sync::OnceLock;

use regex::Regex;

pub fn normalize_source_text(source: &str, text: &str, unescape_madlad: bool, strip_opus_html: bool) -> String {
    let mut out = text.to_string();
    if unescape_madlad && source == "madlad" {
        out = unescape_literal_escapes(&out);
    }
    if strip_opus_html && source == "opus" {
        out = strip_opus_html_fragments(&out);
    }
    out
}

pub fn unescape_literal_escapes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('/') => out.push('/'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Matches a single complete tag at the end of the text, including the
/// JSON-escaped closing form (`<\/p>`) that OPUS rows sometimes carry.
fn trailing_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<\\?/?[a-z][a-z0-9]*(\s[^<>]*)?/?>\s*$").unwrap())
}

/// Drop markup left dangling at the end of an OPUS sentence. Only a complete
/// trailing tag is removed; visible text that follows an earlier tag is kept.
pub fn strip_opus_html_fragments(text: &str) -> String {
    let mut current = text.trim().to_string();
    while let Some(m) = trailing_tag_re().find(&current) {
        current.truncate(m.start());
        current = current.trim().to_string();
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn madlad_unescape() {
        assert_eq!(
            normalize_source_text("madlad", r"a\nb", true, false),
            "a\nb"
        );
    }

    #[test]
    fn opus_trailing_tag_removed() {
        assert_eq!(strip_opus_html_fragments("Waa wanaagsan </p>"), "Waa wanaagsan");
        assert_eq!(strip_opus_html_fragments("Waa wanaagsan <br>"), "Waa wanaagsan");
        assert_eq!(strip_opus_html_fragments("Waa wanaagsan <\\/p>"), "Waa wanaagsan");
    }

    #[test]
    fn opus_text_after_tag_is_kept() {
        assert_eq!(
            strip_opus_html_fragments("Waa wanaagsan<br>Dadku way shaqeeyaan"),
            "Waa wanaagsan<br>Dadku way shaqeeyaan"
        );
    }
}
