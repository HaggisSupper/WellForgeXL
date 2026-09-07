---
name: hydraulics-well-control-rag
description: Use when the question concerns drilling-fluid rheology, hydraulic pressure loss, ECD, bit or nozzle hydraulics, surge/swab, hole cleaning, FIT/LOT, kick/kill calculations, MAASP, or well-control evidence.
---

# Hydraulics and Well Control RAG

## Retrieval focus

Retrieve rheology model, fluid properties, geometry, flow regime, Reynolds/friction-factor evidence, pipe/annulus pressure loss, ECD, nozzle/TFA, surge/swab, transport, FIT/LOT, influx, kill-weight fluid, circulating pressures, MAASP and kick-tolerance concepts.

## Engineering checks

- Preserve absolute versus gauge pressure.
- Preserve TVD versus MD; hydrostatic and ECD relationships normally require TVD while friction/heat-transfer path length follows the physical flow path.
- Preserve density reference state and units.
- Identify rheology model and its parameter definitions before comparing results.
- Treat surge/swab and transport correlations as named model choices, not interchangeable formulas.
- For FIT/LOT and MAASP, identify shoe depth, current fluid density, fracture/formation basis and surface-pressure convention.
- Never infer a missing kill rate, slow-circulating-rate pressure, influx composition or formation pressure.

## Calculation boundary

Use RAG for definitions, model lineage, applicability and acceptance evidence. Use deterministic hydraulics/well-control engines for production calculations when implemented. If a requested well-control capability is not implemented, state that boundary and retrieve the strongest independent evidence needed to specify it.
