---
name: directional-drilling-rag
description: Use when the question concerns surveys, minimum curvature, dogleg severity, toolface, build or turn response, targets, plan-versus-survey, or directional-drilling terminology and evidence.
---

# Directional Drilling RAG

## Retrieval focus

Search for concepts and evidence around trajectory stations, MD/TVD, inclination, azimuth reference, minimum curvature, dogleg angle/DLS, ratio factor, toolface, build/turn response, targets, formations and plan-versus-survey residuals.

## Checks before using evidence

- Confirm angular units and whether DLS is expressed per 30 m, 100 ft, or per metre.
- Confirm azimuth reference: true, grid or magnetic.
- Preserve coordinate-frame conventions and north/east sign conventions.
- Distinguish measured survey stations from interpolated/projected stations.
- For toolface, distinguish gravity/high-side and magnetic toolface where applicable.
- Do not infer missing station spacing, declination, convergence or reference datum.

## Calculation boundary

Use RAG to identify method definitions, assumptions, worked references and validation evidence. Route production trajectory calculations to the deterministic trajectory engine. Treat spreadsheet survey calculators as reference/parity evidence unless their underlying model is independently established.
