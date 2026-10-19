# Changelog

All notable changes to Somali NLP Corpus are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [1.0.0] — 2026-10-19

First release of the cleaned corpus as a dataset: 14 sources, per-record licenses, and a
license-tiered Parquet export for Hugging Face.

### Added

- **Fourteen-source measured run (2026-10-19, release 1.0.0)** — full Track A pipeline after the
  deep-clean fixes, the license policy, and phone masking: 18.1M raw → 11.46M merged →
  7,913,008 final documents · 826,973,689 words · **1,129,084,663 v2 subword tokens**
  (1.3653 tokens/word), measured over every document. Stage funnel, timings, per-source drops,
  and a final-file validation: [docs/runs/2026-10-19.md](docs/runs/2026-10-19.md)
- **Per-record licenses.** Every record's `license` is the license of its own upstream dataset.
  Glot500 is an aggregate, so its downloader now keeps the `dataset` column as
  `provenance.subsource` and the license comes from a sub-source table in the registry. The
  sub-sources `mtdata`, `Earthlings`, `Bible_crawled`, and `Bible_pbc` have no redistributable
  license; they are excluded at merge, and so is any row with a missing or unknown sub-source
  (303,550 rows). The corpus no longer contains a record with license `Other`
- **License-tiered release export** (`scripts/release/export_release.py`): writes zstd Parquet
  shards to `data/open` (commercial use allowed, 6,955,948 records) and `data/non-commercial`
  (957,060 records) from each record's license, plus `manifest.json` and `SHA256SUMS`, and
  verifies checksums and row counts. A license it does not recognize stops the export
- **Dataset card and attribution generator** (`scripts/release/build_card.py`): the card for
  `somast/somali-nlp-corpus` (configs, fields, sources with counts, limitations, access terms
  for the gated repository, versioning policy) and `ATTRIBUTION.md` (upstream credits, Glot500
  sub-source table, Common Crawl and share-alike notices). Counts come from the manifest and the
  token statistics, and the build fails if they describe different data
- **Tokenizer holdout re-scored on the 1.0.0 corpus:** 63,542 documents, v1 1.3830 and v2
  **1.3440** mean tokens/word, against 1.8100 (XLM-RoBERTa-base) and 2.6344 (BERT-base). v2 was
  not retrained
- **Phone-number masking** in deep clean (`mask_phones`, on by default): international numbers,
  Somali local mobile numbers, and local numbers that follow a phone keyword become `⟨phone⟩`.
  Thousands separators, ISBNs, and dotted prices are left alone
- **Exact dedup after deep clean.** Deep clean rewrites text, so records that differed before
  can become identical. The deep-clean writer now drops later copies across all sources, first
  seen wins, with reason `exact_duplicate_after_deep_clean` and `duplicate_of` set on the
  reject sidecar record (53,189 records in the 2026-10-19 run)
- **Seventeen-source measured run (2026-09-24)** — full Track A pipeline on all downloaders:
  18.2M raw → 11.71M merged → 7,981,982 final documents ·
  832M words · **1,136,027,958 v2 subword tokens** (1.3650 tokens/word),
  measured over every document. Timings, downloads, and per-source funnel:
  [docs/runs/2026-09-24.md](docs/runs/2026-09-24.md)
- **FinePDFs, FineWeb-2, and Somali Alpaca** registered in
  `crates/common/src/registry.rs` and appended to `merge_source_order`, so first-seen-wins
  exact dedup keeps the earlier sources' copy of any shared text. Near-dedup still keeps the
  longest member of each cluster, so a longer FineWeb-2 page can displace an HPLT or mC4
  near-duplicate. Contributions: FineWeb-2 (`fineweb-2`) 589,824 final documents (18.4% of
  tokens); FinePDFs (`finepdfs`) 20,771 (2.4%); Somali Alpaca (`somali-dataset`) 37,512
  (0.7%) — figures from the 2026-09-24 run; current figures are in the 2026-10-19 report
- `scripts/data/run_all_downloads.sh` (concurrent downloads, one retry, per-source rows and
  bytes) and `scripts/data/run_pipeline_pruned.sh` (stage-by-stage run that deletes each
  superseded intermediate JSONL once the next stage has written its report)
