//! HTML two-tier policy for deep clean.

use std::sync::OnceLock;

use common::types::QualityFlag;
use regex::Regex;

fn script_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?is)<\?(php)?.*?\?>|<script\b.*?</script>|<style\b.*?</style>|<mutation\b.*?>|<char\b.*?>")
            .unwrap()
    })
}

fn tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)</?[a-zA-Z][a-zA-Z0-9]*(\s[^>]*|/|;|<[^>]*)?>").unwrap())
}

pub fn has_reject_html(text: &str) -> bool {
    script_re().is_match(text)
}

/// Tags that separate words or lines. Removing them without a replacement would
/// glue neighbouring words together (`wanaagsan<br>Dadku` -> `wanaagsanDadku`).
const BREAKING_TAGS: &[&str] = &[
    "br",
    "p",
    "div",
    "li",
    "ul",
    "ol",
    "tr",
    "td",
    "th",
    "table",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "blockquote",
    "section",
    "article",
    "header",
    "footer",
    "hr",
    "pre",
];

fn is_breaking_tag(tag: &str) -> bool {
    let name: String = tag
        .trim_start_matches('<')
        .trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    BREAKING_TAGS.contains(&name.as_str())
}

/// Remove tags, keeping a line boundary where a block-level tag stood.
pub fn strip_benign_tags(text: &str) -> String {
    tag_re()
        .replace_all(
            text,
            |caps: &regex::Captures| {
                if is_breaking_tag(&caps[0]) {
                    "\n"
                } else {
                    ""
                }
            },
        )
        .into_owned()
}

pub fn apply_html_policy(
    text: &str,
    reject_script: bool,
    strip_benign: bool,
) -> Result<String, QualityFlag> {
    if reject_script && has_reject_html(text) {
        return Err(QualityFlag::HtmlRemnant);
    }
    let stripped = if strip_benign {
        strip_benign_tags(text)
    } else {
        text.to_string()
    };
    Ok(stripped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_script() {
        assert!(has_reject_html("hello <?php echo x; ?>"));
    }

    #[test]
    fn strips_inline_tags() {
        assert_eq!(strip_benign_tags("a <b>bold</b> c"), "a bold c");
    }

    #[test]
    fn strips_malformed_tags_seen_in_web_text() {
        assert_eq!(
            strip_benign_tags("Sanadkii <span;>1995 wuxuu"),
            "Sanadkii 1995 wuxuu"
        );
        assert_eq!(strip_benign_tags("Haa!!<br< span=\"\">"), "Haa!!\n");
    }

    #[test]
    fn line_break_tag_keeps_word_boundary() {
        assert_eq!(strip_benign_tags("wanaagsan<br>Dadku"), "wanaagsan\nDadku");
        assert_eq!(strip_benign_tags("a<BR/>b"), "a\nb");
    }

    #[test]
    fn block_tags_keep_boundary_inline_tags_do_not() {
        assert_eq!(strip_benign_tags("<p>one</p><p>two</p>"), "\none\n\ntwo\n");
        assert_eq!(strip_benign_tags("foo<span>bar</span>"), "foobar");
    }
}
