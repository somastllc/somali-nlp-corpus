# Source Registry

Formal catalog of all data sources for Somali NLP Corpus. Each source has a stable
registry key used in metadata, IDs, and file paths.

See also: [METADATA_SCHEMA.md](METADATA_SCHEMA.md), [PLAN.md](../PLAN.md).

**Status:** `done` · `planned` · `skipped`

---

## Registry — Track A (public datasets)

| Key | Name | Category | Track | Est. scale | License | Tool | Status | Notes |
|-----|------|----------|:-----:|------------|---------|------|--------|-------|
| `hplt` | HPLT2.0 cleaned `som_Latn` | web crawl | A | ~505M tokens, ~966K docs | CC0-1.0 | `download_hplt_so` | done | Primary backbone; dominates raw token count |
| `cc100` | CC-100 Somali | web crawl | A | ~81M tokens, ~396K docs | CC-BY-SA-4.0 | `download_cc100_so` | done | ~0.12% overlap with HPLT; net-additive |
| `mc4` | mC4 `so` | web crawl | A | tens of millions | ODC-BY | `download_mc4_so` | done | Overlaps CC ancestry with HPLT; dedup required |
| `madlad` | MADLAD-400 `so` | web crawl | A | tens of millions | ODC-BY | `download_madlad_so` | done | Clean split default; `--include-noisy` optional |
| `opus` | OPUS ParaCrawl `en-so` | parallel text | A | parallel sentences | CC0-1.0 | `download_opus_so` | done | Somali column from `translation.so` |
| `mt560` | MT560 en–so pairs | parallel / religious | A | ~161K pairs | CC-BY-4.0 | `download_mt560_so` | done | Writes `source: mt560` tag in raw JSONL |
| `wikipedia` | Somali Wikipedia | encyclopedia | A | 9,021 articles (measured) | CC-BY-SA-4.0 | `download_wikipedia_so` | done | HF `wikimedia/wikipedia` dataset (`20231101.so`) |
| `xlsum` | XL-Sum Somali | news / summary | A | 7,452 articles (measured) | CC-BY-NC-SA-4.0 | `download_xlsum_so` | done | `csebuetnlp/xlsum` (`somali` config) |
| `nllb` | NLLB English–Somali | parallel text | A | 10.2M pairs raw · 4.4M kept at merge | ODC-BY | `download_nllb_so` | done | Official Meta/AllenAI NLLB TSV export |
| `glot` | Glot500 Somali (`som_Latn`) | web crawl / aggregated | A | ~3.9M docs (HF train) | per record (sub-source) | `download_glot_so` | done | HF `cis-lmu/Glot500` parquet export |
| `somali-web-corpus` | Somali Web Corpus V1 | web crawl | A | ~218K docs (HF train) | MIT | `download_somali_web_corpus_so` | done | HF `maanka2/somali-web-corpus` parquet export |
| `finepdfs` | FinePDFs Somali (`som_Latn`) | PDF documents | A | 21,781 docs (HF train) | ODC-BY | `download_finepdfs_so` | done | HF `HuggingFaceFW/finepdfs` parquet export; long documents |
| `fineweb-2` | FineWeb-2 Somali (`som_Latn`) | web crawl | A | 1,070,384 docs (HF train) | ODC-BY | `download_fineweb2_so` | done | HF `HuggingFaceFW/fineweb-2` parquet export; heavy overlap with HPLT/mC4 |
| `somali-dataset` | Somali Alpaca instructions | instruction data | A | 44,839 rows (HF train) | MIT | `download_somali_dataset_so` | done | HF `burtugeey/Somali_dataset`; instruction, input, and output joined per row |

### Excluded sources

- `somali-tinystories` (Hugging Face `Zyroxx66/somali-tinystories`, 42,000 stories): its
  dataset card states no license, so redistribution is not clear.
