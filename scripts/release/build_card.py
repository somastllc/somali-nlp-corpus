#!/usr/bin/env python3
"""Write the dataset card (README.md) and ATTRIBUTION.md for a release export.

All counts come from ``manifest.json`` (written by export_release.py) and from
``tokenizer/corpus_token_stats.json`` (words and tokens per source), so the documents
cannot disagree with the data they describe.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

DEFAULT_RELEASE = Path("release")
DEFAULT_TOKEN_STATS = Path("tokenizer/corpus_token_stats.json")
DEFAULT_REPO_ID = "somast/somali-nlp-corpus"
DEFAULT_CARGO_TOML = Path("Cargo.toml")

# Upstream details per source key: display name, URL, and what the upstream asks of users.
SOURCES = {
    "hplt": ("HPLT 2.0 cleaned (som_Latn)", "https://huggingface.co/datasets/HPLT/HPLT2.0_cleaned",
             "Packaging under CC0. HPLT does not own the underlying text; takedown requests go to hplt-datasets@ufal.mff.cuni.cz."),
    "cc100": ("CC-100 Somali", "https://data.statmt.org/cc-100/",
              "Derived from Common Crawl; subject to the Common Crawl terms of use."),
    "mc4": ("mC4 / C4 multilingual (so)", "https://huggingface.co/datasets/allenai/c4",
            "ODC-BY; also bound by the Common Crawl terms of use."),
    "madlad": ("MADLAD-400 (so)", "https://huggingface.co/datasets/allenai/MADLAD-400",
               "The upstream card gives both ODC-BY (metadata) and CC-BY-4.0 (text); recorded as ODC-BY. Derived from Common Crawl."),
    "fineweb-2": ("FineWeb-2 (som_Latn)", "https://huggingface.co/datasets/HuggingFaceFW/fineweb-2",
                  "ODC-BY; derived from Common Crawl and subject to its terms of use."),
    "finepdfs": ("FinePDFs (som_Latn)", "https://huggingface.co/datasets/HuggingFaceFW/finepdfs",
                 "ODC-BY. Text extracted from PDF documents."),
    "nllb": ("NLLB English-Somali (Somali side)", "https://storage.googleapis.com/allennlp-data-bucket/nllb/eng_Latn-som_Latn.gz",
             "ODC-BY (AllenAI NLLB mined bitext)."),
    "opus": ("OPUS ParaCrawl en-so (Somali side)", "https://huggingface.co/datasets/Helsinki-NLP/opus_paracrawl",
             "Packaging under CC0; the crawled web text keeps its owners' rights."),
    "mt560": ("MT560 English-Somali (Somali side)", "https://huggingface.co/datasets/michsethowusu/english-somali_sentence-pairs_mt560",
              "CC-BY-4.0 per the dataset card; cite the OPUS MT560 collection."),
    "somali-web-corpus": ("Somali Web Corpus V1", "https://huggingface.co/datasets/maanka2/somali-web-corpus",
                          "MIT for the compilation. Articles were scraped from BBC Somali, Shabeele Media, SONNA and others, whose copyright is unaffected."),
    "somali-dataset": ("Somali Alpaca instructions", "https://huggingface.co/datasets/burtugeey/Somali_dataset",
                       "MIT per the dataset card; the card gives no information about how the data was produced."),
    "wikipedia": ("Somali Wikipedia", "https://huggingface.co/datasets/wikimedia/wikipedia",
                  "CC BY-SA 4.0 (text by Wikipedia contributors). Adaptations must stay under the same license."),
    "xlsum": ("XL-Sum Somali", "https://huggingface.co/datasets/csebuetnlp/xlsum",
              "CC BY-NC-SA 4.0: non-commercial research only. News text copyright belongs to the BBC and original holders."),
    "glot": ("Glot500 (som_Latn)", "https://huggingface.co/datasets/cis-lmu/Glot500",
             "Aggregate of several corpora, each under its own license; see the per-sub-source table."),
}

GLOT_NOTES = {
    "afromaft": ("AfroMAFT", "UK Non-Commercial Government Licence v2"),
    "afriberta": ("AfriBERTa", "Apache License 2.0"),
    "leipzig_community": ("Leipzig Corpora Collection", "CC BY-NC-SA 3.0"),
    "leipzig_news": ("Leipzig Corpora Collection", "CC BY-NC-SA 3.0"),
    "leipzig_newscrawl": ("Leipzig Corpora Collection", "CC BY-NC-SA 3.0"),
    "leipzig_wikipedia": ("Leipzig Corpora Collection", "CC BY-NC-SA 3.0"),
    "wiki": ("Wikipedia (as listed by Glot500)", "CC BY-NC-SA 3.0"),
    "wikidata": ("Wikidata", "CC0 1.0"),
    "nllb_other_tico": ("TICO-19", "CC0 1.0"),
    "nllb_other_hornmt": ("HornMT", "CC BY 4.0"),
    "flores200": ("FLORES-200", "CC BY-SA 4.0"),
    "tatoeba": ("Tatoeba", "CC BY 2.0 FR / CC0 1.0"),
}

CARD_HEADER = """---
license: other
license_name: per-record
license_link: https://huggingface.co/datasets/{repo}/blob/main/ATTRIBUTION.md
language:
- so
pretty_name: Somali NLP Corpus
size_categories:
- 1M<n<10M
task_categories:
- text-generation
- fill-mask
tags:
- somali
- low-resource
- pretraining
configs:
- config_name: open
  data_files: data/open/*.parquet
  default: true
- config_name: non-commercial
  data_files: data/non-commercial/*.parquet
extra_gated_prompt: >-
  This corpus mixes text under many licenses. Each record names its own license in the
  `license` field, and you are responsible for following it. Records in the
  `non-commercial` configuration may not be used commercially. By requesting access you
  also agree not to try to identify private individuals from the text and to honor
  takedown requests sent to {contact}.
extra_gated_fields:
  Name: text
  Organization: text
  Intended use:
    type: select
    options:
    - Research
    - Education
    - Commercial product
    - Other
  I will follow the license of every record I use: checkbox
  I will not try to identify private individuals: checkbox
---
"""

VERSIONING = """## Versions

Releases use [semantic versioning](https://semver.org). Each release is a git tag
(`v{version}`) on this repository and on the GitHub project, and `main` always holds the
latest one.

- **Major**: a change to the record schema, or a removal or relicensing that breaks existing users.
- **Minor**: new sources or a substantially larger corpus.
- **Patch**: takedowns, cleaning corrections, and metadata fixes.

Pin a release with `load_dataset("{repo}", revision="v{version}")`. This card describes
version **{version}**; see the changelog at <https://github.com/somastllc/somali-nlp-corpus/blob/main/CHANGELOG.md>.
"""


def human(n: int) -> str:
    return f"{n:,}"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def source_rows(manifest: dict, stats: dict) -> list[tuple[str, str, int, int, int]]:
    """One row per source: key, tier(s), records, words, tokens."""
    per_source: dict[str, dict] = {}
    for tier, info in manifest["tiers"].items():
        for source, records in info["by_source"].items():
            entry = per_source.setdefault(source, {"records": 0, "tiers": set()})
            entry["records"] += records
            entry["tiers"].add(tier)
    rows = []
    by_source_stats = stats["tokenizers"]["v2"]["by_source"]
    for source, entry in sorted(per_source.items(), key=lambda kv: -kv[1]["records"]):
        s = by_source_stats.get(source, {})
        rows.append(
            (
                source,
                " + ".join(sorted(entry["tiers"])),
                entry["records"],
                s.get("words", 0),
                s.get("tokens", 0),
            )
        )
    return rows


def check_consistent(manifest: dict, stats: dict) -> None:
    """The token statistics must describe exactly the data that was exported."""
    exported = manifest["records"]
    measured = stats["tokenizers"]["v2"]["documents"]
    if exported != measured:
        raise SystemExit(
            f"manifest has {exported:,} records but token stats cover {measured:,} documents; "
            "re-run tokenize_corpus.py on the final file"
        )


def build_card(manifest: dict, stats: dict, repo: str, contact: str, version: str) -> str:
    check_consistent(manifest, stats)
    total = manifest["records"]
    tv2 = stats["tokenizers"]["v2"]
    open_info = manifest["tiers"]["open"]
    nc_info = manifest["tiers"]["non-commercial"]
    lines = [CARD_HEADER.format(repo=repo, contact=contact)]
    lines.append("# Somali NLP Corpus\n")
    lines.append(
        f"A cleaned, deduplicated Somali text corpus for NLP and language-model research: "
        f"**{human(total)} documents, {human(tv2['words'])} words, "
        f"{human(tv2['total_tokens'])} tokens** (v2 ByteLevel BPE, "
        f"{tv2['total_tokens'] / tv2['words']:.3f} tokens per word) "
        f"from {len(set(open_info['by_source']) | set(nc_info['by_source']))} public sources.\n"
    )
    lines.append("## Two license tiers\n")
    lines.append(
        "There is **no single license for this corpus**. Each record keeps the license of the "
        "dataset it came from, in its `license` field, and records are split into two configurations:\n"
    )
    lines.append(
        f"| Config | Records | Use |\n|---|---:|---|\n"
        f"| `open` | {human(open_info['records'])} | Licenses that permit commercial use "
        f"(CC0, CC-BY, CC-BY-SA, ODC-BY, MIT, Apache-2.0). Attribution and, for CC-BY-SA, "
        f"share-alike still apply. |\n"
        f"| `non-commercial` | {human(nc_info['records'])} | Licenses that restrict use to "
        f"non-commercial purposes (CC-BY-NC-SA, UK Non-Commercial Government Licence). |\n"
    )
    lines.append(
        "```python\nfrom datasets import load_dataset\n"
        f"open_data = load_dataset(\"{repo}\", \"open\", split=\"train\")\n```\n"
    )
    lines.append(
        "You are responsible for following each record's license. Filter on `license` or "
        "`source` if you need a narrower set. See `ATTRIBUTION.md` for required credits.\n"
    )
    lines.append("## Fields\n")
    lines.append(
        "| Field | Description |\n|---|---|\n"
        "| `id` | Stable id, `<source>:<hash prefix>` |\n"
        "| `text` | The document or sentence |\n"
        "| `source` | Source key (see table below) |\n"
        "| `subsource` | Upstream dataset inside an aggregate source (Glot500); otherwise null |\n"
        "| `license` | License of this record's upstream dataset |\n"
        "| `detected_language` | Language the detector guessed (mostly `so`; see limitations) |\n"
        "| `lang_score` | Detector confidence for that guess |\n"
        "| `content_hash` | SHA-256 of the normalized text |\n"
        "| `quality_flags` | Non-fatal flags, for example `high_symbol_ratio` |\n"
    )
    lines.append("## Sources\n")
    lines.append("| Source | Config | Records | Words | Tokens |\n|---|---|---:|---:|---:|")
    for source, tier, records, words, tokens in source_rows(manifest, stats):
        name = SOURCES.get(source, (source, "", ""))[0]
        lines.append(f"| {name} (`{source}`) | {tier} | {human(records)} | {human(words)} | {human(tokens)} |")
    lines.append("")
    lines.append("## How it was built\n")
    lines.append(
        "Public datasets are downloaded, merged with exact deduplication across sources "
        "(first seen wins), then cleaned (entities, mojibake, Unicode NFC, control characters, "
        "whitespace, length floors), language-identified (document sources must be Somali), "
        "deep-cleaned (HTML, boilerplate, URL/email/phone masking, paragraph-level language check, "
        "a second exact dedup), and near-deduplicated (MinHash/LSH, Jaccard 0.80, keeping the longest "
        "copy). Sentence sources (NLLB, OPUS, MT560) use a 5-word floor and exact dedup only. "
        "Rejected records are kept in sidecars in the pipeline repository. The pipeline is Rust, "
        "configuration-driven, and reproducible: <https://github.com/somastllc/somali-nlp-corpus>.\n"
    )
    lines.append("## Known limitations\n")
    lines.append(
        "- **Web text is not owned by the corpus.** Dataset licenses cover compilation; the "
        "underlying articles belong to their publishers (BBC Somali and others).\n"
        "- **Language tags are detector guesses.** About 3% of NLLB sentences were tagged as a "
        "language other than Somali. Most have low confidence and are short or noisy Somali, but "
        "a few thousand with high confidence are junk or foreign text. Use `detected_language` and "
        "`lang_score` to filter.\n"
        "- **Personal data is masked on a best-effort basis.** URLs, emails and common phone "
        "formats are replaced with `⟨url⟩`, `⟨email⟩` and `⟨phone⟩`. A few dozen unusual phone "
        "formats (for example 12-digit numbers after a `tel` label) remain, and names and other "
        "personal details in news text are not removed.\n"
        "- **Quality is measured automatically, not by human review.** Dialect and register "
        "coverage were not assessed.\n"
        "- **Some sources were excluded for licensing:** MTData, Earthlings (GPL-3.0), Bible_crawled "
        "and Bible_pbc rows of Glot500; QuranEnc and Tanzil Qur'an translations; Somali TinyStories.\n"
    )
    lines.append(VERSIONING.format(version=version, repo=repo))
    lines.append("## Takedown\n")
    lines.append(
        f"If you own text in this corpus and want it removed, contact {contact} with the record ids "
        "or the source URL. Removals are applied in the next release.\n"
    )
    lines.append("## Citation\n")
    lines.append(
        "```bibtex\n@misc{somali_nlp_corpus,\n  title  = {Somali NLP Corpus: a cleaned Somali text corpus},\n"
        "  author = {Goobo Labs},\n  year   = {2026},\n"
        f"  url    = {{https://huggingface.co/datasets/{repo}}}\n}}\n```\n"
    )
    return "\n".join(lines)


def build_attribution(manifest: dict, stats: dict) -> str:
    lines = ["# Attribution and notices\n"]
    lines.append(
        "Somali NLP Corpus is compiled from the public datasets below. Text was normalized, filtered "
        "and deduplicated; it was not otherwise altered. Each record's `license` field names the "
        "license of its source. Credit the original dataset when you use its records.\n"
    )
    lines.append("| Source | Upstream | Notes |\n|---|---|---|")
    present = set(manifest["tiers"]["open"]["by_source"]) | set(manifest["tiers"]["non-commercial"]["by_source"])
    for source in sorted(present):
        name, url, note = SOURCES.get(source, (source, "", ""))
        lines.append(f"| {name} (`{source}`) | <{url}> | {note} |")
    lines.append("")

    subsources = [
        item
        for tier in manifest["tiers"].values()
        for item in tier["by_subsource"]
        if item["source"] == "glot"
    ]
    if subsources:
        lines.append("## Glot500 sub-sources\n")
        lines.append(
            "Glot500 aggregates other corpora. The `subsource` field holds the original dataset "
            "name and `license` its license (from the Glot500-c source table, "
            "<https://github.com/cisnlp/Glot500>).\n"
        )
        lines.append("| Sub-source | Original corpus | License | Records |\n|---|---|---|---:|")
        for item in sorted(subsources, key=lambda i: -i["records"]):
            corpus, license_name = GLOT_NOTES.get(item["subsource"].lower(), (item["subsource"], item["license"]))
            lines.append(f"| {item['subsource']} | {corpus} | {license_name} | {human(item['records'])} |")
        lines.append("")

    lines.append("## Common Crawl\n")
    lines.append(
        "Records derived from Common Crawl (CC-100, mC4, MADLAD-400, FineWeb-2, HPLT, and parts of "
        "other sources) are also subject to the "
        "[Common Crawl terms of use](https://commoncrawl.org/terms-of-use).\n"
    )
    lines.append("## Share-alike\n")
    lines.append(
        "CC BY-SA 4.0 records (Wikipedia, CC-100 and FLORES-200 rows) and CC BY-NC-SA records "
        "(XL-Sum and the Leipzig, Wikipedia rows of Glot500) are adaptations of the originals. "
        "If you redistribute them, keep them under the same license.\n"
    )
    return "\n".join(lines) + "\n"


def cargo_version(path: Path) -> str:
    """Workspace version from Cargo.toml, so the card cannot name a different release."""
    match = re.search(r'^version\s*=\s*"([^"]+)"', path.read_text(encoding="utf-8"), re.MULTILINE)
    if not match:
        raise SystemExit(f"no version found in {path}")
    return match.group(1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", type=Path, default=DEFAULT_RELEASE)
    parser.add_argument("--token-stats", type=Path, default=DEFAULT_TOKEN_STATS)
    parser.add_argument("--repo-id", default=DEFAULT_REPO_ID)
    parser.add_argument("--contact", required=True, help="Takedown contact, for example an email address")
    parser.add_argument("--cargo-toml", type=Path, default=DEFAULT_CARGO_TOML)
    args = parser.parse_args()
    version = cargo_version(args.cargo_toml)

    manifest = load_json(args.release / "manifest.json")
    stats = load_json(args.token_stats)
    check_consistent(manifest, stats)
    (args.release / "README.md").write_text(
        build_card(manifest, stats, args.repo_id, args.contact, version), encoding="utf-8"
    )
    (args.release / "ATTRIBUTION.md").write_text(build_attribution(manifest, stats), encoding="utf-8")
    print(f"wrote {args.release / 'README.md'} and {args.release / 'ATTRIBUTION.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
