//! Static source registry: the single vocabulary for source keys, classes,
//! licenses, and per-source processing policy (see docs/SOURCES.md and
//! docs/CLEANING_PLAN.md).

use crate::types::{License, SourceKey};

/// Whether a source produces full documents or individual aligned sentences.
/// Controls length floors, LID policy, and near-dedup participation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceClass {
    Document,
    Sentence,
}

/// How language identification treats a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidPolicy {
    /// Drop records below the configured confidence threshold.
    Gate,
    /// Record the language score but never reject (curated parallel data).
    TagOnly,
}

/// Registry metadata for one data source.
#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub key: &'static str,
    pub class: SourceClass,
    pub lid_policy: LidPolicy,
    pub near_dedup: bool,
    /// SPDX-style license identifier; mapped to [`License`] via [`SourceEntry::license`].
    pub license_id: &'static str,
}

impl SourceEntry {
    pub fn source_key(&self) -> SourceKey {
        SourceKey(self.key.to_string())
    }

    /// Resolve the license identifier to a [`License`] value.
    pub fn license(&self) -> License {
        License::from_id(self.license_id)
    }

    /// License for one record of this source, or `None` when the record must not be
    /// released. Aggregate sources carry a different upstream license per sub-source,
    /// and a sub-source we cannot place is excluded rather than guessed.
    pub fn record_license(&self, subsource: Option<&str>) -> Option<License> {
        let Some(table) = subsource_table(self.key) else {
            return Some(self.license());
        };
        let subsource = subsource?.to_ascii_lowercase();
        table
            .iter()
            .find(|(name, _)| *name == subsource)
            .and_then(|(_, license_id)| *license_id)
            .map(License::from_id)
    }
}

/// Glot500 `dataset` values and the upstream license of each, from the Glot500-c
/// source table (github.com/cisnlp/Glot500). `None` marks sub-sources we exclude:
/// MTData lists "multiple licenses", Earthlings is GPL-3.0, Bible_crawled has no
/// license listed, and Bible_pbc is available only on request for academic use.
/// Wikipedia is recorded as Glot lists it (CC BY-NC-SA 3.0), the conservative reading.
const GLOT_SUBSOURCES: &[(&str, Option<&str>)] = &[
    ("afromaft", Some("NCGL-UK-2.0")),
    ("afriberta", Some("Apache-2.0")),
    ("leipzig_community", Some("CC-BY-NC-SA-3.0")),
    ("leipzig_news", Some("CC-BY-NC-SA-3.0")),
    ("leipzig_newscrawl", Some("CC-BY-NC-SA-3.0")),
    ("leipzig_wikipedia", Some("CC-BY-NC-SA-3.0")),
    ("nllb_other_tico", Some("CC0-1.0")),
    ("nllb_other_hornmt", Some("CC-BY-4.0")),
    ("flores200", Some("CC-BY-SA-4.0")),
    ("tatoeba", Some("CC-BY-2.0-FR")),
    ("wiki", Some("CC-BY-NC-SA-3.0")),
    ("wikidata", Some("CC0-1.0")),
    ("mtdata", None),
    ("earthlings", None),
    ("bible_crawled", None),
    ("bible_pbc", None),
];

/// Per-sub-source license table for sources that aggregate other datasets.
fn subsource_table(source: &str) -> Option<&'static [(&'static str, Option<&'static str>)]> {
    match source {
        "glot" => Some(GLOT_SUBSOURCES),
        _ => None,
    }
}

