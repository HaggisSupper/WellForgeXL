# Task 6 Report: End-to-end clean BHA XML converter

## RED

Added `converter_emits_sanitized_structural_and_canonical_json` before the
converter existed. The focused test failed to compile because
`convert_xml`/`InterchangeOutput` were not exported.

## GREEN

Implemented `convert_xml` in the isolated interchange crate. It composes the
existing parser, caller-supplied fingerprint sanitizer, structural tree, and
canonical projection in that order. No solver dependency or command-line
surface was added. The neutral fixture currently contains two components (the
brief's example expected five), so the integration assertion verifies the
fixture's actual two-component contract without changing the fixture.

Commands and results:

- `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test projection converter_emits_sanitized_structural_and_canonical_json` — PASS after implementation.
- `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange` — PASS (16 projection, 10 structural, 0 doc tests).
- `cargo +1.98.0 clippy --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --all-targets -- -D warnings` — PASS.
- `cargo +1.98.0 test --manifest-path engine/Cargo.toml` — PASS.
- `cargo +1.98.0 fmt --all --check --manifest-path engine/Cargo.toml` — PASS.

## Boundary inspection

The Task 6 diff contains only neutral API/test identifiers and no plaintext
restricted deployment tokens. Existing unrelated Task 5 report changes were
left untouched and are excluded from the commit.

## Commit

SHA: efbdcf5f18e48c9d8f05a9b85feab513cae0d85b

## Concerns

The brief and fixture disagree on expected component count (five versus two).
The implementation preserves the fixture and prior task scope; if five
components are required, that fixture change belongs to an explicitly scoped
fixture task.
