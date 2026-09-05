# Drilling calculation workbook research catalog

This directory is a local, deduplicated research catalog sourced from the explicitly authorized `Drilling Background` folder. Run the bounded scanner from the repository root:

```powershell
& '<bundled-python-path>' tools/index_drilling_workbooks.py --preview
& '<bundled-python-path>' tools/index_drilling_workbooks.py
```

The scanner inventories `.xls`, `.xlsx`, `.xlsm`, and `.xlsb` files. It statically inspects OOXML sheet names and formula records, scores drilling-calculation signals, hashes selected candidates, copies one file per exact SHA-256, and preserves every selected source-relative path in `SOURCE_OCCURRENCES.csv`.

The path-free `INDEX.csv` and `CATALOG.md` are retained as repository research metadata. `PRIVATE_INDEX.csv`, source workbooks, detailed source paths, the full inventory, and generated manifests are ignored by Git because they may contain licensed methods, customer identifiers, embedded macros, or other proprietary material. Do not execute macros or treat any archived calculation as validated. Use `INDEX.csv` for discovery, local `PRIVATE_INDEX.csv` to run the analyzer, local `ALL_EXCEL_FILES.csv` for additional triage, and local `MANIFEST.sha256` for integrity checks.

## Deep static analysis

Build the non-executing Rust reader, capture one private JSON document at a
time, and stream-merge the public inventories:

```powershell
cargo build --manifest-path engine/Cargo.toml --release -p wellforge-workbook-audit
py -3 tools/analyze_drilling_workbooks.py --force
```

Use repeated `--workbook-id <id>` arguments with `--force` to replace selected
captures without rereading the collection. `--merge-only` regenerates all
tracked inventories from the 100 private JSON files. The analyzer uses one
disposable `openpyxl` subprocess per OOXML workbook, the Rust Calamine reader,
`olevba` source/p-code analysis, and an optional static `msoffcrypto-tool`
compatibility fallback. It never starts Excel or executes a macro. Calamine
formula totals for binary workbooks are lower bounds where its parser cannot
reconstruct shared or array-formula records.

See `DEEP_ANALYSIS.md` for the calculation, unit, VBA, and migration findings.
`ANALYSIS_SUMMARY.json`, the neutral `INDEX.csv`, and the seven tracked CSV
inventories are the merged, path-free research index. Raw formula and VBA
details remain in the ignored
`outputs/drilling-workbook-analysis/workbooks` directory.
