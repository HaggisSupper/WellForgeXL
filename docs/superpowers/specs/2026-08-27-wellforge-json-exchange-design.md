# WellForge JSON Exchange — Dual VBA and Office Script Design

## Purpose

Provide one versioned, unit-safe data contract for the complete engineering dataset used by the WellForge workbook suite. The same payload must be readable and writable from every workbook through either offline VBA or Office Script automation without changing the formula-driven SI calculation layer.

The contract covers engineering data and provenance. It does not attempt to serialize workbook presentation, cell styles, chart geometry, or formula source code.

## Deliverables

1. `schema/wellforge-analysis-exchange.schema.json` — JSON Schema Draft 2020-12 contract.
2. `data/wellforge-mock-case.json` — complete valid payload derived from the shared suite mock case.
3. `data/wellforge-unit-registry.json` — supported unit symbols, dimensions, SI multipliers and offsets.
4. `VBA/WellForgeJsonExchange.bas` — offline import, export, validation and JSON codec.
5. `OfficeScripts/WellForgeJsonExchange.ts` — Office Script import/export using the same mappings.
6. `tools/Install-WellForgeJsonMacro.ps1` — Windows Excel automation that copies each `.xlsx` to `.xlsm`, imports the VBA module and preserves the original `.xlsx` files.
7. An `Exchange Map` worksheet in each workbook.
8. An `Exchange State` worksheet in each workbook, hidden from normal operation.
9. Contract, round-trip, unit-conversion and workbook-regression tests.

## Canonical Payload

The top-level object contains:

- `schemaVersion`, `caseId`, timestamps and producer information;
- well, field, pad, rig and datum metadata;
- unit preferences by engineering dimension;
- trajectory plan, surveys, targets, slide intervals and formation tops;
- hole sections, drillstring tubulars and BHA components;
- fluid and rheology definitions;
- operating point, rig limits and pump/nozzle configuration;
- API 7G strength inputs and results;
- hydraulics inputs, section losses, nozzle candidates and results;
- torque, drag and buckling inputs and profiles;
- BHA vibration, bending and drill-ahead tendency inputs and results;
- directional decision surfaces and quality checks;
- source/provenance notes, warnings and calculation-method identifiers.

Every physical quantity is represented as an object:

```json
{
  "value": 1200,
  "unit": "kg/m3"
}
```

Optional fields permit `quality`, `source`, `timestamp` and `note`. Unitless numbers use unit `"1"`. Angles and angular gradients retain their actual source units rather than assuming degrees or radians.

## Unit Safety

The unit registry defines each supported symbol by dimension, SI multiplier and SI offset. Import rejects a unit whose dimension differs from the mapping's expected dimension. Conversion uses:

`SI value = source value × multiplier + offset`

`display value = (SI value − offset) ÷ multiplier`

Workbook calculations continue to consume canonical SI cells. Imported source values and units are recorded in `Exchange State`. On export:

1. If the canonical cell is unchanged since import, the original value and unit are emitted exactly.
2. If the canonical cell changed, it is emitted in the workbook's current per-dimension unit selection.
3. If no display preference exists, the canonical SI value and unit are emitted.

This preserves imported units while still allowing formula calculations to remain SI.

## Workbook Mapping

Each workbook receives an `Exchange Map` table with these columns:

| Column | Meaning |
| --- | --- |
| JSON Pointer | Stable path in the payload |
| Direction | `Input`, `Output`, or `Both` |
| Sheet | Worksheet name |
| Address | Scalar cell or tabular range |
| Shape | `Scalar`, `Row`, or `Table` |
| Value column | Value column for tables |
| Unit source | Constant unit, unit cell, or Unit Map domain |
| Dimension | Expected physical dimension |
| Data type | Number, integer, string, Boolean, date, or status |
| Required | Import requirement |
| Writable | Whether import may change the destination |

The macro and Office Script read this map rather than hard-coding workbook addresses. Inputs are writable; formula results are export-only. Import never overwrites formulas or calculated result cells.

`Exchange State` records JSON pointer, original value, original unit, canonical value, destination address and last-import timestamp. It is the round-trip evidence needed to preserve units.

## VBA Automation

