# Reference archive

The Windows-only Rust engine migration draws on an external drilling-engineering
reference corpus maintained at:

```
G:\My Drive\Drilling Background
```

This archive is **not** part of the repository and is **not** redistributable.
It is a user-local Google Drive mount that provides authoritative source
material for engineering math, contract shapes, and industry conventions used
by the WellForgeXL engines. All artifacts derived from it that land in the
repo must be original work (formulas expressed in code, contract fields,
comments in your own words) — no verbatim excerpts of third-party material.

## Ingested index

An inventory of the archive is maintained at:

- `G:\My Drive\Drilling Background\knowledge-base\index.md`
- `G:\My Drive\Drilling Background\knowledge-base\raw\manifest.csv`
- `G:\My Drive\Drilling Background\knowledge-base\wiki\pages\`

The manifest is authoritative (SHA-256 per source file, 37K+ records across
61 top-level collections). The wiki pages are curated navigation.

## Engine → archive mapping

Use these sub-collections as the primary reference when implementing the
corresponding Rust engines.

| Rust engine (planned or in flight) | Primary references in the archive |
|------------------------------------|-----------------------------------|
| Torque & Drag (folds API 7G)       | `Torque and Drag\`, `Pipe Handbooks\`, `Drilling Engineering\tnd.pdf`, `Drilling Practices\` |
| Hydraulics                         | `Hydraulics Models\`, `Drilling Practices\`, `Oilfield Measurements & Calculations Course\` |
| BHA static & vibration analysis    | `BHA Analysis\`, `Vibration Primer\`, `Vibration Documents\`, `Directional STUFF\BHA_MU Torque_Bit Grading\` |
| Directional / survey               | `Directional STUFF\DD Calculations\`, `Directional STUFF\Strap Sheets\`, `Survey with index.xlsb` |
| Well control (context only)        | `Saudi Aramco - Well Control\`, `GRACE__R._D.__1994_._Advanced_Blowout_and_Well_Control\` |
| Logging (context only)             | `Logging\`, `Log interpretation\`, `University of Houston - Geophysics+Geology\` |

## Usage rules for engine authors

1. Cite the archive folder (not the file) in commit messages and design notes
   when a formula, table, or convention is drawn from it, e.g.
   *"Slack-off tension per Torque and Drag reference set."*
2. Do not copy figures, tables, or copyrighted text into the repo. Re-express
   the underlying math or convention in your own code and comments.
3. When a reference is decisive for a design choice, add a short entry to the
   relevant `docs/superpowers/plans/*.md` under a *"Reference basis"* bullet.
4. If the archive path is unavailable (fresh checkout, CI, another workstation),
   the engines must still build and test from the repo alone. Reference
   material informs code; it is never a runtime input.
