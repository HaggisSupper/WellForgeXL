# Consolidation to main

Consolidation baseline: `b15bd31f66036a5caa9a6af96a80f1619fdf97cc`.

The latest numerical implementation from `merge/all-to-main` is integrated with the newer application, workbook, research, RAG and hardening work already in main. Older application files were not used to replace newer main implementations. Branch tips and their full histories remain recoverable through the remote tags in `branch-archive.json` and a verified local Git bundle.

## Integrated capability

- Shared numerical primitives and contract governance/registry crates.
- Sparse LU for BHA static solves; triangular solves for modal transformation.
- Bounded Newton, root finding, robust regression, interpolation, interval/volume calculations, parcel transport and continuity primitives.
- Torque/drag supplied-result refinement blending, retained spatial classification and severity behavior.
- Numerical CI now targets main; RAG has a committed lockfile and locked CI commands.

`path-disposition.json` records branch-only additions; `changed-path-disposition.json` records every historical changed path and whether the latest numerical version or current main was retained. The one-shot lockfile-refresh workflow remains archived because it automatically writes to a retired feature branch. The trajectory fixture evidence hash now matches the consolidated engine lockfile; expected engineering outputs are unchanged.

## Verification

See `verification.json` for results and environment limitations. These checks establish source integration, not an industry-qualified release. Windows Excel automation, native desktop packaging and independent engineering benchmarks remain release gates.

## Reassessment

Sparse factorization is implemented, but dense BHA mass/stiffness storage remains. Torque/drag refinement blends supplied stiff results; it does not implement a stiff equilibrium solver. Regression uses normal equations rather than QR/SVD, and automatic differentiation is not implemented. Numerical primitives do not establish new drilling physics by themselves.

Before a paid release, prioritize independently validated engineering benchmark cases and failure envelopes, complete the actual stiff torque/drag solver, validate workbook-to-engine units/contracts and provenance, and establish reproducible signed desktop/Excel releases. RAG and standalone analytics are separate supporting tools and require their own dependency and product validation.

## Recovery

Fetch tags and create a branch at the corresponding `archive/pre-consolidation/...` tag to restore any former branch. The cloud only contains its current working checkout; desktop worktrees must be inventoried on that desktop before removal. Archive mirrors and the backup bundle are retained intentionally.
