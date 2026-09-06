# Production / Workover Workbook Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or subagent-driven development for implementation.

**Goal:** Establish a second, privacy-preserving static workbook research corpus for production, completions, workover, intervention and adjacent engineering calculations, then merge its capability evidence with the existing drilling inventory.

**Architecture:** Raw Google Drive binaries remain private. Each workbook is SHA-256 identified and statically inspected without Excel execution. Public repository artifacts contain only path-free workbook IDs, aggregate formula/VBA metrics, capability labels and cross-corpus evidence summaries.

**Tech Stack:** Rust 2024 canonical `wellforge-workbook-audit` reader; BIFF/CFB compatibility scan when the canonical executable is unavailable; CSV/JSON/Markdown research outputs; GitHub Actions for canonical Linux audit-binary build.

**Spec:** Existing `research/drilling-calculation-workbooks/README.md` evidence and privacy policy plus the user-approved adjacent-corpus expansion.

## Global Constraints

- Never execute workbook macros, Excel events, external links or recalculation.
- Never commit raw Drive workbooks, private Drive paths, customer/vendor identifiers, full formula source or VBA source.
- Deduplicate by exact SHA-256 before aggregation.
- Treat workbook results as parity/research evidence, never implementation authority.
- BIFF-encrypted worksheet streams are excluded from formula/sheet totals until an approved static decryption path is available.

### Task 1: Curate and deduplicate the adjacent corpus
- Select calculation-bearing production, completions, workover, well-control, cementing, tubular, hydraulics, motor and thermal workbooks.
- Hash every file and collapse exact duplicates.
- Verify source binaries remain outside the repository.

### Task 2: Run static extraction
- Build or obtain `wellforge-workbook-audit` for Linux.
- Run non-executing static extraction for formulas, names, sheets and workbook structure.
- Statically inventory VBA storage/procedure counts.
- Mark encrypted workbook bodies as limited evidence rather than parsing ciphertext.

### Task 3: Classify calculation capabilities
- Apply domain vocabulary to workbook and static text evidence.
- Separate genuinely new production/completions/workover capabilities from cross-domain parity evidence for existing drilling engines.
- Preserve unresolved/limited evidence instead of inventing model labels.

### Task 4: Merge with drilling evidence
- Produce `MERGED_CAPABILITY_INVENTORY.csv` combining original drilling formula-occurrence evidence with adjacent-workbook counts.
- Mark capabilities as `drilling-only`, `cross-corpus`, or `adjacent-new`.

### Task 5: Verify and publish metadata
- Confirm raw filenames and Drive paths do not appear in public outputs.
- Verify CSV/JSON parseability and aggregate totals.
- Commit metadata to `research/production-completions-workover-workbooks/` on a research branch.
- Run source-verification CI and open a PR only after static evidence gates are satisfied.
