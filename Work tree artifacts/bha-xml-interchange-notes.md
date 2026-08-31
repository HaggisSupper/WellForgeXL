# BHA XML interchange worktree notes

Archived from the merged `bha-xml-interchange` worktree on 2026-08-31. These
are the only uncommitted report addenda retained from that worktree.

## Review follow-up

- Fix round 4 tightened supplied RSS/stabilizer dimensional values to positive
  domains and added an integration regression for invalid booleans and duplicate
  detail blocks. Parent: `5b475fc`.
- Fix round 5 added independent regressions for duplicate scalars, absent
  optionals, and invalid geometry. Parent: `3f2a1b0`.
- Fix round 6 added isolated coverage for detail conflicts, exact
  stabilizer/RSS projections, nested motor geometry, and invalid dimensions.
  Commit: `f4c3d4f381bc2b612db060a5661f3c22984ac4dd`; projection tests: 13
  passing; structural tests: 10 passing.

## Implementation follow-up

- Locked crate tests passed with 17 projection tests and 11 structural tests.
- Follow-up commits: `4564362ad65fd7c2909e0ce0331d770ab1118c5c` and
  `9e7ba16f3191e8244113823083cf3a418294392f`.
