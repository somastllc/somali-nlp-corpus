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

/// Numbers written locally after a word that announces a phone number (`tel:0502488985`,
/// `Utas: 0407353505`). The prefix is not restricted to Somali operators, since these
/// numbers appear in diaspora ads; the keyword is what makes a bare digit run a phone.
fn contextual_phone_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:tel\w*|utas|mob(?:ile)?|phone|call|whats?app|wac|lambar\w*|number)\b[\s.:\-–—]*(0\d{8,10})",
        )
        .unwrap()
    })
}

/// A candidate counts as a phone number only when it is not part of a longer token:
/// not preceded by a letter, digit, or one of `+ . -`, and not followed by a digit.
/// Exceptions: a `.` after a letter (`Tel.0634426761`) and a `-` or `.` before a `+`
/// number (`Tell:-+6125324626`) are separators. A comma preceded by a digit may be a
/// thousands separator (`1,000`), so it blocks `00` and local candidates only.
fn has_phone_boundaries(text: &str, start: usize, end: usize, plus_prefixed: bool) -> bool {
    let mut before = text[..start].chars().rev();
    let before_ok = match before.next() {
        None => true,
        Some(c) if c.is_alphanumeric() || c == '+' => false,
        Some('.') => plus_prefixed || before.next().is_some_and(char::is_alphabetic),
        Some('-') => plus_prefixed,
        Some(',') => plus_prefixed || !before.next().is_some_and(|p| p.is_ascii_digit()),
        Some(_) => true,
    };
    let after_ok = text[end..]
        .chars()
        .next()
        .map_or(true, |c| !c.is_ascii_digit());
    before_ok && after_ok
}

fn mask_phones_with(text: &str, re: &Regex, international: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text) {
        let digits = digit_count(m.as_str());
        let plausible = (MIN_PHONE_DIGITS..=MAX_PHONE_DIGITS).contains(&digits)
            && (!international || !m.as_str().contains('.'));
        if plausible && has_phone_boundaries(text, m.start(), m.end(), m.as_str().starts_with('+'))
        {
            out.push_str(&text[last..m.start()]);
            out.push_str(PHONE_SENTINEL);
            last = m.end();
        }
    }
    out.push_str(&text[last..]);
    out
}

fn mask_contextual_phones(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for caps in contextual_phone_re().captures_iter(text) {
        let number = caps.get(1).expect("group 1 is not optional");
        if text[number.end()..]
            .chars()
            .next()
            .map_or(true, |c| !c.is_ascii_digit())
        {
            out.push_str(&text[last..number.start()]);
            out.push_str(PHONE_SENTINEL);
            last = number.end();
        }
    }
    out.push_str(&text[last..]);
    out
}

pub fn mask_phone_numbers(text: &str) -> String {
    let once = mask_phones_with(text, international_phone_re(), true);
    let local = mask_phones_with(&once, somali_local_phone_re(), false);
    mask_contextual_phones(&local)
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
        let out = mask_contacts(
            "visit https://example.com or mail a@b.com",
            true,
            true,
            false,
        );
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
        assert_eq!(
            mask_phone_numbers("qiimo 1,000 5000 1000 123"),
            "qiimo 1,000 5000 1000 123"
        );
    }

    #[test]
    fn number_inside_a_longer_digit_run_is_not_masked() {
        assert_eq!(mask_phone_numbers("id 99061219223599"), "id 99061219223599");
    }

    #[test]
    fn masks_numbers_with_separators_and_phone_keywords() {
        for (text, expected) in [
            ("Tel.Tel.0634426761 iyo", "Tel.Tel.⟨phone⟩ iyo"),
            ("Tell:-+6125324626 iyo", "Tell:-⟨phone⟩ iyo"),
            ("Tell:⟨phone⟩,0615771050.", "Tell:⟨phone⟩,⟨phone⟩."),
            ("UAE, tel:0502488985— waa", "UAE, tel:⟨phone⟩— waa"),
            (
                "Telefonkan–:07939213271 Wakhtigana",
                "Telefonkan–:⟨phone⟩ Wakhtigana",
            ),
            (
                "bga. Utas: 0407353505 Canberrad",
                "bga. Utas: ⟨phone⟩ Canberrad",
            ),
        ] {
            assert_eq!(mask_phone_numbers(text), expected);
        }
    }

    #[test]
    fn bare_digit_runs_without_a_phone_context_stay() {
        for text in [
            "Wa-l-arqaam 0123456789. Tuktabu",
            "ISBN 978-0195125580.",
            "version 1.0634426761 waa",
            "tel 05024889851234",
            "lambar 0123 4",
        ] {
            assert_eq!(mask_phone_numbers(text), text);
        }
    }
}
