# Workbook and package readiness

The consolidated suite retains the engineering, visualization, unit-selection,
and exchange features already present in main. No older workbook source tree or
generated workbook was imported. Numerical Core remains the numerical authority;
this packaging work introduces no physics or field-qualification claim.

| Retained surface | Coverage and disposition |
| --- | --- |
| Shared SI inputs and exchange | Stable IDs, schema/quantity units, workbook-owned exchange state, formula protection, transactional import and rollback remain. |
| Engineering detail | API 7G catalog/load/strength views; hydraulics fluid/flow/nozzle/pressure views; T&D operation and buckling views; BHA assembly, modes, bending, tendency and polar views remain. |
| Visualization | Dashboards, observed-data helpers, sensitivities, persisted chart settings, response-X/reversed-depth-Y roadmaps, radar grid and connected WOB/toolface XY traces remain. |
| Units and runtime | SI, Imperial, Mixed and Custom surfaces remain. Five workbook dispatchers, twelve VBA modules, four Rust executables and their SHA-256 sidecars remain. Existing versioned hydraulics/hash/identity checks are unchanged. |
| Older formula-only and partial macro packages | Retained functionality is already covered by the current sources. Historical generated artifacts are not substitutes for native release acceptance. |

The VBA suite builder resolves relative output paths from the repository root and
rejects blank, absolute, path-bearing and non-`.xlsx` workbook names before source
processing. Versioned input hashes remain mandatory. BHA end-to-end diagnostics
(`bha-e2e-result.json` and `.wfbridge`) go to the log directory, outside the exact
installable payload. Recursive temporary/staging cleanup checks containment and
rejects reparse-point ancestors before deletion.

The trajectory release helper validates its explicit temporary parent and exact
run child before setup, uses atomic exclusive directory creation, and records
ownership only after creation succeeds. Its cleanup revalidates that same child
and all existing ancestors immediately before recursive deletion. Failed setup
never authorizes deleting an existing directory; uncertain material is retained
with a blocked/manual-cleanup report.

The installer now includes both `LICENSE` and `LICENSE-APACHE`, all five workbooks,
and all four executable/sidecar pairs. Installer README and launch conveniences
remain installer-specific. The strict release archive continues to reject extra
files, including build diagnostics; its allowlist has not been expanded.

## BHA authoring and native-template representations

The source-authoring exporter combines radar and XY groups in one chart with
independent axes. Its exporter removes obsolete chart parts **and** their
content-type declarations. An older exporter removed a chart part while leaving
its declaration, producing an invalid package; that implementation was not adopted.

The retained template uses the existing Excel-compatible repair: radar and XY are
separate chart parts, both titled `WOB/toolface polar response`, anchored to the
same display bounds on `Polar Plot`, with radar before XY. Four radar rings and
two connected WOB traces retain their data references, axes and 65% line opacity.
The checked-in XLSX bytes and hash manifest are unchanged.

`WellForgeWorkbookPackage.ps1` validates content-type declarations, relationship
targets and relationship references. Its BHA contract additionally validates the
separate layers, sheet/drawing attachment, display order/bounds and data traces.
The native builder checks both input and staged output packages; the release
runner checks extracted packages. Neither path invokes the repair script or
regenerates templates. The source-authoring combined-chart test remains distinct
from the repaired-template test; passing one does not prove rendering equivalence.

## Native acceptance and cleanup boundaries

`WellForge_BuildInitialize` retains the first initialization error through
best-effort application-state restoration, clears its busy latch, and disables
its handler before propagating that error once. Each setting restoration is
attempted independently; if initialization succeeded, the first restoration
failure is propagated instead. State-read failures are handled before any
application-setting mutation, without restoring uncaptured defaults. The
successful initialization sequence and workbook ownership rules are unchanged.
Focused source/structural tests cover this error-path contract, but do not prove
native VBA semantics. Runtime-injected-error acceptance remains pending safe-close
and reviewed native execution. This change does not establish the cause of, or
claim to resolve, the observed native initialization stall.

The shared `ReadUtf8File` path now removes only a leading UTF-8 BOM code unit
when ADODB.Stream exposes it as U+FEFF. This addresses the native HYD evidence
where the external 64-character executable and sidecar digests matched but the
VBA-side validation reported `ENGINE HASH MISMATCH`. The sidecar remains lower-
cased, trimmed, and subject to the existing exact 64-character hexadecimal
validation and full digest comparison; no substring or alternate hash is
accepted. Focused source tests cover the normalization contract, but do not
prove ADODB.Stream behavior or native Excel semantics. No Rust engine or solver
code changed.

The bounded runner launches its helper hidden. On timeout it may stop only its
captured helper handle; it does not infer ownership of descendants or Excel from
baseline differences, start times or stale PIDs. Failures preserve Excel and
record blocked/manual cleanup. Acceptance records include Excel PID, creation
identity and COM window identity for diagnostics, with no forced Excel termination.

The builder and acceptance runner close only the workbook objects they opened.
Before quitting their COM application they check that no workbook remains open.
An unexpected workbook, unreadable collection, failed quit, unknown identity or
unverified acceptance-process shutdown blocks cleanup and requires manual review.
No trust setting is changed.

The native release gate now invokes the observed `cargo-deny 0.20.2` version and
`cargo-deny --frozen check licenses bans sources` command, retaining output and
exit evidence. A missing or wrong version, or a failed policy check, blocks the
gate. This uses the existing process PATH without global installation/configuration;
the separate CI advisory policy and the generic builder's locked/offline checks
are unchanged.

Historical macro builds produced only three of five workbook outputs. The current
builder still has a compatibility bypass for BHA/directional `BuildInitialize`
and unit tests, assigns `Summary!K5` directly, and runs an external BHA smoke
sequence. Its warning now explicitly states that this is **not native workbook
acceptance**, and does not assert that the current host blocks Office child
processes. A version cell or external smoke result cannot qualify those workbooks.

The actual release runner must execute all five dispatchers, VBA compilation,
unit switching, visualization and chart export, exchange/engine rollback, and
independent package extraction/reopening. Failed dispatch or host ASR controls
must remain failures. The clean-tree gate remains strict. Native verification is
controller-owned after review, from a clean committed checkout; it was not run
for this implementation. This remains a workbook/runtime package, with no newly
qualified `.xlam`, `.xla` or `.xll` add-in and no claim of Excel add-in acceptance.