- `quran` (QuranEnc, Yacob Yusuf translation) and `tanzil` (Tanzil.net, Abduh translation):
  the upstream terms could not be confirmed as allowing redistribution inside a dataset, so
  both were dropped together with their downloaders. Together they were 12,845 final records.
- Four Glot500 sub-sources are excluded at merge, 303,550 rows in all: `mtdata` (197,228),
  `Earthlings` (76,762, GPL-3.0), `Bible_crawled` (29,557), and `Bible_pbc` (3).
  A Glot500 row with a missing or unknown sub-source is excluded too.

### Track A licensing note

There is **no single corpus license**. Redistribution of the combined corpus requires
honoring each upstream license. The `license` field on processed records is copied
from this registry (see [METADATA_SCHEMA.md](METADATA_SCHEMA.md)). Glot500 is an aggregate:
its records take the license of their sub-source, recorded in `provenance.subsource`.

| Tier | Licenses | Final records |
|------|----------|--------------:|
| `open` (commercial use allowed) | ODC-BY 5,476,490 · CC0-1.0 568,494 · Apache-2.0 399,434 · CC-BY-SA-4.0 302,154 · MIT 159,055 · CC-BY-4.0 50,321 | 6,955,948 |
| `non-commercial` | NCGL-UK-2.0 860,590 · CC-BY-NC-SA-3.0 91,443 · CC-BY-NC-SA-4.0 5,027 | 957,060 |

The release export builds these two Hugging Face configurations from the per-record field.
Share-alike obligations (CC-BY-SA, CC-BY-NC-SA) and the Common Crawl terms of use still apply
to the records they cover; the release `ATTRIBUTION.md` lists them.

### Track A outlook

Fourteen shipped public sources total **18.1M raw documents** (NLLB dominates). The
fourteen-source Track A corpus (release 1.0.0) measured **7,913,008 final documents · 827M
words · 1,129M v2 tokens** (2026-10-19 run — see [README.md](../README.md)). Previous runs:
seventeen sources (2026-09-24) · 7.98M final documents · 832M words · 1,136M v2 tokens, and
thirteen sources (2026-09-02) · 7.35M final documents · 666M words · 912M v2 tokens. Incremental measurement
notes: [reports/runs/MEASUREMENT.md](../reports/runs/MEASUREMENT.md).

### Measured final documents (2026-10-19, 1.0.0)

| Key | Raw | Kept at merge | Final | Words | v2 tokens |
|-----|----:|-------------:|------:|------:|----------:|
| `nllb` | 10,229,073 | 4,416,610 | 4,151,034 | 62,757,823 | 83,017,929 |
| `glot` | 3,915,898 | 3,555,894 | 1,354,751 | 63,617,306 | 84,989,034 |
| `fineweb-2` | 1,070,384 | 854,713 | 589,888 | 156,978,595 | 208,761,477 |
| `mc4` | 893,012 | 892,852 | 586,110 | 214,033,522 | 313,655,232 |
| `hplt` | 966,507 | 798,364 | 554,929 | 184,729,346 | 247,476,026 |
| `cc100` | 396,524 | 374,721 | 296,221 | 49,299,863 | 61,322,715 |
| `madlad` | 200,494 | 200,484 | 128,708 | 61,837,547 | 82,261,565 |
| `somali-web-corpus` | 217,528 | 217,390 | 121,792 | 4,891,698 | 6,112,389 |
| `mt560` | 161,865 | 51,083 | 49,195 | 1,162,745 | 1,423,242 |
| `somali-dataset` | 44,839 | 44,793 | 37,263 | 5,777,922 | 7,969,383 |
| `finepdfs` | 21,781 | 21,780 | 20,750 | 18,626,664 | 27,654,839 |
| `opus` | 14,879 | 12,296 | 12,125 | 381,720 | 517,110 |
| `wikipedia` | 9,021 | 8,994 | 5,215 | 1,120,131 | 1,733,899 |
| `xlsum` | 7,452 | 7,451 | 5,027 | 1,758,807 | 2,189,823 |
| **Total** | **18,149,257** | **11,457,425** | **7,913,008** | **826,973,689** | **1,129,084,663** |

