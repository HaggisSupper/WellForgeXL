---
name: drilling-evidence-grounding
description: Use when drilling evidence conflicts, a source must be cited, model validity is questioned, or an agent must distinguish a reference implementation from independent engineering support.
---

# Drilling Evidence Grounding

## Evidence hierarchy

Prefer, in order appropriate to the claim:

1. Governing standards and primary technical literature.
2. Independently measured or independently derived engineering evidence.
3. Published worked examples and reputable engineering references.
4. Internal reference fixtures and independently implemented software comparisons.
5. Legacy workbook/VBA parity evidence.
6. Unverified notes or generated candidate concepts.

A lower-ranked source can still be useful, but label what it proves.

## RAG workflow

Use `rag_search`, hydrate with `rag_get_concept`/`rag_get_artifact`, then use `rag_related` to inspect `derived_from`, `validated_by`, `implemented_by`, `supersedes`, and `depends_on` edges.

Every material claim grounded in RAG should carry the artifact locator returned by the corpus. Do not invent page, sheet, range, curve, line, or table locators.

## Contradictions

When sources disagree:

- preserve both claims and their provenance;
- compare units, reference states, pressure basis, temperature basis, sign conventions, geometry assumptions and model applicability;
- prefer the source appropriate to the requested claim rather than averaging conflicting values;
- state unresolved uncertainty when evidence does not settle the issue.

## Calculation boundary

Retrieved equations can explain or audit a model. Production numerical outputs come from the appropriate deterministic WellForge engine when that engine exists. If no engine exists, describe the evidence and missing capability rather than implying RAG executed a validated calculation.
