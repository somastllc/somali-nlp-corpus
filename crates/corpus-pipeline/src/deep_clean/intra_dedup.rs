//! Intra-document paragraph deduplication.

use std::collections::HashSet;

/// Word 3-gram shingles. Unlike a bag of words, they depend on word order, so two
/// paragraphs that merely share vocabulary are not mistaken for duplicates.
const SHINGLE_WORDS: usize = 3;

fn tokens(text: &str) -> Vec<String> {
    text.split_whitespace().map(|w| w.to_lowercase()).collect()
}

fn shingles(words: &[String]) -> HashSet<Vec<&str>> {
    words
        .windows(SHINGLE_WORDS)
        .map(|w| w.iter().map(String::as_str).collect())
        .collect()
}

fn jaccard<T: std::hash::Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

/// Whether `current` repeats `previous`: identical text, or for paragraphs long
/// enough to shingle, ordered shingle overlap of at least `tau`.
fn is_repeat(previous: &[String], current: &[String], tau: f64) -> bool {
    if previous == current {
        return true;
    }
    if previous.len() < SHINGLE_WORDS || current.len() < SHINGLE_WORDS {
        return false;
    }
    jaccard(&shingles(previous), &shingles(current)) >= tau
}

pub fn dedup_paragraphs(text: &str, enabled: bool, tau: f64) -> String {
    if !enabled {
        return text.to_string();
    }
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    if paragraphs.len() <= 1 {
        return text.to_string();
    }
    let mut kept = Vec::new();
    let mut prev: Option<Vec<String>> = None;
    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        let words = tokens(trimmed);
        let duplicate = prev
            .as_ref()
            .is_some_and(|previous| is_repeat(previous, &words, tau));
        if !duplicate {
            kept.push(trimmed);
            prev = Some(words);
        }
    }
    kept.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_duplicate_paragraph() {
        let text = "Wax wanaagsan oo dheer\n\nWax wanaagsan oo dheer\n\nKale";
        let out = dedup_paragraphs(text, true, 0.95);
        assert_eq!(out.matches("Wax wanaagsan").count(), 1);
    }

    #[test]
    fn same_vocabulary_in_different_order_is_kept() {
        let text = "alpha beta gamma delta epsilon zeta\n\nzeta epsilon delta gamma beta alpha";
        let out = dedup_paragraphs(text, true, 0.95);
        assert_eq!(out.matches("\n\n").count(), 1);
    }

    #[test]
    fn short_distinct_paragraphs_are_kept() {
        let out = dedup_paragraphs("Hubaal\n\nMahadsanid", true, 0.95);
        assert!(out.contains("Hubaal") && out.contains("Mahadsanid"));
    }

    #[test]
    fn identical_short_paragraph_is_removed() {
        let out = dedup_paragraphs("Hubaal\n\nHubaal", true, 0.95);
        assert_eq!(out, "Hubaal");
    }
}