Raw, merge, and final counts come from `reports/01_merge_stats.json` and
`reports/05_near_dedup_stats.json`; words and tokens from `tokenizer/corpus_token_stats.json`.
Glot500 raw includes the 303,550 rows excluded by license policy. Per-stage drops and timings:
[runs/2026-10-19.md](runs/2026-10-19.md).

### Measured final documents (2026-09-24, 17 sources, superseded)

| Key | Raw | Kept at merge | Final | Words | v2 tokens |
|-----|----:|-------------:|------:|------:|----------:|
| `nllb` | 10,229,073 | 4,372,863 | 4,108,233 | 62,072,423 | 82,095,163 |
| `glot` | 3,915,898 | 3,840,628 | 1,432,277 | 66,937,207 | 89,396,283 |
| `fineweb-2` | 1,070,384 | 854,429 | 589,824 | 156,995,286 | 208,794,537 |
| `mc4` | 893,012 | 892,852 | 586,265 | 214,035,586 | 313,609,489 |
| `hplt` | 966,507 | 798,161 | 554,851 | 184,700,703 | 247,457,800 |
| `cc100` | 396,524 | 374,720 | 296,154 | 49,297,018 | 61,321,488 |
| `madlad` | 200,494 | 177,030 | 128,700 | 61,831,676 | 82,255,318 |
| `somali-web-corpus` | 217,528 | 217,390 | 121,764 | 4,890,563 | 6,111,014 |
| `mt560` | 161,865 | 51,083 | 49,195 | 1,162,745 | 1,423,242 |
| `somali-dataset` | 44,839 | 44,793 | 37,512 | 5,781,223 | 7,974,652 |
| `finepdfs` | 21,781 | 21,780 | 20,771 | 18,625,820 | 27,654,658 |
| `opus` | 14,879 | 12,296 | 12,126 | 381,716 | 517,056 |
| `quran` | 7,373 | 7,277 | 7,072 | 160,068 | 240,053 |
| `quran-tanzil` | 6,236 | 6,048 | 5,773 | 106,282 | 164,038 |
| `wikipedia` | 9,021 | 8,994 | 5,338 | 1,130,331 | 1,748,915 |
| `xlsum` | 7,452 | 7,451 | 5,027 | 1,758,807 | 2,189,823 |
| **Total** | **18,204,866** | **11,708,895** | **7,981,982** | **832,264,349** | **1,136,027,958** |

Raw, merge, and final counts come from `reports/01_merge_stats.json` and
`reports/05_near_dedup_stats.json`; words and tokens from `tokenizer/corpus_token_stats.json`
(which skips the 6 empty records).

### Measured final documents (2026-08-31)

| Key | Raw | Kept at merge | Final |
|-----|----:|-------------:|------:|
| `nllb` | 10,229,073 | 4,727,602 | 4,463,021 |
| `mc4` | 893,012 | 892,852 | 595,193 |
| `hplt` | 966,507 | 798,364 | 576,995 |
| `cc100` | 396,524 | 374,721 | 300,664 |
| `madlad` | 200,494 | 200,484 | 133,410 |
| `mt560` | 161,865 | 51,083 | 49,195 |
| `opus` | 14,879 | 12,296 | 12,126 |
| `quran` | 7,373 | 7,277 | 7,072 |
| `xlsum` | 7,452 | 7,451 | 5,649 |
| `wikipedia` | 9,021 | 8,994 | 5,503 |
| `quran-tanzil` | 6,236 | 6,048 | 5,773 |
| **Total** | **12,892,436** | **7,087,172** | **6,154,601** |

---

## Registry — Track B (collected sources)

