use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::json;

use crate::stats::Stats;

pub struct JsonlWriter {
    writer: BufWriter<File>,
    progress: ProgressBar,
    pub stats: Stats,
}

impl JsonlWriter {
    pub fn create(path: &Path, label: &str) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }

        let file = File::create(path)
            .with_context(|| format!("creating output file {}", path.display()))?;
        let progress = ProgressBar::new_spinner();
        progress.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg} {pos} docs")
                .context("progress bar template")?
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        progress.set_draw_target(ProgressDrawTarget::stderr());
        progress.set_message(label.to_string());

        Ok(Self {
            writer: BufWriter::new(file),
            progress,
            stats: Stats::default(),
        })
    }

    pub fn write_text(&mut self, text: &str) -> Result<()> {
        let line = serde_json::to_string(&json!({ "text": text }))?;
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.stats.record(text);
        self.progress.inc(1);
        Ok(())
    }

    pub fn write_text_source(&mut self, source: &str, text: &str) -> Result<()> {
        let line = serde_json::to_string(&json!({ "text": text, "source": source }))?;
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.stats.record_source(source, text);
        self.progress.inc(1);
        Ok(())
    }

    /// Write a merged record: text, source key, and the sub-source when the source
    /// aggregates other datasets (e.g. Glot500).
    pub fn write_record(
        &mut self,
        source: &str,
        subsource: Option<&str>,
        text: &str,
    ) -> Result<()> {
        let mut value = json!({ "text": text, "source": source });
        if let Some(subsource) = subsource {
            value["subsource"] = json!(subsource);
        }
        let line = serde_json::to_string(&value)?;
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.stats.record_source(source, text);
        self.progress.inc(1);
        Ok(())
    }

    /// Write a downloaded row with the name of the upstream dataset it came from.
    pub fn write_text_subsource(&mut self, text: &str, subsource: &str) -> Result<()> {
        let line = serde_json::to_string(&json!({ "text": text, "subsource": subsource }))?;
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.stats.record(text);
        self.progress.inc(1);
        Ok(())
    }

    pub fn write_tagged(&mut self, text: &str, source: &str) -> Result<()> {
        self.write_text_source(source, text)
    }

    pub fn finish(self) {
        self.progress.finish_and_clear();
    }
}

/// Lazily-created sidecar for records dropped during merge exact-dedup.
pub struct DroppedWriter {
    path: PathBuf,
    writer: Option<BufWriter<File>>,
    pub count: u64,
}

impl DroppedWriter {
    pub fn for_output(output: &Path) -> Self {
        let stem = output
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "merged".into());
        let path = output.with_file_name(format!("{stem}.dropped.jsonl"));
        Self {
            path,
            writer: None,
            count: 0,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(
        &mut self,
        source: &str,
        text: &str,
        reason: &str,
        kept_source: Option<&str>,
    ) -> Result<()> {
        if self.writer.is_none() {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating directory {}", parent.display()))?;
            }
            let file = File::create(&self.path)
                .with_context(|| format!("creating {}", self.path.display()))?;
            self.writer = Some(BufWriter::new(file));
        }
        let mut row = json!({
            "text": text,
            "source": source,
            "reason": reason,
        });
        if let Some(kept) = kept_source {
            row["kept_source"] = json!(kept);
        }
        let line = serde_json::to_string(&row)?;
        let writer = self.writer.as_mut().unwrap();
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
        self.count += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            writer.flush()?;
        }
        Ok(())
    }
}

pub fn is_non_empty(text: &str) -> bool {
    !text.trim().is_empty()
}

/// Write each non-empty text as one `{"text": ...}` record, stopping at `limit`.
pub fn write_texts(output: &Path, texts: &[String], limit: Option<u64>) -> Result<Stats> {
    let mut writer = JsonlWriter::create(output, "Writing")?;
    let mut written = 0u64;
    for text in texts {
        if limit.is_some_and(|limit| written >= limit) {
            break;
        }
        if !is_non_empty(text) {
            continue;
        }
        writer.write_text(text)?;
        written += 1;
    }
    let stats = writer.stats.clone();
    writer.finish();
    Ok(stats)
}

/// One non-empty raw row: its text and, for aggregate sources, the upstream dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRow {
    pub text: String,
    pub subsource: Option<String>,
}

pub fn read_jsonl_texts(path: &Path) -> Result<impl Iterator<Item = Result<String>> + '_> {
    Ok(read_jsonl_rows(path)?.map(|row| row.map(|row| row.text)))
}

pub fn read_jsonl_rows(path: &Path) -> Result<impl Iterator<Item = Result<RawRow>> + '_> {
    use std::io::{BufRead, BufReader};

    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .enumerate()
        .filter_map(move |(index, line)| match line {
            Ok(raw) => {
                let raw = raw.trim();
                if raw.is_empty() {
                    return None;
                }
                let row = match parse_jsonl_row(raw, path, index + 1) {
                    Ok(row) => row,
                    Err(_) => return None,
                };
                if row.text.is_empty() {
                    return None;
                }
                Some(Ok(row))
            }
            Err(error) => Some(Err(error.into())),
        }))
}

fn parse_jsonl_row(line: &str, path: &Path, line_number: usize) -> Result<RawRow> {
    let value: serde_json::Value = serde_json::from_str(line)
        .with_context(|| format!("invalid JSON on line {line_number} of {}", path.display()))?;
    let text = value
        .get("text")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    let subsource = value
        .get("subsource")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    Ok(RawRow { text, subsource })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_round_trip_with_and_without_sub_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rows.jsonl");
        let mut writer = JsonlWriter::create(&path, "test").unwrap();
        writer
            .write_record("glot", Some("Afriberta"), "alpha beta")
            .unwrap();
        writer.write_record("hplt", None, "gamma delta").unwrap();
        writer.finish();

        let rows: Vec<RawRow> = read_jsonl_rows(&path)
            .unwrap()
            .collect::<Result<_>>()
            .unwrap();
        assert_eq!(
            rows,
            vec![
                RawRow {
                    text: "alpha beta".into(),
                    subsource: Some("Afriberta".into())
                },
                RawRow {
                    text: "gamma delta".into(),
                    subsource: None
                },
            ]
        );
    }
}
