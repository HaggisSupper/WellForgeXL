# Drilling calculation workbook catalog

## Scan summary

The bounded scan of `Drilling Background` found 1,282 Excel-format files. Seven temporary Excel owner files were excluded. The remaining 1,275 workbooks were classified without launching Excel or executing macros.

- 164 high-confidence drilling-calculation occurrences were selected.
- SHA-256 deduplication reduced those occurrences to 100 unique workbooks.
- The local ignored archive occupies 189.2 MiB.
- 399 likely calculation workbooks remain in the local full inventory for later review.
- 477 files were classified as drilling references or data rather than calculations.
- 235 files did not meet the drilling-calculation evidence threshold.

## Archived categories

| Category | Unique workbooks |
| --- | ---: |
| Hydraulics | 41 |
| BHA and downhole tools | 24 |
| Torque, drag, and drillstring | 16 |
| Directional drilling | 6 |
| Cementing and casing | 3 |
| Well control | 3 |
| Thermal | 3 |
| General drilling | 2 |
| Uncategorized calculation evidence | 2 |

## Formats

| Format | Unique workbooks | Static formula inspection |
| --- | ---: | --- |
| `.xlsx` | 49 | Yes |
| `.xls` | 39 | Yes, with the Rust Calamine reader |
| `.xlsm` | 10 | Yes, macros not executed |
| `.xlsb` | 2 | Yes, with the Rust Calamine reader |

## Temperature-model leads

| Workbook | Static evidence | Research relevance |
| --- | --- | --- |
| `b147eb91b2f2fd18` | 28,862 formulas; temperature-sensitive elastomer, fit, wear, and life logic | Tool/material behavior, not a wellbore heat-exchange solver |
| `348dd5eb51c7fac7` | 357 formulas in 3 filled-column families | Formation-temperature profile and gradient fixtures |
| `dc888cd53f297b84` | 152 formulas in 43 families; 10 radial-resistance/overall-coefficient cells | Property conversion and radial heat-transfer fixtures |

See `DEEP_ANALYSIS.md` and the tracked `ANALYSIS_SUMMARY.json` plus CSV
inventories for the complete calculation-family, unit, and VBA analysis.

## Research controls

- Treat every workbook as untrusted input and never enable macros during extraction.
- Use workbook equations as comparison material, not as validated production authority.
- Confirm units, sign conventions, geometry, and applicability against independent sources before porting a method.
- Preserve neutral physics-based names in WellForge code rather than historical product or vendor identifiers.
- Use `INDEX.csv` for path-free discovery. Detailed source provenance and binaries remain local and ignored.