| Key | Name | Category | Track | Est. scale | License | Tool | Status | Notes |
|-----|------|----------|:-----:|------------|---------|------|--------|-------|
| `web` | Somali web scraping | news / blogs / gov | B | ~40M words | per-site | collector (planned) | planned | ~100 sites; robots.txt required |
| `wikimedia` | Wiktionary / Wikiquote / Wikinews | reference | B | small | CC-BY-SA-4.0 | collector (planned) | planned | Supplement to Wikipedia |
| `books` | Books & educational materials | literature / education | B | 10–20M tokens | per-work | collector (planned) | planned | Public domain or author-approved |
| `social` | Social media & forums | informal | B | 5–15M tokens | per-platform | collector (planned) | planned | Heavy cleaning required |
| `subtitles` | Video subtitles | conversational | B | 5–10M tokens | per-collection | collector (planned) | planned | Movies, educational video |
| `ocr` | OCR digitization | historical | B | 5–15M tokens | per-work | collector (planned) | planned | Scanned books, newspapers |
| `community` | Community contributions | mixed | B | ongoing | per-submission | intake (planned) | planned | Stories, poems, essays |

---

## Per-source detail

### `hplt`

- **Upstream:** [HPLT/HPLT2.0_cleaned](https://huggingface.co/datasets/HPLT/HPLT2.0_cleaned) config `som_Latn`
- **Access:** Hugging Face parquet shards
- **Output:** `data/raw/hplt/hplt_so.jsonl`
- **License:** CC0-1.0 (verify on upstream dataset card before release)

### `cc100`

- **Upstream:** [CC-100 Somali](https://data.statmt.org/cc-100/so.txt.xz)
- **Access:** Direct HTTP (xz compressed)
- **Output:** `data/raw/cc100/cc100_so.jsonl`
- **License:** CC-BY-SA-4.0

### `mc4`

- **Upstream:** [allenai/c4](https://huggingface.co/datasets/allenai/c4) config `so`
- **Access:** Hugging Face gzip-json shards (enumerated)
- **Output:** `data/raw/mc4/mc4_so.jsonl`
- **License:** ODC-BY (verify on upstream)

### `madlad`

- **Upstream:** [allenai/MADLAD-400](https://huggingface.co/datasets/allenai/MADLAD-400) language `so`
- **Access:** Hugging Face jsonl.gz shards
- **Output:** `data/raw/madlad/madlad_so.jsonl`
- **License:** ODC-BY (verify on upstream)

### `opus`

- **Upstream:** [Helsinki-NLP/opus_paracrawl](https://huggingface.co/datasets/Helsinki-NLP/opus_paracrawl) config `en-so`
- **Access:** Hugging Face parquet; field `translation.so`
- **Output:** `data/raw/opus/opus_so.jsonl`
- **License:** CC0-1.0 (verify on upstream)

### `mt560`

- **Upstream:** [english-somali_sentence-pairs_mt560](https://huggingface.co/datasets/michsethowusu/english-somali_sentence-pairs_mt560)
- **Access:** Hugging Face parquet; column `som`
- **Output:** `data/raw/mt560/mt560_so.jsonl`
- **License:** CC-BY-4.0 (verify on upstream)

### `wikipedia`

- **Upstream:** [wikimedia/wikipedia](https://huggingface.co/datasets/wikimedia/wikipedia) config `20231101.so`
- **Access:** Hugging Face parquet shards
- **Output:** `data/raw/wikipedia/wikipedia_so.jsonl`
- **License:** CC-BY-SA-4.0

### `xlsum`

- **Upstream:** [csebuetnlp/xlsum](https://huggingface.co/datasets/csebuetnlp/xlsum) config `somali`
- **Access:** Hugging Face parquet auto-converted revision (`PARQUET_REVISION`)
- **Output:** `data/raw/xlsum/xlsum_so.jsonl`
- **License:** CC-BY-NC-SA-4.0 (non-commercial; the upstream card says CC BY-NC-SA 4.0)

### `glot`

- **Upstream:** [cis-lmu/Glot500](https://huggingface.co/datasets/cis-lmu/Glot500) config `som_Latn`
- **Access:** Hugging Face parquet export (`refs/convert/parquet`); train split
- **Output:** `data/raw/glot/glot_so.jsonl`; each row keeps the Glot500 `dataset` column as `subsource`
- **License:** per sub-source, from the Glot500-c source table (see `GLOT_SUBSOURCES` in
  `crates/common/src/registry.rs`). Kept: `Afriberta` (Apache-2.0), `afromaft` (NCGL-UK-2.0),
  `Leipzig_*` and `wiki` (CC-BY-NC-SA-3.0), `Flores200` (CC-BY-SA-4.0), `nllb_other_hornmt`
  (CC-BY-4.0), `nllb_other_tico` (CC0-1.0), `tatoeba` (CC-BY-2.0-FR) and `wikidata` (CC0-1.0) where present. Excluded:
  `mtdata`, `Earthlings`, `Bible_crawled`, `Bible_pbc`, and any row with a missing or unknown sub-source

### `somali-web-corpus`

- **Upstream:** [maanka2/somali-web-corpus](https://huggingface.co/datasets/maanka2/somali-web-corpus) config `default`
- **Access:** Hugging Face parquet export (`refs/convert/parquet`); train split (~218K rows)
- **Output:** `data/raw/somali-web-corpus/somali-web-corpus_so.jsonl`
- **License:** MIT

### `finepdfs`

- **Upstream:** [HuggingFaceFW/finepdfs](https://huggingface.co/datasets/HuggingFaceFW/finepdfs) subset `som_Latn`
- **Access:** Hugging Face parquet export (`refs/convert/parquet`); `som_Latn/train` shards, `text` column
- **Output:** `data/raw/finepdfs/finepdfs_so.jsonl`
- **License:** ODC-BY

### `fineweb-2`

- **Upstream:** [HuggingFaceFW/fineweb-2](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2) subset `som_Latn`
- **Access:** Hugging Face parquet export (`refs/convert/parquet`); `som_Latn/train` shards, `text` column
- **Output:** `data/raw/fineweb-2/fineweb-2_so.jsonl`
- **License:** ODC-BY
- **Overlap:** built from Common Crawl like HPLT and mC4; 215,950 rows are exact
  cross-source duplicates at merge and 235,557 more are removed by near-dedup (2026-09-24)

### `somali-dataset`

- **Upstream:** [burtugeey/Somali_dataset](https://huggingface.co/datasets/burtugeey/Somali_dataset) config `default`
- **Access:** Hugging Face parquet export (`refs/convert/parquet`); train split. Each row's
  non-empty `instruction`, `input`, and `output` are joined with a blank line
- **Output:** `data/raw/somali-dataset/somali-dataset_so.jsonl`
- **License:** MIT (dataset card)


### `nllb`

- **Upstream:** [allenai/nllb](https://storage.googleapis.com/allennlp-data-bucket/nllb/eng_Latn-som_Latn.gz)
- **Access:** Direct Google Cloud Storage bucket gzip stream
- **Output:** `data/raw/nllb/nllb_so.jsonl`
- **License:** ODC-BY


---

## Output paths

| Stage | Path pattern |
|-------|--------------|
| Per-source raw | `data/raw/<key>/<key>_so.jsonl` |
| Merged raw | `data/merged/merged_so.jsonl` |
| Cleaned | `data/cleaned/cleaned_so.jsonl` |
| LID verified | `data/lid/lid_so.jsonl` |
| Deep clean (v0.2) | `data/deep_clean/deep_clean_so.jsonl` |
| Final (near dedup) | `data/final/final_so.jsonl` |
| Rejected (sidecar) | `data/<stage>/*.rejected.jsonl` |

---

## Adding a new source

1. Choose a stable registry key (lowercase, no spaces).
2. Add a row to this table with license, scale estimate, and tool.
3. Implement downloader or collector.
4. Update [METADATA_SCHEMA.md](METADATA_SCHEMA.md) if new `meta` fields are needed.
5. Document overlap expectations with existing sources.
