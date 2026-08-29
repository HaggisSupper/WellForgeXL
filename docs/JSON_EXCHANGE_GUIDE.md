# WellForge JSON Exchange Guide

## Contract

The executable interchange format is JSON Schema version `1.0.0`. YAML is not imported by the workbooks. The schema, supported units and a complete shared case are supplied in:

- `schema/wellforge-analysis-exchange.schema.json`
- `data/wellforge-unit-registry.json`
- `data/wellforge-mock-case.json`

Physical values always carry their unit, for example:

```json
{ "value": 28, "unit": "klbf" }
```

Workbook calculations stay in SI. On import the automation converts through the registry and records the original value, unit and canonical value in the hidden `Exchange State` sheet. If the canonical value is unchanged at export, the original unit/value pair is restored. Changed values use the selected display-unit preference when supported, otherwise canonical SI.

## Desktop VBA workflow

1. Close the five source `.xlsx` workbooks.
2. In Windows PowerShell, run `tools/Install-WellForgeJsonMacro.ps1`.
3. If prompted, enable **Trust access to the VBA project object model** in Excel Trust Center, then rerun the installer.
4. Open the generated workbook in `outputs/macro-enabled` and enable macros.
5. Run `WellForge_LoadJson`, `WellForge_SaveJson` or `WellForge_ValidateExchange` from Excel's Macro dialog.

The installer creates `.xlsm` copies and does not alter the source `.xlsx` files. Run `tools/Test-WellForgeJsonMacro.ps1` to check module installation and validation on a Windows machine with Excel.

Save uses a sibling temporary file. When replacing an existing JSON file, the prior file is retained as a timestamped `.bak` before the temporary file is renamed into place.

## Office Script workflow

1. Open a workbook in an Excel environment that supports Office Scripts.
2. From **Automate > New Script**, paste `OfficeScripts/WellForgeJsonExchange.ts`.
3. For import or validation, paste the JSON into `Exchange Buffer!B5` or supply `jsonText`.
4. Run `main(workbook, "Validate")` or `main(workbook, "Import")`.
5. Run `main(workbook, "Export", "", true)` to include results. The returned JSON is also written to `Exchange Buffer!B5`.

Office Scripts cannot display a local file-open/save dialog; copy the buffer or returned text to a `.json` file when required.

## Safety and merge behavior

- Imports are planned and validated before any cell is changed.
- Only workbook-owned mappings can write cells; JSON never supplies worksheet names or addresses.
- Formula destinations and result cells are never overwritten.
- If any write fails, captured prior values are restored.
- Tables merge by stable `id`, retain unknown fields/records and reject duplicate IDs.
- Strings beginning with Excel formula-control characters are stored as literal text.
- Unknown payload branches survive export, allowing one shared file to move between the five analysis workbooks.

## Compatibility

Readers accept compatible `1.x.x` payloads and reject unsupported major versions. Keep the schema and unit registry beside any external system that generates WellForge payloads. The workbooks are planning/review screens and require qualified engineering validation before operational use.

## Recovery

If an import is rejected, read `Exchange Buffer` diagnostics and correct the JSON without manually editing `Exchange State`. If a desktop save is interrupted, use the timestamped `.bak` file. The original `.xlsx` suite remains available even if macro installation fails.
