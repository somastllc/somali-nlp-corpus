//! Per-source raw file paths under `data/raw/`. Most sources use
//! `{source}_so.jsonl`.

use std::path::{Path, PathBuf};

/// All JSONL inputs for one registry source key, in read order.
pub fn source_raw_paths(raw_dir: &Path, source: &str) -> Vec<PathBuf> {
    vec![raw_dir.join(source).join(format!("{source}_so.jsonl"))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_source_uses_so_jsonl() {
        let paths = source_raw_paths(Path::new("data/raw"), "hplt");
        assert_eq!(paths, vec![PathBuf::from("data/raw/hplt/hplt_so.jsonl")]);
    }
}
