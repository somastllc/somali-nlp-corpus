//! URL, email, and phone-number masking for deep clean.

use std::sync::OnceLock;

use regex::Regex;

pub const URL_SENTINEL: &str = "⟨url⟩";
pub const EMAIL_SENTINEL: &str = "⟨email⟩";
pub const PHONE_SENTINEL: &str = "⟨phone⟩";

fn url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:https?://|www\.)[^\s<>]+|\b[a-z0-9][a-z0-9.-]+\.(?:com|org|net|so|edu|gov|info|io|co)(?:/[^\s]*)?",
        )
        .unwrap()
    })
}

fn email_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").unwrap())
}

/// International numbers: a `+` or `00` prefix, then digits with space, hyphen, or
/// parenthesis separators. A `.` is not a separator, which keeps `1.000-2.000` out.
fn international_phone_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:\+|00)\d[\d ()\-]{7,18}\d").unwrap())
}

/// Somali mobile numbers written locally: `06x`, `07x`, or `09x`, then 6 or 7 digits
/// with optional single spaces. Hyphens are not accepted, which keeps exam codes such
/// as `065 300-209` out.
fn somali_local_phone_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"0(?:6[1-9]|7[1-9]|9[0-9]) ?\d{3} ?\d{3,4}").unwrap())
}

const MIN_PHONE_DIGITS: usize = 9;
const MAX_PHONE_DIGITS: usize = 15;

fn digit_count(s: &str) -> usize {
    s.chars().filter(char::is_ascii_digit).count()
}

/// A candidate counts as a phone number only when it is not part of a longer token:
/// not preceded by a letter, digit, or one of `+ . -`, and not followed by a digit. A
/// comma may precede only a `+` number: before `00` or a local prefix it could be a
/// thousands separator (`1,000`).
fn has_phone_boundaries(text: &str, start: usize, end: usize, plus_prefixed: bool) -> bool {
    let before_ok = text[..start].chars().next_back().is_none_or(|c| {
        !(c.is_alphanumeric() || matches!(c, '+' | '.' | '-') || (c == ',' && !plus_prefixed))
    });
    let after_ok = text[end..].chars().next().is_none_or(|c| !c.is_ascii_digit());
    before_ok && after_ok
}

fn mask_phones_with(text: &str, re: &Regex, international: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text) {
        let digits = digit_count(m.as_str());
        let plausible = (MIN_PHONE_DIGITS..=MAX_PHONE_DIGITS).contains(&digits)
            && (!international || !m.as_str().contains('.'));
        if plausible && has_phone_boundaries(text, m.start(), m.end(), m.as_str().starts_with('+')) {
            out.push_str(&text[last..m.start()]);
            out.push_str(PHONE_SENTINEL);
            last = m.end();
        }
    }
    out.push_str(&text[last..]);
    out
}

pub fn mask_phone_numbers(text: &str) -> String {
    let once = mask_phones_with(text, international_phone_re(), true);
    mask_phones_with(&once, somali_local_phone_re(), false)
}

pub fn mask_contacts(text: &str, mask_urls: bool, mask_emails: bool, mask_phones: bool) -> String {
    let mut out = text.to_string();
    if mask_emails {
        out = email_re().replace_all(&out, EMAIL_SENTINEL).into_owned();
    }
    if mask_urls {
        out = url_re().replace_all(&out, URL_SENTINEL).into_owned();
    }
    if mask_phones {
        out = mask_phone_numbers(&out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_url_and_email() {
        let out = mask_contacts("visit https://example.com or mail a@b.com", true, true, false);
        assert!(out.contains(URL_SENTINEL));
        assert!(out.contains(EMAIL_SENTINEL));
    }

    #[test]
    fn masks_international_and_somali_local_numbers() {
        let text = "Tel +252 63 6898119 ama 00252634495860, mobile: +252-615536608 iyo 0612192235 \
                    ama 061 5303154 ama 071 873 1902.";
        let out = mask_phone_numbers(text);
        assert_eq!(out.matches(PHONE_SENTINEL).count(), 6, "{out}");
        assert!(!out.chars().any(|c| c.is_ascii_digit()), "{out}");
    }

    #[test]
    fn leaves_non_phone_digit_strings_alone() {
        for text in [
            "Qiimaha 1.000-2.000-5.000-10.000-50.000 taasi",
            "ISBN: 978-0993431371 |",
            "imtixaanka 300-209 70-243 065 300-209",
            "Sanadkii 1995 wuxuu xiriirinayay 2,500,000 qof",
            "taariikhda 12/03/2024 iyo 2024-03-12",
            "lambarka 123456789012345678",
        ] {
            assert_eq!(mask_phone_numbers(text), text);
        }
    }

    #[test]
    fn international_number_after_a_comma_is_masked() {
        let out = mask_phone_numbers("Shirka waa, +46-735804527 Waa bilaahi");
        assert_eq!(out, "Shirka waa, ⟨phone⟩ Waa bilaahi");
        assert_eq!(mask_phone_numbers("qiimo 1,000 5000 1000 123"), "qiimo 1,000 5000 1000 123");
    }

    #[test]
    fn number_inside_a_longer_digit_run_is_not_masked() {
        assert_eq!(mask_phone_numbers("id 99061219223599"), "id 99061219223599");
    }
}