/// Track A sources, in registry order. See docs/SOURCES.md.
pub const SOURCES: &[SourceEntry] = &[
    SourceEntry {
        key: "hplt",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "CC0-1.0",
    },
    SourceEntry {
        key: "cc100",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "CC-BY-SA-4.0",
    },
    SourceEntry {
        key: "mc4",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "ODC-BY",
    },
    SourceEntry {
        key: "madlad",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "ODC-BY",
    },
    SourceEntry {
        key: "opus",
        class: SourceClass::Sentence,
        lid_policy: LidPolicy::TagOnly,
        near_dedup: false,
        license_id: "CC0-1.0",
    },
    SourceEntry {
        key: "mt560",
        class: SourceClass::Sentence,
        lid_policy: LidPolicy::TagOnly,
        near_dedup: false,
        license_id: "CC-BY-4.0",
    },
    SourceEntry {
        key: "wikipedia",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "CC-BY-SA-4.0",
    },
    SourceEntry {
        key: "xlsum",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "CC-BY-NC-SA-4.0",
    },
    SourceEntry {
        key: "nllb",
        class: SourceClass::Sentence,
        lid_policy: LidPolicy::TagOnly,
        near_dedup: false,
        license_id: "ODC-BY",
    },
    SourceEntry {
        key: "glot",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "Other",
    },
    SourceEntry {
        key: "somali-web-corpus",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "MIT",
    },
    SourceEntry {
        key: "finepdfs",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "ODC-BY",
    },
    SourceEntry {
        key: "fineweb-2",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "ODC-BY",
    },
    SourceEntry {
        key: "somali-dataset",
        class: SourceClass::Document,
        lid_policy: LidPolicy::Gate,
        near_dedup: true,
        license_id: "MIT",
    },
];

/// Look up a source entry by its registry key.
pub fn lookup(key: &str) -> Option<&'static SourceEntry> {
    SOURCES.iter().find(|entry| entry.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_track_a_keys_present() {
        for key in [
            "hplt",
            "cc100",
            "mc4",
            "madlad",
            "opus",
            "mt560",
            "wikipedia",
            "xlsum",
            "nllb",
            "glot",
            "somali-web-corpus",
            "finepdfs",
            "fineweb-2",
            "somali-dataset",
        ] {
            assert!(lookup(key).is_some(), "missing source: {key}");
        }
    }

    #[test]
    fn sentence_sources_skip_near_dedup() {
        assert!(!lookup("opus").unwrap().near_dedup);
        assert!(!lookup("mt560").unwrap().near_dedup);
        assert_eq!(lookup("opus").unwrap().class, SourceClass::Sentence);
    }

    #[test]
    fn document_sources_use_gate_policy() {
        assert_eq!(lookup("hplt").unwrap().lid_policy, LidPolicy::Gate);
        assert!(lookup("hplt").unwrap().near_dedup);
    }

    #[test]
    fn xlsum_is_non_commercial_share_alike() {
        // The XL-Sum card restricts the data to non-commercial research (CC BY-NC-SA 4.0).
        let license = lookup("xlsum").unwrap().license();
        assert_eq!(
            serde_json::to_string(&license).unwrap(),
            "\"CC-BY-NC-SA-4.0\""
        );
    }

    fn glot_license(subsource: Option<&str>) -> Option<String> {
        lookup("glot")
            .unwrap()
            .record_license(subsource)
            .map(|l| serde_json::to_string(&l).unwrap())
    }

    #[test]
    fn glot_records_take_their_sub_source_license() {
        assert_eq!(glot_license(Some("Afriberta")).as_deref(), Some("\"Apache-2.0\""));
        assert_eq!(glot_license(Some("afromaft")).as_deref(), Some("\"NCGL-UK-2.0\""));
        assert_eq!(
            glot_license(Some("Leipzig_news")).as_deref(),
            Some("\"CC-BY-NC-SA-3.0\"")
        );
        assert_eq!(glot_license(Some("Flores200")).as_deref(), Some("\"CC-BY-SA-4.0\""));
    }

    #[test]
    fn glot_excludes_restricted_and_unknown_sub_sources() {
        for excluded in ["mtdata", "Earthlings", "Bible_crawled", "Bible_pbc"] {
            assert_eq!(glot_license(Some(excluded)), None, "{excluded}");
        }
        assert_eq!(glot_license(Some("some_new_dataset")), None);
        assert_eq!(glot_license(None), None);
    }

    #[test]
    fn other_sources_ignore_the_sub_source() {
        let hplt = lookup("hplt").unwrap();
        assert!(hplt.record_license(None).is_some());
        assert!(hplt.record_license(Some("anything")).is_some());
    }

    #[test]
    fn unknown_key_is_none() {
        assert!(lookup("oscar").is_none());
    }
}
