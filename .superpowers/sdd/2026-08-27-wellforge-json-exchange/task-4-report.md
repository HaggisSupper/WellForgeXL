# Task 4 Report — Unit-safe JSON exchange engine

## Result

Implemented the dependency-free JSON pointer and transactional import/export oracle against base `d1dee132b9271647924b9a24990a0c155640a412`.

## Implementation

- Added RFC 6901 pointer traversal and mutation with escape decoding and unsafe prototype-token rejection.
- Added recursive object merge and stable-ID array merge that retain unknown fields and records.
- Added staged import behavior: parse, schema/map validation, complete change planning, capture, and apply.
- Made writes atomic by restoring the complete captured change set after an apply failure; capture and validation failures occur before any write.
- Rejected formula destinations, missing required values, unit/dimension mismatches, duplicate stable IDs, malformed table matrices, unsupported types, and non-finite quantities.
- Converted workbook inputs through canonical SI while resolving both fixed and workbook-cell unit sources.
- Stored auditable original-unit/canonical state and used relative tolerance `1e-12` to preserve unchanged imported quantities exactly.
- Exported changed quantities in the requested `displayUnits` preference, falling back to a registered canonical SI unit.
- Escaped formula-control string prefixes on import and restored their literal JSON text on export.
- Added a deterministic in-memory workbook adapter with capture/restore, formula, and injected-failure support.

## TDD evidence

1. Initial RED:
   - Command: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_roundtrip.test.mjs`
   - Result: failed with `ERR_MODULE_NOT_FOUND` for the absent `exchange_engine.mjs`.
2. Initial GREEN:
   - Same command.
   - Result: 9 passed, 0 failed.
3. Capture-stage RED:
   - Command: `"$CODEX_PRIMARY_RUNTIME_NODE" --test --test-name-pattern='capture failures' tests/exchange_roundtrip.test.mjs`
   - Result: failed because the capture exception escaped instead of returning diagnostics.
4. Capture-stage GREEN and required regression:
   - Command: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_roundtrip.test.mjs tests/exchange_schema.test.mjs tests/exchange_mock_payload.test.mjs`
   - Result: 26 passed, 0 failed.

## Verification

- All exchange tests:
  - Command: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_*.test.mjs`
  - Result: 41 passed, 0 failed.
- Patch hygiene:
  - Command: `git diff --check`
  - Result: exit 0.

## Changed files

- `src/exchange/json_pointer.mjs`
- `src/exchange/exchange_engine.mjs`
- `tests/helpers/mock_workbook_adapter.mjs`
- `tests/exchange_roundtrip.test.mjs`
- `.superpowers/sdd/2026-08-27-wellforge-json-exchange/task-4-report.md`

## Remaining concern

The JavaScript oracle is complete within Task 4 scope. Host-specific Office Script and VBA ports remain assigned to later tasks.
