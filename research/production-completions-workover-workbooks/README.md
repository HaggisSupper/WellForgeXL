# Production, completions, and workover workbook research

This directory is the public metadata layer for a second static workbook corpus adjacent to the drilling-only collection. Raw Drive workbooks, source paths, customer/vendor identifiers, and complete VBA/formula source are not committed.

## Scope

The initial tranche contains **19 unique legacy engineering workbooks** selected because static evidence indicates calculation content for production, completions, workover, well control, cementing, tubular mechanics, hydraulics, motors, or thermal support. SHA-256 deduplication found no duplicates in this tranche.

## Static extraction outcome

- 93 readable worksheets
- 2,650 readable BIFF formula records
- 1,302 structural formula families
- 981 defined names
- 10 workbooks with VBA storage
- 42 statically recovered VBA modules
- 142 statically identified procedures
- 2 BIFF-encrypted workbooks whose worksheet formulas/names are excluded from the numeric totals
- zero parser failures in the selected tranche

No workbook, macro, formula, external link, or Excel event was executed.

## Evidence policy

These files are research and parity evidence, not implementation authority. A calculation is accepted into a Rust engine only after an independent standard, primary literature source, closed-form case, or first-principles acceptance suite supports the model.

`INDEX.csv` contains path-free workbook identities and aggregate static metrics. `CAPABILITY_INVENTORY.csv` summarizes capability evidence. `VBA_SUMMARY.csv` contains only aggregate macro counts. `MERGED_CAPABILITY_INVENTORY.csv` combines this adjacent corpus with the earlier drilling inventory at capability level.
