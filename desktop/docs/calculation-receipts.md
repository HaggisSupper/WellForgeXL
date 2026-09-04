# Calculation receipts

Every engineering result that crosses a WellForge application boundary must
have an immutable receipt created by Rust. A receipt identifies:

- the algorithm and version;
- one or more immutable input revisions with SHA-256 content digests;
- unit-system and coordinate-reference context;
- the computational backend;
- the responsible actor and explicit warnings; and
- a SHA-256 digest of the canonical JSON output.

The output digest is calculated from a recursively canonicalized JSON value,
so object-key order does not alter the result. Arrays retain their order.
Frontend clients render receipt data but do not create, replace, or derive
engineering receipts.

The current contract is `wellforge_core::CalculationReceipt`. Solver commands
should create it at their Rust boundary after successful validation and return
it alongside their typed output. The local authority store persists this typed
contract only after verifying the output digest and binding a
`project_revision` input to the exact stored revision ID and content digest.
Typed reads deserialize and revalidate the persisted receipt bytes.

This receipt is provenance metadata, not a signature or an authorization
decision. Release and actor-authentication policy remains a separate boundary.

## Current local minimum-curvature receipt

The desktop `calculate_minimum_curvature` command accepts only two SI survey
stations. It requires an already active project selected by the native desktop
flow; no local path or provenance fields are accepted from, or returned to, the
frontend. Rust reads the exact bounded selected bytes once, saves them as the
current immutable local revision event, and binds the receipt to that event's
unique ID and SHA-256 content digest. It also records the SHA-256 digest of the
canonical JSON request containing the two stations. The typed receipt is
persisted before the command returns its result; a read or persistence failure
returns a structured error.

This vertical slice fixes the context to SI, `EPSG:4979`, and the CPU backend.
It currently uses `local-workstation` as a stable actor identifier. That actor
means only that the calculation originated from this installed desktop client;
it is not user authentication. A later authenticated actor source can replace
this temporary local identifier without changing the receipt shape.