- **Thirteen-source measured run (2026-09-02)** — full Track A pipeline on all downloaders:
  17.0M raw → 10.79M merged → 7,352,961 final documents · 666M words ·
  **911,824,557 v2 subword tokens**, measured over every document rather than extrapolated
  from the holdout ratio
- **Glot500** (`glot`) and **Somali Web Corpus** (`somali-web-corpus`) downloaders, registry
  entries, and pipeline registration. Glot500 contributes 1,433,476 final documents (9.8% of
  tokens); Somali Web Corpus contributes 121,777 (0.7%)
- **`tokenizer/tokenize_corpus.py`** — full-corpus tokenization pass. Shards the corpus by
  byte range across a process pool, keeps counters only, and derives percentiles from a
  mergeable histogram, so peak memory is independent of corpus size
- Incremental measurement log: [reports/runs/MEASUREMENT.md](reports/runs/MEASUREMENT.md)
- **`merge_source_order`** in `configs/pipeline.toml` extended to all thirteen sources
- **Somali BPE tokenizer pipeline** (`tokenizer/`) — prepare, train, benchmark, and export
  scripts using Hugging Face `tokenizers` on the final release corpus
- **v2 tokenizer** (`tokenizer/somali-bpe-v2.json`, 48k ByteLevel BPE) trained on the full
  eleven-source corpus, plus a `transformers`-loadable export at `tokenizer/v2/`
- Deterministic held-out evaluation split (59,029 documents, `blake2b-8(id) mod 1000 < 8`).
  Hashing the record id keeps the split stable as the corpus grows: every id held out of the
  thirteen-source corpus was also held out of the eleven-source corpus v2 trained on
- Per-source composition stats in `tokenizer_stats.json` — document share and word share
  diverge sharply (NLLB is 55.9% of documents but 9.3% of words; Glot500 19.5% against 10.1%)
- Property test suite for the tokenizer (round-trip losslessness, zero-`<unk>` invariant,
  vocabulary-truncation equivalence) plus `pyproject.toml` with ruff and pytest config
- Technical note: [tokenizer/PAPER.md](tokenizer/PAPER.md)

### Removed

- **QuranEnc and Tanzil** (`quran`, `quran-tanzil`): the terms for redistributing the
  translations inside a dataset could not be confirmed. Both downloaders, registry entries and
  merge-order entries are gone; they were 12,845 final records
- **Somali TinyStories** (`somali-tinystories`): its dataset card carries no license, so there
  is no clear right to redistribute it. The downloader, registry entry, merge-order entry and
  download-script wiring are gone. It contributed 21,100 documents (0.3% of tokens)

### Fixed

- **Malformed tags survived deep clean.** Fragments such as `<span;>` and `<br< span="">` were
  not matched by the tag pattern. They are now stripped, and the final file has none
- **Corpus docs claimed NLLB rows carry no language tag.** They carry the detector's guess,
  and 107,326 final records are tagged with a language other than `so`. The documentation now
  says so
- **Deep clean joined words across HTML tags.** Tags were replaced with nothing, so
  `wanaagsan<br>Dadku` became `wanaagsanDadku`. Block-level tags (`br`, `p`, `div`, `li`, ...)
  now leave a line break, and self-closing tags such as `<br/>` are matched
- **OPUS cleanup truncated real text.** It cut everything after the last `<` when a short
  tail contained `>`, dropping visible text that followed a tag. It now removes only a
  complete trailing tag
- **Length floors were not re-checked after deep clean.** Markup and boilerplate removal could
  leave a record far below the 25-word (documents) or 5-word (sentences) floor. The floor is
  applied again before language ID
- **Stale quality flags reached the final output.** `html_remnant`, `mostly_numbers`, and
  `high_symbol_ratio` set by earlier stages are recomputed from the final text. Records on
  review fell from 8,678 to 40
- **Paragraph dedup compared vocabulary, not order.** Different paragraphs with the same words
  scored 1.0, and two paragraphs without long words both scored 1.0 as empty sets. It now
  compares word sequences: identical text always matches, and paragraphs of three or more
  words match on ordered 3-gram Jaccard
- **A lone carriage return was deleted,** gluing the neighbouring words. It now becomes a line
  break
