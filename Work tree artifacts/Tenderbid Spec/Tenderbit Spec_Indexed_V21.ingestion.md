# Tenderbit Spec workbook ingestion

Source: `Tenderbit Spec_Indexed_V21.xlsm`

- Drive file: `1yrkwEdBgyC8obNa-Xk0jKW913qfExFSg`
- Retrieved from the Tenderbid Spec Drive folder on 2026-09-01
- Size: 8,181,407 bytes
- SHA-256: `a20de360a6901fad79c401839c408f64467118988ca1bb227488c402f284af09`
- Package validation: Open XML ZIP test passed
- Package entries: 400
- Worksheets: 45
- Embedded VBA project: present (`xl/vbaProject.bin`); it was not executed
- Charts: 15 chart parts, all scatter charts; 21 drawing parts

## Scatter-plot extraction candidates

The workbook contains the source structures needed by the desktop HTML UI:

- `BHA Shape` → `chart5.xml`: four series using the BHA coordinate/geometry ranges
- `Size-Translate-Rotate` → `chart7.xml`–`chart15.xml`: translated/rotated component outlines
- `IntervalDataWithBitPos` → `chart2.xml`: interval and bit-position series
- `IntervalData (2)` → `chart1.xml` and `chart4.xml`: interval profile series
- `Well Timeline` → `chart3.xml`: timeline points
- `Drilling KPI_RT Chart` → `chart6.xml`: KPI scatter series

These chart references are the preferred AST extraction inputs because they preserve
the workbook's plotted coordinate ranges without requiring VBA execution.

## Purge review

The source contains 14 textual cells with the requested blocked vendor/reference
terms. They are located in `MC2 Reqs`, `MC2 Planning Matrix`, and `Additonal
ReqsBUG mc2`. No semantic `K1` or `K2` string value was found; apparent `dk1`/`dk2`
matches are OOXML theme tokens and are not data references.

The `.xlsm` is retained as an unchanged, read-only provenance artifact. Any JSON
or Rust-facing extraction must sanitize those 14 cells before they enter the
application model; the original VBA project must not be copied into the port.

## Follow-up fixes proposed

1. Build a sanitized JSON interchange from the listed chart source ranges and
   workbook tables, preserving sheet/range provenance.
2. Omit the VBA project and replace macro-only calculations with deterministic
   Rust functions.
3. Remove the 14 blocked text values during extraction and add a regression scan
   for the blocked terms in generated JSON and source code.