`WellForgeJsonExchange.bas` is self-contained and uses late binding, so no manual VBA references are required. It provides public macros:

- `WellForge_LoadJson()` — file picker, parse, validate, map inputs, recalculate and report.
- `WellForge_SaveJson()` — file picker, merge mapped values into an existing or new payload, validate and save UTF-8 JSON.
- `WellForge_ValidateExchange()` — validate mappings, units and required fields without modifying inputs.

The module includes a JSON parser/serializer supporting objects, arrays, strings, Unicode escapes, finite numbers, Booleans and null. It performs atomic saves through a temporary sibling file and retains a timestamped backup when replacing an existing payload.

The installer creates macro-enabled copies rather than overwriting the `.xlsx` originals. It reports the exact files created and stops if Excel's programmatic VBA-project access is disabled, with the required Trust Center setting shown to the user.

## Office Script Automation

`WellForgeJsonExchange.ts` exposes one `main` function with:

- `action`: `Import`, `Export`, or `Validate`;
- `jsonText`: required for import, optional for export merge;
- `includeResults`: controls whether formula outputs are included.

It returns a structured result containing success, diagnostics and exported JSON. It also writes the latest payload to an `Exchange Buffer` sheet for desktop copying or Power Automate integration. Office Script cannot open arbitrary local file dialogs; this is an Excel platform limitation, not a schema difference.

The TypeScript implementation and VBA implementation consume the same Exchange Map, Exchange State and unit registry data.

## Merge Semantics

Each workbook owns only its mapped branches. Export into an existing payload updates those branches while retaining unknown fields and branches owned by other workbooks. New exports initialize a complete schema-shaped payload and mark unavailable analysis branches as `notCalculated` rather than inventing results.

Array records carry stable identifiers. Merge is by identifier, not row position. Duplicate identifiers are rejected.

## Validation and Failure Handling

Import is two-phase:

1. Parse and validate the complete proposed change in memory.
2. Write permitted inputs only after all blocking checks pass.

Blocking failures include malformed JSON, unsupported schema major version, missing required identifiers, incompatible units, non-finite numbers, duplicate identifiers, invalid mapping rows and attempts to write formula outputs.

Warnings include unknown extension fields, unavailable optional analysis branches and benign display-unit substitutions. Diagnostics are written to `Checks` and returned by Office Script.

If a write fails, VBA restores the prior cell values from an in-memory transaction log. Office Script writes in bounded batches and restores changed ranges from captured values before returning failure.

## Security

- No code, formulas, links or workbook addresses are accepted from JSON.
- Only destinations already declared in `Exchange Map` are writable.
- Strings beginning with `=`, `+`, `-` or `@` are written as text when mapped as strings to prevent formula injection.
- External links and VBA calls are never serialized or executed.
- File operations remain local unless the user explicitly uses Power Automate or cloud storage.

## Testing

Tests must prove:

1. The mock payload validates against the schema.
2. Every mapped physical quantity has a supported dimension and unit.
3. JSON → workbook → JSON round trips preserve unchanged source values and units.
4. Changed values export in the current custom unit selection.
5. Directional and Torque/Drag share the same trajectory identifiers and station values.
6. Repeated fluid, tubular, BHA, WOB, flow and torque values remain consistent.
7. Results and formula cells cannot be imported.
8. Unknown payload branches survive workbook-specific save operations.
9. Malformed payloads cause no partial workbook mutation.
10. All existing workbook formula, chart and visible-sheet checks remain green.

## Compatibility and Versioning

The initial schema version is `1.0.0`. Minor versions may add optional fields. Major versions may change required fields or meanings and must be rejected until the automation explicitly supports them. Payloads include a producer name and version for traceability.

JSON is the canonical wire format. A YAML rendering may be generated for human review, but macros read and write JSON only so there is one executable contract.

## Acceptance Criteria

- Both automation hosts import the same mock JSON into every applicable workbook.
- Both export schema-valid JSON with unchanged units preserved.
- All five workbooks expose auditable mapping and state sheets.
- `.xlsx` originals remain available and VBA-enabled `.xlsm` copies are generated for offline use.
- Formula calculations remain canonical SI and VBA-free `.xlsx` operation remains supported.
- No partial writes, formula injection, external links or silent unit substitutions occur.
