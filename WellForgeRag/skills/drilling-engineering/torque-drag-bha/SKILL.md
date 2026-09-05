---
name: torque-drag-bha-rag
description: Use when the question concerns torque and drag, drillstring friction/contact, buckling, tubular limits, BHA static response, vibration, modal/FRF/Campbell results, motors, or drillstring mechanical evidence.
---

# Torque Drag and BHA RAG

## Retrieval focus

Search for drillstring geometry, trajectory, buoyed weight, friction coefficients, contact force, axial load, torque, connection/tubular limits, buckling, BHA component geometry, static bending, modal response, FRF, Campbell diagrams and motor performance.

## Engineering checks

- Distinguish pickup, slack-off, rotating-off-bottom and drilling load cases.
- Identify whether a source uses soft-string or stiff-string/contact mechanics.
- Preserve tension/compression and torque sign conventions.
- Separate a buckling screening criterion from a post-buckling/contact solution.
- Check material properties, OD/ID, connection capacity and safety-factor basis before comparing limits.
- For BHA dynamics, distinguish modal frequencies, forced response and actual nonlinear impact/whirl dynamics.
- Treat equipment motor curves as versioned equipment data, not universal physics.

## Calculation boundary

Use RAG to retrieve model assumptions and evidence. Route implemented numerical cases to torque-drag or BHA Rust engines. Do not describe the current soft-string engine as a stiff-string solver and do not infer nonlinear contact/dynamics from a linear modal result.
