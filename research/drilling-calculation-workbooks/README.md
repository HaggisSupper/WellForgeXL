# Drilling calculation workbook research catalog

This directory is a local, deduplicated research catalog sourced from the explicitly authorized `Drilling Background` folder. Run the bounded scanner from the repository root:

```powershell
& '<bundled-python-path>' tools/index_drilling_workbooks.py --preview
& '<bundled-python-path>' tools/index_drilling_workbooks.py
```

The scanner inventories `.xls`, `.xlsx`, `.xlsm`, and `.xlsb` files. It statically inspects OOXML sheet names and formula records, scores drilling-calculation signals, hashes selected candidates, copies one file per exact SHA-256, and preserves every selected source-relative path in `SOURCE_OCCURRENCES.csv`.

The path-free `INDEX.csv` and `CATALOG.md` are retained as repository research metadata. Source workbooks, detailed source paths, the full inventory, and generated manifests are ignored by Git because they may contain licensed methods, customer identifiers, embedded macros, or other proprietary material. Do not execute macros or treat any archived calculation as validated. Use `INDEX.csv` for discovery, local `ALL_EXCEL_FILES.csv` for additional triage, and local `MANIFEST.sha256` for integrity checks.
