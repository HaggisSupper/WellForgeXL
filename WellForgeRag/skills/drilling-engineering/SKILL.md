---
name: drilling-engineering-rag
description: Use when an agent must answer drilling-engineering questions from the WellForgeRag corpus or decide which focused drilling skill to load.
---

# Drilling Engineering RAG

## Purpose

Use WellForgeRag to retrieve evidence and concepts. RAG does **not** replace the deterministic Rust calculation engines and does not establish engineering model authority by retrieval frequency or vector similarity.

## Routing

Load the focused skill that matches the task:

- Source quality, conflicts, citations, standards, model lineage → `evidence-grounding/SKILL.md`
- Surveys, trajectory, DLS, toolface, targets → `directional/SKILL.md`
- Rheology, pressure loss, ECD, surge/swab, hole cleaning, well control → `hydraulics-well-control/SKILL.md`
- Torque/drag, buckling, drillstring, BHA, vibration → `torque-drag-bha/SKILL.md`
- ROP, MSE, d-exponent, rig state, drilling efficiency → `drilling-performance/SKILL.md`

## Retrieval sequence

1. Call `rag_search` with the engineering question and domain filters when known.
2. Hydrate the strongest hits with `rag_get_concept` or `rag_get_artifact`.
3. Follow typed relationships with `rag_related` when dependencies or model lineage matter.
4. Cite the returned artifact locator in the answer.
5. If a numerical answer is required, identify the required deterministic WellForge engine and its inputs; do not calculate by copying an archived spreadsheet formula unless the user explicitly asks for spreadsheet parity analysis.

## Non-negotiable distinctions

- **Reference/parity evidence:** shows that two implementations agree.
- **Independent model evidence:** supports the underlying physics/model through standards, primary literature, first principles, independently derived solutions, or independently measured data.
- **Candidate concept:** useful retrieval knowledge that has not been accepted as validated engineering authority.

Never silently convert the first category into the second.
