"""Tests for build_card: consistency guard and generated text."""

from __future__ import annotations

import pytest

import build_card as bc


def manifest(records: int = 3) -> dict:
    return {
        "records": records,
        "tiers": {
            "open": {
                "records": 2,
                "shards": ["train-00000-of-00001.parquet"],
                "by_source": {"hplt": 1, "glot": 1},
                "by_license": {"CC0-1.0": 1, "Apache-2.0": 1},
                "by_subsource": [
                    {"source": "glot", "subsource": "Afriberta", "license": "Apache-2.0", "records": 1}
                ],
            },
            "non-commercial": {
                "records": 1,
                "shards": ["train-00000-of-00001.parquet"],
                "by_source": {"xlsum": 1},
                "by_license": {"CC-BY-NC-SA-4.0": 1},
                "by_subsource": [],
            },
        },
    }


def stats(documents: int = 3) -> dict:
    by_source = {
        "hplt": {"documents": 1, "words": 10, "tokens": 14},
        "glot": {"documents": 1, "words": 20, "tokens": 27},
        "xlsum": {"documents": 1, "words": 30, "tokens": 40},
    }
    return {
        "tokenizers": {
            "v2": {"documents": documents, "words": 60, "total_tokens": 81, "by_source": by_source}
        }
    }


def test_card_numbers_come_from_the_manifest_and_stats() -> None:
    card = bc.build_card(manifest(), stats(), "somast/x", "t@example.org", "1.0.0")
    assert "**3 documents, 60 words, 81 tokens**" in card
    assert "| `open` | 2 |" in card
    assert "| `non-commercial` | 1 |" in card
    assert "t@example.org" in card
    assert "1.350 tokens per word" in card


def test_card_is_gated_and_states_its_version() -> None:
    card = bc.build_card(manifest(), stats(), "somast/x", "t@example.org", "1.2.3")
    header = card.split("---")[1]
    assert "extra_gated_prompt:" in header
    assert "extra_gated_fields:" in header
    assert "takedown requests sent to t@example.org" in header
    assert "revision=\"v1.2.3\"" in card
    assert "version **1.2.3**" in card


def test_cargo_version_reads_the_workspace_version(tmp_path) -> None:
    toml = tmp_path / "Cargo.toml"
    toml.write_text('[workspace.package]\nversion = "2.4.6"\nedition = "2021"\n', encoding="utf-8")
    assert bc.cargo_version(toml) == "2.4.6"


def test_mismatched_stats_are_refused() -> None:
    with pytest.raises(SystemExit, match="re-run tokenize_corpus"):
        bc.build_card(manifest(3), stats(4), "somast/x", "t@example.org", "1.0.0")


def test_attribution_lists_glot_sub_sources_with_their_license() -> None:
    text = bc.build_attribution(manifest(), stats())
    assert "| Afriberta | AfriBERTa | Apache License 2.0 | 1 |" in text
    assert "XL-Sum Somali" in text
    assert "Common Crawl" in text
