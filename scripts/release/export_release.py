#!/usr/bin/env python3
"""Export the final corpus as license-tiered Parquet shards for release.

Reads ``data/final/final_so.jsonl`` and writes

    <out>/data/open/train-00000-of-0000N.parquet            commercial use allowed
    <out>/data/non-commercial/train-00000-of-0000N.parquet  non-commercial licenses
    <out>/manifest.json                                     counts per tier, source, license
    <out>/SHA256SUMS                                        checksum of every shard

Every record is assigned to a tier from its own ``license`` field. A license this script
does not know is an error, never a default, so a new or mislabeled source cannot slip
into the open tier. Text is copied unchanged.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

OPEN = "open"
NON_COMMERCIAL = "non-commercial"

# Licenses that permit commercial use (attribution and share-alike obligations still apply).
OPEN_LICENSES = frozenset(
    {
        "CC0-1.0",
        "CC-BY-4.0",
        "CC-BY-SA-4.0",
        "CC-BY-2.0-FR",
        "ODC-BY",
        "MIT",
        "Apache-2.0",
        "public-domain",
    }
)
# Licenses that restrict use to non-commercial purposes.
NON_COMMERCIAL_LICENSES = frozenset({"CC-BY-NC-SA-4.0", "CC-BY-NC-SA-3.0", "NCGL-UK-2.0"})

DEFAULT_INPUT = Path("data/final/final_so.jsonl")
DEFAULT_OUTPUT = Path("release")
# Start a new shard once it holds about this much text.
DEFAULT_SHARD_BYTES = 1_000_000_000
ROW_GROUP_ROWS = 50_000

SCHEMA = pa.schema(
    [
        ("id", pa.string()),
        ("text", pa.string()),
        ("source", pa.string()),
        ("subsource", pa.string()),
        ("license", pa.string()),
        ("detected_language", pa.string()),
        ("lang_score", pa.float32()),
        ("content_hash", pa.string()),
        ("quality_flags", pa.list_(pa.string())),
    ]
)


def tier_of(license_id: str) -> str:
    """Release tier for a license id; raises for any license that is not classified."""
    if license_id in OPEN_LICENSES:
        return OPEN
    if license_id in NON_COMMERCIAL_LICENSES:
        return NON_COMMERCIAL
    raise ValueError(f"unclassified license {license_id!r}: add it to OPEN_LICENSES or NON_COMMERCIAL_LICENSES")


def to_row(record: dict) -> dict:
    provenance = record["provenance"]
    quality = record.get("quality", {})
    return {
        "id": record["id"],
        "text": record["text"],
        "source": provenance["source"],
        "subsource": provenance.get("subsource"),
        "license": record["license"],
        "detected_language": provenance.get("lang"),
        "lang_score": quality.get("lang_score"),
        "content_hash": record["content_hash"],
        "quality_flags": quality.get("flags", []),
    }


class TierWriter:
    """Writes one tier's rows to numbered shards, rotating by accumulated text size."""

    def __init__(self, directory: Path, shard_bytes: int) -> None:
        self.directory = directory
        self.shard_bytes = shard_bytes
        self.paths: list[Path] = []
        self.rows: dict[str, list] = {}
        self._reset()
        self._writer: pq.ParquetWriter | None = None
        self._shard_text_bytes = 0
        self.records = 0

    def _reset(self) -> None:
        self.rows = {name: [] for name in SCHEMA.names}

    def _open(self) -> None:
        self.directory.mkdir(parents=True, exist_ok=True)
        path = self.directory / f"part-{len(self.paths):05d}.parquet.tmp"
        self.paths.append(path)
        self._writer = pq.ParquetWriter(path, SCHEMA, compression="zstd")
        self._shard_text_bytes = 0

    def _flush(self) -> None:
        if not self.rows["id"]:
            return
        assert self._writer is not None
        self._writer.write_table(pa.Table.from_pydict(self.rows, schema=SCHEMA))
        self._reset()

    def add(self, row: dict) -> None:
        if self._writer is None:
            self._open()
        for name in SCHEMA.names:
            self.rows[name].append(row[name])
        self._shard_text_bytes += len(row["text"].encode("utf-8"))
        self.records += 1
        if len(self.rows["id"]) >= ROW_GROUP_ROWS:
            self._flush()
        if self._shard_text_bytes >= self.shard_bytes:
            self._close_shard()

    def _close_shard(self) -> None:
        self._flush()
        if self._writer is not None:
            self._writer.close()
            self._writer = None

    def finish(self, tier: str) -> list[Path]:
        """Close the last shard and rename shards to their final ``-of-N`` names."""
        self._close_shard()
        total = len(self.paths)
        final: list[Path] = []
        for index, tmp in enumerate(self.paths):
            path = self.directory / f"train-{index:05d}-of-{total:05d}.parquet"
            tmp.rename(path)
            final.append(path)
        return final


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def export(input_path: Path, output_dir: Path, shard_bytes: int, limit: int | None = None) -> dict:
    data_dir = output_dir / "data"
    writers = {
        OPEN: TierWriter(data_dir / OPEN, shard_bytes),
        NON_COMMERCIAL: TierWriter(data_dir / NON_COMMERCIAL, shard_bytes),
    }
    by_source: Counter = Counter()
    by_license: Counter = Counter()
    by_subsource: Counter = Counter()
    by_tier: Counter = Counter()
    total = 0

    with input_path.open(encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            if limit is not None and total >= limit:
                break
            record = json.loads(line)
            row = to_row(record)
            tier = tier_of(row["license"])
            writers[tier].add(row)
            by_tier[tier] += 1
            by_source[(tier, row["source"])] += 1
            by_license[(tier, row["license"])] += 1
            if row["subsource"]:
                by_subsource[(tier, row["source"], row["subsource"], row["license"])] += 1
            total += 1

    shards = {tier: writer.finish(tier) for tier, writer in writers.items() if writer.paths}
    checksums = {}
    for tier, paths in shards.items():
        for path in paths:
            checksums[path.relative_to(output_dir).as_posix()] = sha256_of(path)

    (output_dir / "SHA256SUMS").write_text(
        "".join(f"{digest}  {name}\n" for name, digest in sorted(checksums.items())),
        encoding="utf-8",
    )
    manifest = {
        "records": total,
        "tiers": {
            tier: {
                "records": by_tier[tier],
                "shards": [p.name for p in shards.get(tier, [])],
                "by_source": {s: n for (t, s), n in sorted(by_source.items()) if t == tier},
                "by_license": {lic: n for (t, lic), n in sorted(by_license.items()) if t == tier},
                "by_subsource": [
                    {"source": src, "subsource": sub, "license": lic, "records": n}
                    for (t, src, sub, lic), n in sorted(by_subsource.items())
                    if t == tier
                ],
            }
            for tier in (OPEN, NON_COMMERCIAL)
        },
    }
    (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def check(output_dir: Path) -> None:
    """Re-read every shard: checksums must match and row counts must equal the manifest."""
    manifest = json.loads((output_dir / "manifest.json").read_text(encoding="utf-8"))
    for line in (output_dir / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
        digest, name = line.split("  ", 1)
        if sha256_of(output_dir / name) != digest:
            raise SystemExit(f"checksum mismatch: {name}")
    for tier, info in manifest["tiers"].items():
        rows = sum(
            pq.ParquetFile(output_dir / "data" / tier / shard).metadata.num_rows
            for shard in info["shards"]
        )
        if rows != info["records"]:
            raise SystemExit(f"{tier}: shards hold {rows} rows, manifest says {info['records']}")
    print(f"ok: {manifest['records']:,} records, all checksums and row counts match")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--shard-bytes", type=int, default=DEFAULT_SHARD_BYTES)
    parser.add_argument("--limit", type=int, default=None, help="Export only the first N records")
    parser.add_argument("--check", action="store_true", help="Verify an existing export and exit")
    args = parser.parse_args()

    if args.check:
        check(args.output)
        return 0
    if not args.input.is_file():
        print(f"input not found: {args.input}", file=sys.stderr)
        return 1
    manifest = export(args.input, args.output, args.shard_bytes, args.limit)
    for tier, info in manifest["tiers"].items():
        print(f"{tier}: {info['records']:,} records in {len(info['shards'])} shard(s)")
    print(f"total: {manifest['records']:,}")
    check(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
