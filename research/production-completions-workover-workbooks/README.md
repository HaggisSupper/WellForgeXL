# Production, completions, and workover workbook research

This directory is the public metadata layer for a second static workbook corpus adjacent to the drilling-only collection. Raw Drive workbooks, source paths, customer/vendor identifiers, and full VBA/formula source are **not committed**.

## Scope and verified extraction

The first curated tranche contains **19 unique legacy engineering workbooks** after SHA-256 deduplication. The exact Rust `wellforge-workbook-audit` executable was built in GitHub Actions (run `33996605263`) and used locally against the private source bytes.

- 17 workbooks were read by the canonical Rust/Calamine path.
- 2 BIFF-encrypted workbooks were rejected by the canonical reader and remain static-limited evidence.
- 93 readable worksheets were inventoried.
- 2,052 formula records were recovered. This is a **lower bound** for legacy BIFF because shared/array formula records are not fully reconstructed.
- 1,551 structural formula-family rows were identified from the recovered formulas.
- 981 defined names were recovered.
- 10 workbooks contain VBA storage; 42 modules and 142 procedures were statically inventoried without execution.

No workbook, macro, external link, Excel event, or recalculation was executed.

## Public artifacts

- `INDEX.csv` — path-free workbook IDs, SHA-256 identities, aggregate structural counts and capability labels.
- `TOPIC_INVENTORY.csv` — workbook-count evidence by adjacent-domain topic.
- `CAPABILITY_INVENTORY.csv` — implementation-candidate versus cross-domain parity disposition.
- `MERGED_CAPABILITY_INVENTORY.csv` — capability-level comparison with the existing drilling corpus.
- `VBA_SUMMARY.csv` — path-free VBA aggregate counts only.
- `ANALYSIS.md` — engineering interpretation and limitations.

## Evidence policy

Workbook results are research, parity, and adversarial evidence, **not implementation authority**. A capability enters a released Rust engine only after an independent standard, primary-literature source, closed-form case, or first-principles acceptance suite supports the model.
