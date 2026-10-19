"""Tests for export_release: tier assignment, row mapping, and shard round trip."""

from __future__ import annotations

import json
from pathlib import Path

import pyarrow.parquet as pq
import pytest

import export_release as er


def record(
    doc_id: str,
    source: str,
    license_id: str,
    text: str = "Soomaaliya waa dal wanaagsan",
    subsource: str | None = None,
    lang: str = "so",
    flags: list[str] | None = None,
) -> dict:
    provenance = {"source": source, "lang": lang}
    if subsource is not None:
        provenance["subsource"] = subsource
    return {
        "id": doc_id,
        "text": text,
        "provenance": provenance,
        "license": license_id,
        "content_hash": f"hash-{doc_id}",
        "quality": {"lang_score": 0.99, "flags": flags or []},
    }


def write_jsonl(path: Path, records: list[dict]) -> None:
    path.write_text("".join(json.dumps(r) + "\n" for r in records), encoding="utf-8")


@pytest.mark.parametrize(
    "license_id",
    ["CC0-1.0", "CC-BY-4.0", "CC-BY-SA-4.0", "ODC-BY", "MIT", "Apache-2.0", "CC-BY-2.0-FR"],
)
def test_commercial_licenses_go_to_the_open_tier(license_id: str) -> None:
    assert er.tier_of(license_id) == er.OPEN


@pytest.mark.parametrize("license_id", ["CC-BY-NC-SA-4.0", "CC-BY-NC-SA-3.0", "NCGL-UK-2.0"])
def test_non_commercial_licenses_go_to_the_restricted_tier(license_id: str) -> None:
    assert er.tier_of(license_id) == er.NON_COMMERCIAL


@pytest.mark.parametrize("license_id", ["Other", "GPL-3.0", "", "cc-by-4.0"])
def test_unknown_license_is_an_error_not_a_default(license_id: str) -> None:
    with pytest.raises(ValueError, match="unclassified license"):
        er.tier_of(license_id)


def test_row_keeps_text_unchanged_and_maps_fields() -> None:
    row = er.to_row(
        record("glot:1", "glot", "Apache-2.0", text="a  b\nc", subsource="Afriberta", lang="en")
    )
    assert row["text"] == "a  b\nc"
    assert row["subsource"] == "Afriberta"
    assert row["detected_language"] == "en"
    assert row["lang_score"] == pytest.approx(0.99)


def test_missing_subsource_is_null() -> None:
    assert er.to_row(record("hplt:1", "hplt", "CC0-1.0"))["subsource"] is None


def test_export_splits_tiers_and_round_trips(tmp_path: Path) -> None:
    records = [
        record("hplt:1", "hplt", "CC0-1.0"),
        record("xlsum:1", "xlsum", "CC-BY-NC-SA-4.0"),
        record("glot:1", "glot", "Apache-2.0", subsource="Afriberta"),
        record("glot:2", "glot", "CC-BY-NC-SA-3.0", subsource="Leipzig_news"),
        record("wiki:1", "wikipedia", "CC-BY-SA-4.0", flags=["high_symbol_ratio"]),
    ]
    source = tmp_path / "final.jsonl"
    write_jsonl(source, records)
    out = tmp_path / "release"

    manifest = er.export(source, out, shard_bytes=1_000_000)

    assert manifest["records"] == 5
    assert manifest["tiers"][er.OPEN]["records"] == 3
    assert manifest["tiers"][er.NON_COMMERCIAL]["records"] == 2
    assert manifest["tiers"][er.NON_COMMERCIAL]["by_source"] == {"glot": 1, "xlsum": 1}
    assert manifest["tiers"][er.NON_COMMERCIAL]["by_subsource"] == [
        {"source": "glot", "subsource": "Leipzig_news", "license": "CC-BY-NC-SA-3.0", "records": 1}
    ]

    open_rows = pq.read_table(out / "data" / er.OPEN / "train-00000-of-00001.parquet").to_pylist()
    assert [r["id"] for r in open_rows] == ["hplt:1", "glot:1", "wiki:1"]
    assert open_rows[2]["quality_flags"] == ["high_symbol_ratio"]
    assert all(er.tier_of(r["license"]) == er.OPEN for r in open_rows)

    er.check(out)


def test_export_rotates_shards_and_names_them_with_the_total(tmp_path: Path) -> None:
    records = [record(f"hplt:{i}", "hplt", "CC0-1.0", text="x" * 100) for i in range(10)]
    source = tmp_path / "final.jsonl"
    write_jsonl(source, records)

    manifest = er.export(source, tmp_path / "release", shard_bytes=250)

    shards = manifest["tiers"][er.OPEN]["shards"]
    assert len(shards) == 4
    assert shards[0] == "train-00000-of-00004.parquet"
    assert shards[-1] == "train-00003-of-00004.parquet"
    er.check(tmp_path / "release")


def test_export_refuses_an_unclassified_license(tmp_path: Path) -> None:
    source = tmp_path / "final.jsonl"
    write_jsonl(source, [record("glot:1", "glot", "Other")])
    with pytest.raises(ValueError, match="unclassified license"):
        er.export(source, tmp_path / "release", shard_bytes=1_000_000)


def test_check_detects_a_corrupted_shard(tmp_path: Path) -> None:
    source = tmp_path / "final.jsonl"
    write_jsonl(source, [record("hplt:1", "hplt", "CC0-1.0")])
    out = tmp_path / "release"
    er.export(source, out, shard_bytes=1_000_000)
    shard = out / "data" / er.OPEN / "train-00000-of-00001.parquet"
    shard.write_bytes(shard.read_bytes() + b"x")
    with pytest.raises(SystemExit, match="checksum mismatch"):
        er.check(out)
