# WITSML 2.x alignment boundary

The BHA Rust request is grounded in Energistics WITSML 2.x object identity and measurement semantics without claiming full WITSML server or ETP conformance.

## Authoritative source objects

| Analysis concept | WITSML object identity |
|---|---|
| Well context | `Well` |
| Wellbore parent | `Wellbore` |
| MD, inclination and azimuth | `Trajectory` |
| Hole section diameter | `WellboreGeometry` |
| BHA component geometry | `Tubular` |
| Run/operating context | `BhaRun` |
| Future DAT/EDR channels | `Log` → `ChannelSet` → `Channel` |

Relationships use UUID and optional `eml://` URI. Human-readable names are citations only. Every source reference carries normalized-content SHA-256 and source-system provenance.

Quantities preserve the original symbol/value and are converted to canonical SI through `uom`. The registry rejects a symbol from the wrong physical dimension.

## Implemented subset

- Strict JSON source references for the object types above.
- Offline XML root/UUID/name projection for supported WITSML objects using `quick-xml`.
- Ordered trajectory stations in metres/radians.
- Versioned, unknown-field-denying request/result JSON Schemas.

## Not claimed

- Full Energistics XSD validation.
- WITSML store query/update behavior.
- ETP discovery, streaming or authorization.
- Vendor extension interpretation.
- DAT channel calibration or nonlinear time-domain vibration in Release 1.

See the official [Energistics WITSML schema overview](https://energistics.org/sites/default/files/witsml_schema_overview.html) and [Energistics documentation](https://docs.energistics.org/).