- **Near-dedup was killed by the OOM killer on the 16-source corpus** (18 GB resident). It no
  longer holds document text in memory: clustering uses only shingles, lengths and ids, and the
  input is read a second time to write the output. Output is byte-identical to the previous
  implementation on a 550,000-record sample, with 44% lower peak memory
- `run_pipeline_pruned.sh` used the BSD-only `stat -f`, so its report-freshness check failed on
  Linux

- **Tokenizer decode was destroying whitespace.** v1's `BPEDecoder` was configured to strip
  an `</w>` suffix its trainer never emitted, so `decode(encode(x))` concatenated all tokens:
  `"Soomaaliya waa dal"` → `"Soomaaliyawaadal"`. It fails on **all 59,029** held-out
  documents, and on every document of the full corpus. v2's ByteLevel decoder round-trips exactly (fidelity 1.000).
- **Unknown-token loss.** v1 mapped out-of-alphabet characters to `<unk>` (4 held-out
  documents affected). v2 seeds all 256 bytes, making `<unk>` unreachable.
- **Benchmark memory.** The reservoir sampler held every sampled document in memory at once
  (an estimated 10–14 GB at full corpus size); it now streams and retains only per-document
  ratios.
- **Silent training on a stale corpus.** `train.py` checked only that the training text
  existed. It now compares a corpus fingerprint recorded by `prepare_corpus.py` and refuses
  to proceed on a mismatch unless `--allow-stale` is passed.
- **`train.py` could silently overwrite the shipped v2 artifact.** `--output` defaults to
  `somali-bpe-v2.json`, so a smoke-test run at a small vocabulary replaced the release model.
  It now refuses to overwrite an existing `--output` unless `--force` is passed.
- **`benchmark.py --sweep-dir` silently scored nothing.** Relative paths resolve against the
  repository root, not `tokenizer/`, so `--sweep-dir sweep` globbed a directory that does not
  exist and skipped every sweep candidate without a warning. It now fails with a clear error.
- **`somali_raw_corpus.txt` was never one document per line.** 18% of documents contain
  internal newlines, so v1's published "per-document" ratios were largely per-paragraph.
  The evaluation split is now JSONL, making the unit unambiguous.

### Changed

- **Renamed to Somali NLP Corpus** (repository `somali-nlp-corpus`, dataset
  `somast/somali-nlp-corpus`), formerly SomNLP-Corpus. The new name says what the project is
  and is the same on GitHub and the Hugging Face Hub. Past changelog entries keep the old name.
  The `Cargo.toml` repository URL, which pointed at an old organization, now points at
  `somastllc/somali-nlp-corpus`
- **Pipeline stages use every core.** `clean`, `lid`, `deep_clean`, and `near_dedup` parse,
  process, and serialize records on a rayon pool in batches, while stats, exact dedup, and
  writes stay sequential in input order. Output is byte-identical to the serial path
  (verified on a 249,665-document sample across all stages, sidecars, and reports).
  Full 17-source run: 1 h 18 min end to end
- `Detector` is now `Send + Sync` so one language detector is shared across threads
- Tokenizer figures re-measured on the 17-source corpus: holdout of 64,041 documents,
  v1 1.3834 and v2 **1.3444** mean tokens/word. v2 was **not** retrained;
  FinePDFs is the one new source above the holdout mean by more than 10%
  (1.4906), matching the register of Wikipedia (1.4842) and mC4 (1.4742) and carrying
  2.4% of tokens
- v1 benchmark figures superseded. Under the corrected document-level protocol on the
  thirteen-source corpus, v1 measures 1.3885 mean tokens/word and v2 measures
  **1.3468**, against 2.6336 (BERT-base) and 1.8158 (XLM-RoBERTa).
  The previously published 1.53 is not comparable — it scored lines, not documents.
- Corpus token total is now measured rather than extrapolated. The previous 826M came from
  multiplying the word count by the holdout ratio; encoding every document gives
  **911,824,557**.
- v2 was **not** retrained when the corpus grew to thirteen sources. The two additions
  tokenize better than the corpus mean under the existing merge table — `glot` 1.3370 and
  `somali-web-corpus` 1.2472 against a holdout mean of 1.3468 — so a retrain would move the
  headline by less than the vocabulary sweep's smallest step.
