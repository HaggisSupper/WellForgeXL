#!/usr/bin/env python3
"""Inventory and locally archive drilling-calculation workbooks.

The scanner is metadata-first. It never opens Excel or executes workbook macros.
Selected files are content-hashed, deduplicated, and copied beneath a repository-
contained destination. Detailed source paths and binary files are local research
artifacts and are ignored by Git by default.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import re
import shutil
import sys
import unicodedata
import zipfile
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from xml.etree import ElementTree


EXCEL_EXTENSIONS = {".xls", ".xlsx", ".xlsm", ".xlsb"}
OPENXML_EXTENSIONS = {".xlsx", ".xlsm"}

CATEGORY_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "thermal",
        re.compile(r"(?i)(thermal|temperature|heat[ _-]?exchang)"),
    ),
    (
        "well-control",
        re.compile(
            r"(?i)(well[ _-]?control|pre[ _-]?kick|kick[ _-]?sheet|kill[ _-]?sheet|"
            r"gas[ _-]?(calc|sheet)|gascalcs|influx|blowout)"
        ),
    ),
    (
        "hydraulics",
        re.compile(
            r"(?i)(hydraulic|pressure[ _-]?loss|rheolog|nozzle|\becd\b|hole[ _-]?clean|"
            r"cuttings[ _-]?transport|flow[ _-]?(loss|rate|calculation)|pvt|2[ _-]?phase|"
            r"two[ _-]?phase|lag[ _-]?model|fann|power[ _-]?law|herschel|bingham|"
            r"yield[ _-]?power|\bypl|fric(?:tion)?[ _-]?factor)"
        ),
    ),
    (
        "torque-drag-drillstring",
        re.compile(
            r"(?i)(torque|drag|drill[ _-]?string|over[ _-]?pull|bending[ _-]?strength|"
            r"connection[ _-]?calc|stress[ _-]?(calc|screen)|hook[ _-]?load|buckl)"
        ),
    ),
    (
        "directional",
        re.compile(
            r"(?i)(directional|trajectory|survey[ _-]?(calc|spreadsheet)|dogleg|"
            r"ouija|above[ _-]?below)"
        ),
    ),
    (
        "cementing-casing",
        re.compile(r"(?i)(casing|cement(?:ing)?)"),
    ),
    (
        "bha-tools",
        re.compile(
            r"(?i)(\bbha\b|bottom[ _-]?hole|\bpdm\b|mud[ _-]?motor|rotor[ _-]?catch|"
            r"bit[ _-]?(hydraulic|gauge)|flexi[ _-]?shaft|reaming|pdc[ _-]?thrust)"
        ),
    ),
    (
        "general-drilling",
        re.compile(r"(?i)(drilling|wellbore|completion)"),
    ),
)

CALCULATION_PATTERN = re.compile(
    r"(?i)(calc(?:ulation|ulator|s)?|model|simulat|spreadsheet|worksheet|template|"
    r"programme|program|equation|design|toolbox)"
)
VALIDATION_PATTERN = re.compile(r"(?i)(validation|verify|match|benchmark|testcase)")
NEGATIVE_PATTERN = re.compile(
    r"(?i)(inventory|pricing|progress[ _-]?report|final[ _-]?report|data[ _-]?compaction|"
    r"cement[ _-]?data|drawing|feature[s]?[ _-]?comparison|raw[ _-]?data|processed[ _-]?data)"
)
FORMULA_PATTERN = re.compile(rb"<(?:[A-Za-z_][\w.-]*:)?f(?:\s|>)")


def parse_args() -> argparse.Namespace:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        default=r"G:\My Drive\Drilling Background",
        help="Bounded source directory to scan recursively.",
    )
    parser.add_argument(
        "--destination",
        default=str(repo_root / "research" / "drilling-calculation-workbooks"),
        help="Repository-contained local catalog directory.",
    )
    parser.add_argument(
        "--minimum-score",
        type=int,
        default=5,
        help="Minimum deterministic evidence score selected for the archive.",
    )
    parser.add_argument(
        "--preview",
        action="store_true",
        help="Classify without hashing, copying, or writing catalog files.",
    )
    return parser.parse_args()


def is_within(child: Path, parent: Path) -> bool:
    try:
        child.relative_to(parent)
        return True
    except ValueError:
        return False


def walk_excel_files(source: Path) -> list[Path]:
    files: list[Path] = []
    for root, directories, names in os.walk(source, followlinks=False):
        directories.sort(key=str.casefold)
        names.sort(key=str.casefold)
        for name in names:
            path = Path(root) / name
            if path.suffix.lower() in EXCEL_EXTENSIONS and not name.startswith("~$"):
                files.append(path)
    return files


def category_for(text: str) -> tuple[str, str | None]:
    for category, pattern in CATEGORY_PATTERNS:
        match = pattern.search(text)
        if match:
            return category, match.group(0)
    return "uncategorized", None


def count_formulas(payload: bytes) -> int:
    return len(FORMULA_PATTERN.findall(payload))


def inspect_openxml(path: Path) -> dict[str, Any]:
    details: dict[str, Any] = {
        "sheet_names": [],
        "formula_count": None,
        "has_macros": False,
        "inspection_status": "not-inspected",
    }
    if path.suffix.lower() not in OPENXML_EXTENSIONS:
        details["inspection_status"] = "binary-format-metadata-only"
        return details

    try:
        with zipfile.ZipFile(path) as workbook:
            entries = workbook.namelist()
            lower_entries = {entry.lower(): entry for entry in entries}
            details["has_macros"] = "xl/vbaproject.bin" in lower_entries

            workbook_entry = lower_entries.get("xl/workbook.xml")
            if workbook_entry:
                root = ElementTree.fromstring(workbook.read(workbook_entry))
                details["sheet_names"] = [
                    node.attrib.get("name", "")
                    for node in root.iter()
                    if node.tag.rsplit("}", 1)[-1] == "sheet"
                ]

            formula_count = 0
            for entry in entries:
                normalized = entry.replace("\\", "/").lower()
                if re.fullmatch(r"xl/worksheets/sheet\d+\.xml", normalized):
                    formula_count += count_formulas(workbook.read(entry))
            details["formula_count"] = formula_count
            details["inspection_status"] = "openxml-inspected"
    except (OSError, ValueError, zipfile.BadZipFile, ElementTree.ParseError) as error:
        details["inspection_status"] = f"inspection-error:{type(error).__name__}"
    return details


def score_workbook(
    relative_path: str,
    name: str,
    extension: str,
    details: dict[str, Any],
) -> dict[str, Any]:
    stem = Path(name).stem
    parent_text = str(Path(relative_path).parent)
    sheet_text = " | ".join(details["sheet_names"])
    score = 0
    signals: list[str] = []

    filename_category, filename_domain = category_for(stem)
    path_category, path_domain = category_for(parent_text)
    sheet_category, sheet_domain = category_for(sheet_text)

    if filename_domain:
        score += 4
        signals.append(f"filename-domain:{filename_domain}")
    if CALCULATION_PATTERN.search(stem):
        score += 3
        signals.append("filename-calculation-term")
    if VALIDATION_PATTERN.search(stem):
        score += 2
        signals.append("filename-validation-term")
    if path_domain:
        score += 1
        signals.append(f"path-domain:{path_category}")
    if CALCULATION_PATTERN.search(parent_text):
        score += 1
        signals.append("path-calculation-term")

    formula_count = details["formula_count"]
    if isinstance(formula_count, int) and formula_count > 0:
        score += 2
        signals.append("contains-formulas")
        if formula_count >= 20:
            score += 1
            signals.append("formula-rich")
    if sheet_domain:
        score += 2
        signals.append(f"sheet-domain:{sheet_category}")
    if sheet_text and CALCULATION_PATTERN.search(sheet_text):
        score += 1
        signals.append("sheet-calculation-term")
    if details["has_macros"]:
        score += 1
        signals.append("contains-vba-project")
    if extension in {".xls", ".xlsb"} and (
        filename_domain or CALCULATION_PATTERN.search(stem)
    ):
        score += 1
        signals.append("binary-format-name-evidence")
    if NEGATIVE_PATTERN.search(stem):
        score -= 3
        signals.append("filename-reference-or-data-term")

    if filename_category != "uncategorized":
        category = filename_category
    elif sheet_category != "uncategorized":
        category = sheet_category
    else:
        category = path_category

    if score >= 7:
        classification = "drilling_calculation"
    elif score >= 5:
        classification = "likely_drilling_calculation"
    elif filename_domain or sheet_domain or path_domain:
        classification = "drilling_reference_or_data"
    else:
        classification = "not_selected"

    return {
        "score": score,
        "classification": classification,
        "category": category,
        "signals": ";".join(signals),
    }


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def safe_stem(name: str) -> str:
    normalized = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    normalized = re.sub(r"[^A-Za-z0-9]+", "-", normalized).strip("-").lower()
    return (normalized or "workbook")[:80].rstrip("-")


def write_csv(path: Path, rows: list[dict[str, Any]], fields: list[str]) -> None:
    with path.open("w", newline="", encoding="utf-8") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)


def main() -> int:
    args = parse_args()
    source = Path(args.source).resolve(strict=True)
    destination = Path(args.destination).resolve()
    repo_root = Path(__file__).resolve().parents[1]
    if not source.is_dir():
        raise ValueError(f"source is not a directory: {source}")
    if not is_within(destination, repo_root) or destination == repo_root:
        raise ValueError(f"destination must remain within repository root: {repo_root}")
    if args.minimum_score < 1:
        raise ValueError("minimum score must be positive")

    paths = walk_excel_files(source)
    metadata_cache: dict[tuple[str, str, int], dict[str, Any]] = {}
    rows: list[dict[str, Any]] = []
    for index, path in enumerate(paths, start=1):
        stat = path.stat()
        relative = path.relative_to(source).as_posix()
        extension = path.suffix.lower()
        cache_key = (path.name.casefold(), extension, stat.st_size)
        details = metadata_cache.get(cache_key)
        if details is None:
            details = inspect_openxml(path)
            metadata_cache[cache_key] = details
        scored = score_workbook(relative, path.name, extension, details)
        rows.append(
            {
                "source_relative_path": relative,
                "original_name": path.name,
                "extension": extension,
                "bytes": stat.st_size,
                "modified_utc": datetime.fromtimestamp(
                    stat.st_mtime, timezone.utc
                ).isoformat(),
                "sheet_count": len(details["sheet_names"]),
                "sheet_names": " | ".join(details["sheet_names"]),
                "formula_count": details["formula_count"],
                "has_macros": details["has_macros"],
                "inspection_status": details["inspection_status"],
                **scored,
                "sha256": "",
                "archive_relative_path": "",
            }
        )
        if index % 100 == 0:
            print(f"classified {index}/{len(paths)}", file=sys.stderr)

    selected = [row for row in rows if row["score"] >= args.minimum_score]
    preview = {
        "source": str(source),
        "excel_files": len(rows),
        "selected_occurrences": len(selected),
        "selected_mib_before_deduplication": round(
            sum(row["bytes"] for row in selected) / (1024 * 1024), 1
        ),
        "classification_counts": dict(Counter(row["classification"] for row in rows)),
        "selected_category_counts": dict(Counter(row["category"] for row in selected)),
        "minimum_score": args.minimum_score,
    }
    if args.preview:
        preview["selected_sample"] = [
            {
                "score": row["score"],
                "category": row["category"],
                "path": row["source_relative_path"],
            }
            for row in sorted(
                selected,
                key=lambda row: (-row["score"], row["source_relative_path"].casefold()),
            )[:40]
        ]
        print(json.dumps(preview, indent=2))
        return 0

    destination.mkdir(parents=True, exist_ok=True)
    files_root = destination / "files"
    files_root.mkdir(exist_ok=True)

    selected_by_hash: dict[str, list[dict[str, Any]]] = defaultdict(list)
    path_by_relative = {path.relative_to(source).as_posix(): path for path in paths}
    hash_errors = 0
    for index, row in enumerate(selected, start=1):
        path = path_by_relative[row["source_relative_path"]]
        try:
            digest = sha256_file(path)
            row["sha256"] = digest
            selected_by_hash[digest].append(row)
        except OSError as error:
            hash_errors += 1
            row["inspection_status"] += f";hash-error:{type(error).__name__}"
        if index % 25 == 0:
            print(f"hashed {index}/{len(selected)}", file=sys.stderr)

    unique_rows: list[dict[str, Any]] = []
    manifest_lines: list[str] = []
    for digest, occurrences in sorted(selected_by_hash.items()):
        representative = sorted(
            occurrences,
            key=lambda row: (-row["score"], row["source_relative_path"].casefold()),
        )[0]
        category = representative["category"] or "uncategorized"
        extension = representative["extension"]
        archive_name = f"{safe_stem(Path(representative['original_name']).stem)}--{digest[:12]}{extension}"
        relative_archive = Path("files") / category / archive_name
        target = (destination / relative_archive).resolve()
        if not is_within(target, files_root.resolve()):
            raise ValueError(f"archive path escaped destination: {target}")
        target.parent.mkdir(parents=True, exist_ok=True)
        source_path = path_by_relative[representative["source_relative_path"]]
        if not target.exists() or sha256_file(target) != digest:
            shutil.copy2(source_path, target)

        archive_text = relative_archive.as_posix()
        manifest_lines.append(f"{digest}  {archive_text}")
        for occurrence in occurrences:
            occurrence["archive_relative_path"] = archive_text

        unique_rows.append(
            {
                "id": digest[:16],
                "classification": representative["classification"],
                "score": max(row["score"] for row in occurrences),
                "category": category,
                "archive_relative_path": archive_text,
                "original_name": representative["original_name"],
                "extension": extension,
                "bytes": representative["bytes"],
                "sha256": digest,
                "source_occurrences": len(occurrences),
                "sheet_count": representative["sheet_count"],
                "sheet_names": representative["sheet_names"],
                "formula_count": representative["formula_count"],
                "has_macros": representative["has_macros"],
                "inspection_status": representative["inspection_status"],
                "signals": representative["signals"],
            }
        )

    unique_rows.sort(key=lambda row: (row["category"], row["original_name"].casefold()))
    rows.sort(key=lambda row: row["source_relative_path"].casefold())
    occurrence_rows = [
        {
            "sha256": row["sha256"],
            "archive_relative_path": row["archive_relative_path"],
            "source_relative_path": row["source_relative_path"],
            "modified_utc": row["modified_utc"],
        }
        for row in rows
        if row["sha256"]
    ]

    write_csv(
        destination / "INDEX.csv",
        unique_rows,
        [
            "id",
            "classification",
            "score",
            "category",
            "archive_relative_path",
            "original_name",
            "extension",
            "bytes",
            "sha256",
            "source_occurrences",
            "sheet_count",
            "sheet_names",
            "formula_count",
            "has_macros",
            "inspection_status",
            "signals",
        ],
    )
    write_csv(
        destination / "SOURCE_OCCURRENCES.csv",
        occurrence_rows,
        ["sha256", "archive_relative_path", "source_relative_path", "modified_utc"],
    )
    write_csv(
        destination / "ALL_EXCEL_FILES.csv",
        rows,
        [
            "classification",
            "score",
            "category",
            "source_relative_path",
            "original_name",
            "extension",
            "bytes",
            "modified_utc",
            "sheet_count",
            "sheet_names",
            "formula_count",
            "has_macros",
            "inspection_status",
            "signals",
            "sha256",
            "archive_relative_path",
        ],
    )
    (destination / "MANIFEST.sha256").write_text(
        "\n".join(manifest_lines) + ("\n" if manifest_lines else ""),
        encoding="utf-8",
    )

    summary = {
        **preview,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "destination": str(destination),
        "unique_archived_workbooks": len(unique_rows),
        "archived_mib": round(
            sum(row["bytes"] for row in unique_rows) / (1024 * 1024), 1
        ),
        "exact_duplicate_occurrences_removed": len(selected) - len(unique_rows),
        "hash_errors": hash_errors,
        "archived_format_counts": dict(
            Counter(row["extension"] for row in unique_rows)
        ),
        "archived_category_counts": dict(
            Counter(row["category"] for row in unique_rows)
        ),
        "limitations": [
            "XLS and XLSB files are classified from path and filename metadata only.",
            "Formula counts are static OOXML counts and do not execute formulas.",
            "Macros are never executed; archived workbooks must be treated as untrusted input.",
            "Selection is deterministic triage for research, not engineering validation.",
        ],
    }
    (destination / "scan-summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
