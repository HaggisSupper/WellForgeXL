---
name: drilling-performance-rag
description: Use when the question concerns ROP, mechanical specific energy, d-exponent, drilling efficiency, rig-state/activity classification, real-time drilling channels, or performance-model evidence.
---

# Drilling Performance RAG

## Retrieval focus

Search for ROP, WOB, torque, RPM, bit diameter, mechanical specific energy (MSE), d-exponent/corrected d-exponent, rig state, activity predicates, baselines, transition evidence and real-time channels.

## Engineering checks

- Pair torque and angular velocity from the same mechanical location when reasoning about power.
- Preserve ROP units and distinguish penetration rate from block movement.
- Distinguish surface and downhole motor power contributions.
- Treat d-exponent as a drilling-response trend, not an automatic pore-pressure truth source.
- For rig-state evidence, inspect sampling cadence, stale/gap handling, hysteresis/dwell, baseline definitions and label provenance.
- Severe class imbalance makes raw point accuracy misleading; prefer per-state recall, balanced accuracy, macro-F1 and transition/segment metrics.

## Evidence boundary

Legacy activity labels and workbook MSE outputs are useful parity/challenger evidence. Hand-labelled intervals, synthetic transition traces, first-principles energy relationships and independently sourced measurements provide stronger validation.

Route production rig-state and drilling-performance calculations to their deterministic Rust services when implemented. A statistical/tabular challenger remains advisory unless a separate governed contract explicitly changes that authority boundary.
