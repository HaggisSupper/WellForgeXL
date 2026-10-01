# Local committed feature delta

The unpublished feature-delta branch is integrated with current main. It adds hydraulics request/result validation, consistent units, chart export, guarded workbook package processing, and the portable rig-state demo. Current main CI continues to target main with locked dependencies; the immutable numerical governance and existing spatial torque/drag behavior remain.

Two older regressions were corrected during verification: the synchronous hidden helper ignored its TimeoutSeconds argument, so the bounded captured-process execution remains authoritative; initialization and unit-switch errors propagate their original metadata instead of being reduced to status messages. Native hidden-engine execution remains separate follow-up work. Generated demo JSON snapshots use LF consistently so Windows checkout does not break exact regeneration tests.

This merge covers committed source only. Conflicting uncommitted copies and private research/import folders remain local and are not published. The full Rust workspace tests, formatting and strict Clippy, Node source/release checks and demo tests/build are the verification gates. Native Excel/VBA/COM release acceptance was not executed by this consolidation.