- Test fixture extended to 210 documents spanning all thirteen sources.
- `tokenizer/test_tokenizer.py` renamed to `tokenizer/benchmark.py`; it is a benchmark CLI,
  and the old name caused pytest to collect a file containing no tests.
- `tokenizer/somali-bpe-tokenizer.json` (v1) retained unchanged so its figures stay
  reproducible.

### Known issues

- 107,326 final records have a language tag other than `so`, almost all NLLB. The tag is the
  detector's guess; most of these are short or noisy Somali, but a few thousand with high
  confidence are junk or foreign text.
- 28 final records still hold a phone number after a phone keyword that the masking does not
  cover (12 to 14 digits, such as German mobiles and `00`-prefixed numbers after a dash).
  Names and other personal details in news text are not masked at all.
- 40 final records remain on review (`high_symbol_ratio`).
- The share-alike terms of CC-100, Wikipedia, and FLORES-200 have not had a legal review.
- The v2 tokenizer was trained on the earlier 11-source corpus and not retrained for the
  sources added since; FinePDFs (1.4905 tokens/word on the holdout) and Wikipedia (1.4971) cost
  the most tokens per word.

### Planned

- Hugging Face release packaging (`v0.2-clean`)
- Wikipedia and Somali web collectors
- Books, subtitles, OCR, and community contribution intake

## [0.2.0] — 2026-07-07

### Added

- **v0.2 pipeline topology** — new `deep_clean` stage between LID and near-dedup:
  `merge → clean → lid → deep_clean → near_dedup → final`
- `deep_clean` binary and `[deep_clean]` config section (see `docs/CLEANING_STRATEGY.md`)
- Deep-clean sub-stages: source-aware normalize, HTML/contact masking, boilerplate
  removal, segment-level LID, intra-doc dedup, quality heuristics v2
- Output paths: `data/deep_clean/deep_clean_so.jsonl`; release remains `data/final/final_so.jsonl`
- Stage reports renumbered: `reports/04_deep_clean_stats.json`, `reports/05_near_dedup_stats.json`
- `QualityFlag::Boilerplate` for deep-clean rejects
- Post-run cleaning audit (`reports/06_cleaning_audit.md`)

### Changed

- `near_dedup` default input: `data/deep_clean/deep_clean_so.jsonl` (was `data/lid/lid_so.jsonl`)
- `run_pipeline` runs five post-merge stages including `deep_clean`
- MADLAD export unescape and OPUS HTML escape strip (P0 export fixes)

### Corpus results (full 6-source run)

| Stage | Documents | Removed |
|-------|----------:|--------:|
| Raw (merged input) | 2,633,281 | — |
| Merged | 2,329,800 | 303,481 |
| Cleaned | 2,225,791 | 104,009 |
| LID verified | 2,035,287 | 190,504 |
| Deep cleaned | 2,003,228 | 32,059 |
| **Final** | **1,668,080** | 335,148 |

- **528,853,952 words** · **~793M subword tokens** (×1.5) · **~4.0 GB** `data/final/final_so.jsonl`
- Deep-clean reject breakdown: boilerplate 23,948 · not_somali 6,906 · too_long 1,060 ·
  mostly_numbers 117 · html_remnant 23
- v0.1 → v0.2: 1,774,891 → 1,668,080 docs (−6.0%); 591M → 529M words (−10.5%)
- Audit highlights: URL remnants 18.8% → 0.07%; escaped `\n` 10.5% → 0.01%;
  boilerplate 4.3% → 1.06%; URLs masked to `⟨url⟩` sentinel in 16.17% of docs

## [0.1.0] — 2026-07-07

### Added

- Rust workspace with `common` and `corpus-tools` crates
- Minimal `Document` record type (`text` + optional `source`)
- Public dataset downloaders: HPLT, CC100, mC4, OPUS, MADLAD, MT560
- `merge_corpora` tool to combine raw JSONL sources
- `corpus-pipeline` crate: clean, LID (`lingua`), near-dedup (MinHash + LSH)
- `run_pipeline` stage runner and per-stage stats reports
- Documentation: architecture, pipeline, plan, roadmap, source catalog

### Corpus results (v0.1 baseline, no deep clean)

- 2.63M raw → **1,774,891** final documents · **591M words** · **~887M tokens** · ~4.5 GB
