<h1 align="center">Somali NLP Corpus</h1>

<p align="center">
  <strong>A reproducible Somali text corpus and data-processing pipeline for NLP and language-model research.</strong><br>
  <em>Qoraal Soomaali nadiif ah oo loogu talagalay cilmi-baarista AI iyo NLP-ga.</em>
</p>

<p align="center">
  Rust-first · streaming · configuration-driven · source-level provenance
</p>

Somali NLP Corpus downloads public Somali datasets, normalizes and filters their text,
removes exact and near duplicates, and emits training-ready JSONL. The repository also
contains a corpus-trained ByteLevel BPE tokenizer.

*Formerly SomNLP-Corpus.*

## Current release

Version **1.0.0** is the build of **2026-10-19**, combining **14 public sources**:

| Final documents | Words | v2 subword tokens | JSONL size |
| --------------: | ----: | ----------------: | ---------: |
| 7,913,008 | 826,973,689 | 1,129,084,663 | 8.45 GiB |

The pipeline output is `data/final/final_so.jsonl`. Corpus data is not stored in Git;
build it with the pipeline below or download the release from Hugging Face (see
[Release and versions](#release-and-versions)). Trained tokenizer artifacts and their
benchmark summary are tracked for reproducibility.

Public-dataset ingestion, the processing pipeline, and release packaging are complete.
Targeted web collection is next. See [ROADMAP.md](ROADMAP.md) for milestones and
[CHANGELOG.md](CHANGELOG.md) for project history.

## What is included

- Downloaders for HPLT, CC100, mC4, OPUS, MADLAD, MT560, Wikipedia, XL-Sum, NLLB,
  Glot500, Somali Web Corpus, FinePDFs, FineWeb-2, Somali Alpaca (`Somali_dataset`)
- Streaming merge and exact deduplication
- Text normalization, language identification, source-aware deep cleaning, and
  MinHash/LSH near-deduplication
- Per-record provenance, license, content hash, deduplication metadata, and quality
  metadata
- An export that splits the corpus into commercial-use (`open`) and `non-commercial`
  Parquet configurations from each record's own license
- Reject sidecars and per-stage statistics for auditing removed records
- A 48k-vocabulary Somali ByteLevel BPE tokenizer and reproducible training scripts

All pipeline settings live in [`configs/pipeline.toml`](configs/pipeline.toml).

## Pipeline and measured retention

```text
download → merge + exact dedup → clean → LID → deep clean → near dedup → final
raw/       merged/              cleaned/  lid/   deep_clean/  final/
```

| Stage | Documents | Removed at stage |
| ----- | --------: | ---------------: |
| Downloaded | 18,149,257 | — |
| Merged | 11,457,425 | 6,691,832 |
| Cleaned | 8,962,643 | 2,494,782 |
| LID verified | 8,646,184 | 316,459 |
| Deep cleaned | 8,531,896 | 114,288 |
| **Final** | **7,913,008** | **618,888** |

The merge stage also applies the license policy: 303,550 Glot500 rows from sub-sources
without a redistributable license are excluded there. These figures describe the 1.0.0 build
(14 sources) and may change when upstream datasets change. Previous runs: 17 sources,
2026-09-24 (7.98M final · 832M words · 1.14B tokens); 13 sources, 2026-09-02 (7.35M final ·
666M words · 912M tokens). Each local run
writes detailed stage statistics under `reports/`; the timings, per-source funnel, and
final-file validation for this run are in [docs/runs/2026-10-19.md](docs/runs/2026-10-19.md).

Document-class sources use a 25-word minimum, Somali language-ID gating, and near
deduplication. Sentence-class sources use a 5-word minimum and exact deduplication; their
language score is recorded but not used as a rejection gate. The complete behavior and
commands are documented in [`docs/DATA_PIPELINE.md`](docs/DATA_PIPELINE.md).

## Quick start

### Requirements

- Rust 1.75 or newer
- Approximately 45 GB of free disk space for a full build
- Network access to the upstream datasets
- A Hugging Face token for sources that require authentication (`HF_TOKEN` or
  `HUGGING_FACE_HUB_TOKEN`)

Build all tools:

```bash
cargo build --release
```

Run a small end-to-end smoke test:

```bash
./target/release/download_hplt_so --limit 100
./target/release/run_pipeline \
  --stages merge,clean,lid,deep_clean,near_dedup \
  --limit 100
```

The smoke test writes under `data/`. Move or remove that test data before starting a full
build if you want a clean run.

### Build the full corpus

Run the 14 downloaders:

```bash
./target/release/download_hplt_so
./target/release/download_cc100_so
./target/release/download_mc4_so
./target/release/download_opus_so
./target/release/download_madlad_so
./target/release/download_mt560_so
./target/release/download_wikipedia_so
./target/release/download_xlsum_so
./target/release/download_nllb_so
./target/release/download_glot_so
./target/release/download_somali_web_corpus_so
./target/release/download_finepdfs_so
./target/release/download_fineweb2_so
./target/release/download_somali_dataset_so
```

Then run every processing stage:

```bash
./target/release/run_pipeline --config configs/pipeline.toml
```

Generated corpus files and local run reports are ignored by Git. The pipeline is
streaming, but the largest stages still require substantial disk space and runtime.

## Sources and licensing

| Source | Data class | Upstream license |
| ------ | ---------- | ---------------- |
| HPLT 2.0 (`som_Latn`) | document | CC0-1.0 |
| CC-100 Somali | document | CC-BY-SA-4.0 |
| mC4 Somali | document | ODC-BY |
| MADLAD-400 Somali | document | ODC-BY |
| Glot500 (`som_Latn`) | document | per sub-source (Apache-2.0, CC0-1.0, CC-BY-4.0, CC-BY-SA-4.0, CC-BY-NC-SA-3.0, NCGL-UK-2.0) |
| Somali Web Corpus V1 | document | MIT |
| FinePDFs (`som_Latn`) | document | ODC-BY |
| FineWeb-2 (`som_Latn`) | document | ODC-BY |
| Somali Alpaca (`burtugeey/Somali_dataset`) | document | MIT |
| Somali Wikipedia | document | CC-BY-SA-4.0 |
| XL-Sum Somali | document | CC-BY-NC-SA-4.0 |
| OPUS ParaCrawl (`en-so`) | sentence | CC0-1.0 |
| MT560 (`en-so`) | sentence | CC-BY-4.0 |
| NLLB (`eng_Latn-som_Latn`) | sentence | ODC-BY |

> **There is no single corpus-wide license.** Every processed record carries its
> upstream license. Users are responsible for following the terms of each source when
> redistributing or using the combined corpus.

The release export sorts records into two tiers by their own `license` field: `open`
(CC0, CC-BY, CC-BY-SA, ODC-BY, MIT, Apache-2.0; 6,955,948 records) and `non-commercial`
(CC-BY-NC-SA and the UK Non-Commercial Government Licence; 957,060 records). An unclassified
license stops the export instead of defaulting to the open tier.

Excluded for licensing: the QuranEnc and Tanzil Qur'an translations, Somali TinyStories (no
license on its card), and the MTData, Earthlings (GPL-3.0), Bible_crawled, and Bible_pbc rows
of Glot500.
Dataset URLs, access methods, measured scale, and detailed licensing notes are maintained
in [`docs/SOURCES.md`](docs/SOURCES.md). Record-level licensing is described in
[`docs/METADATA_SCHEMA.md`](docs/METADATA_SCHEMA.md).

## Record format

Processed records are newline-delimited JSON with this shape (values abbreviated):

```json
{
  "id": "hplt:a3f8c2…",
  "text": "Soomaaliya waa dal ku yaal Geeska Afrika.",
  "provenance": {
    "source": "hplt",
    "collected_at": "2026-09-01T00:00:00Z",
    "lang": "so",
    "subsource": null
  },
  "license": "CC0-1.0",
  "content_hash": "a3f8c2…",
  "dedup": { "is_duplicate": false, "content_hash": "a3f8c2…" },
  "quality": { "disposition": "kept", "flags": [] },
  "schema_version": 1
}
```

`subsource` names the upstream dataset inside Glot500 and is absent for other sources. See
[`docs/METADATA_SCHEMA.md`](docs/METADATA_SCHEMA.md) for optional provenance, quality, and
metadata fields. Language tags on NLLB rows are detector guesses, not annotations: 107,326
records in the corpus are tagged with a language other than `so`, almost all of them NLLB.

## Tokenizer

The repository includes two trained tokenizer artifacts:

- **v1:** a retained 32k whitespace BPE baseline
- **v2:** the recommended 48k ByteLevel BPE with exact encode/decode round trips and
  byte-level coverage

| Metric | v1 (32k) | **v2 (48k)** |
| ------ | --------: | -----------: |
| Mean tokens/word | 1.3830 | **1.3440** |
| P95 tokens/word | 1.8889 | **1.8333** |
| Bytes/token | 4.766 | **4.835** |
| Round-trip fidelity | 0.000 | **1.000** |
| Documents emitting `<unk>` | 8 | **0** |

Scored on 63,542 held-out documents from the 14-source 1.0.0 corpus, against 1.8100 for
XLM-RoBERTa-base and 2.6344 for BERT-base on the same text. v2's vocabulary was trained on
the earlier 11-source corpus and was not retrained for the sources added since. On the
holdout, FineWeb-2 (1.3394) tokenizes at or below the mean of 1.3440; Somali Alpaca (1.4022)
is slightly above it, and FinePDFs (1.4905) is in line with Wikipedia (1.4971) and mC4
(1.4725), which v2 was trained on.

The 1,129,084,663-token release total (1.3653 tokens/word over the full corpus) comes from
`tokenize_corpus.py`, which encodes every document in the corpus rather than extrapolating
from the holdout ratio.

To run the tokenizer tests and reproduce its preparation, training, and benchmark steps:

```bash
cd tokenizer
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

pytest
python prepare_corpus.py --stats
python train.py
python benchmark.py
python tokenize_corpus.py
```

The training corpus and evaluation split are generated locally and ignored by Git. See
[`tokenizer/PAPER.md`](tokenizer/PAPER.md) for methodology, vocabulary-size experiments,
baseline comparisons, and Hugging Face export instructions.

## Release and versions

The corpus is published as a gated dataset on Hugging Face,
[`somast/somali-nlp-corpus`](https://huggingface.co/datasets/somast/somali-nlp-corpus), with
two configurations: `open` and `non-commercial`. Access requires accepting the terms on the
dataset page. Takedown requests go to info@somast.so.

Versions follow [semantic versioning](https://semver.org): a major version changes the record
schema or removes or relicenses sources, a minor version adds sources or substantial data,
and a patch version covers takedowns, cleaning corrections, and metadata fixes. Each release
is a `vX.Y.Z` tag on this repository and on the Hugging Face dataset.

To cite the corpus, use the "Cite this repository" button on GitHub or
[`CITATION.cff`](CITATION.cff).

To rebuild the release files from a finished `data/final/final_so.jsonl`:

```bash
cd scripts/release
python -m venv .venv && .venv/bin/pip install -r requirements.txt
cd ../..
scripts/release/.venv/bin/python scripts/release/export_release.py          # Parquet + checksums
scripts/release/.venv/bin/python scripts/release/build_card.py --contact info@somast.so
```

## Development

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Contribution workflow and code standards are in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Repository layout

```text
somali-nlp-corpus/
├── configs/pipeline.toml       # pipeline settings and source order
├── crates/
│   ├── common/                 # record types, hashing, source registry
│   ├── corpus-tools/           # downloaders and merge logic
│   └── corpus-pipeline/        # cleaning, LID, dedup, stage runner
├── docs/                       # specifications and technical documentation
├── reports/                    # selected measurements and local run output
├── tokenizer/                  # tokenizer scripts, tests, and trained artifacts
└── data/                       # generated corpus data; not tracked
```

Start with [`docs/DATA_PIPELINE.md`](docs/DATA_PIPELINE.md) to operate the pipeline,
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) to understand the codebase, and
[`docs/SOURCES.md`](docs/SOURCES.md) to evaluate the input data.
