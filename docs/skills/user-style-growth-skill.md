---
name: user-style-growth
description: Session-local guidance for growing with a user who prefers document-first, phased, high-signal corpus work.
version: 1
scope: session-local
signals:
  - document-first
  - phased execution
  - high-signal triage
  - budget-aware prompting
  - corpus-driven validation
  - concise sync notes
growth_rules:
  - infer from repeated behavior, not one-off requests
  - prefer concrete artifacts over abstract promises
  - keep retrieval and routing strict before expensive work
  - preserve structure in text extraction
  - treat plan and pickup notes as the source of truth
---

# User Style Growth

## What this is
This document captures the working style inferred from this session so other instances can grow into the same mode instead of starting blank.

## Core style
- Document-first over code-first.
- Phased execution over one-shot implementation.
- High-value corpus triage over broad noisy sweeps.
- Budget-aware use of model passes.
- Visible progress through plan and pickup artifacts.
- Concise status updates with concrete state changes.

## How to grow into it
1. Watch for repeated preference signals across turns.
2. Update the routing/classification layer before expensive extraction.
3. Keep the most important artifacts synchronized: plan, pickup note, todos.
4. Preserve tables, examples, formulas, and section structure in extracted text.
5. Prefer strict primary/secondary/noise buckets.

## What to avoid
- Broad speculative implementation.
- Flattening technical documents into plain prose too early.
- Hidden progress with no artifact trail.
- Running expensive passes on noise.

## Handoff rule
If a session ends, the next session should read this file, the pickup note, and the live plan before doing any new work.
