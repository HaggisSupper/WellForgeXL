# Task 6 — Offline VBA JSON exchange

## Delivered

- `VBA/WellForgeJsonExchange.bas` is a self-contained VBA7 module with no added references or network dependency.
- Public entry points are `WellForge_LoadJson`, `WellForge_SaveJson`, and `WellForge_ValidateExchange`.
- The module reads and writes UTF-8 JSON, including Unicode surrogate pairs, strict number grammar, duplicate-key rejection, trailing-token rejection, invariant decimal output, arrays, objects, Booleans, and nulls.
- Import destinations come only from the protected `Exchange Map`; JSON cannot choose a sheet or address.
- Imports validate first, snapshot every cell, reject formula destinations, store formula-like text literally, merge tables by stable ID, and roll back the complete applied change set on failure.
- `Exchange State` retains original quantity units only while the canonical value remains unchanged. Changed quantities export in the workbook-mapped unit.
- `Exchange Buffer!B5` is the payload source of record. Status and diagnostics are written to `B7:B8`.
- Saves write a UTF-8 temporary sibling, copy the replaced file to a timestamped `.bak`, and rename the temporary file into place.
- Both file-dialog macros restore Excel calculation, event, and screen-updating state through one cleanup path.

## Verification

Static contract tests cover importable VBA7 source limits, the exact embedded unit registry, public/offline interfaces, the JSON codec, UTF-8 and atomic file handling, workbook-owned mappings, validation, transaction rollback, formula protection, stable-ID behavior, state-based round trips, Buffer B5, dialogs, and Excel-state cleanup.

The cross-platform exchange-engine round-trip suite is also run with the VBA contract suite. It exercises unit preservation and change detection, rollback, literal formula-control text, stable-ID merging, unknown-record retention, duplicate IDs, and capacity handling.

## Desktop smoke test

This Linux build environment cannot execute Excel/VBA. On Windows with desktop Excel, use `tools/Install-WellForgeJsonMacro.ps1` to create the macro-enabled copies, then run `tools/Test-WellForgeJsonMacro.ps1`. That is the final runtime/compile smoke test for the imported module.
