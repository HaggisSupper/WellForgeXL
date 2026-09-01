# WellForge HTML UI

This is the local, multi-file browser UI for inspecting the shared WellForge engine exchange case. It follows the visual language of the quote-processing graph viewer found at `I:\^^Move\Projects\Quote DB pocessing\graphify-out\graph.html`: dark graph canvas, left project rail, compact legend/info blocks, and muted community colors. It has tabbed views for overview, trajectory, BHA geometry, hydraulics, torque & drag, and raw exchange data.

All data grids are powered by the vendored Tabulator 6.3.1 distribution in `vendor/` with movable columns, pagination, resizing, and a readable fallback for file-only previews.

`tokens.css` centralizes the semantic colors, spacing, radii, focus treatment, and motion settings adapted from the shared `APP_STYLE_GUIDE.md`. The Apps launcher supports `Ctrl+K`/`Cmd+K`, `Escape`, click-outside dismissal, keyboard focus trapping, and focus restoration.

## Run locally

From the repository root:

```powershell
python -m http.server 8080
```

Open `http://localhost:8080/HTML%20UI/` in a browser. The server is needed because the UI fetches `data/wellforge-mock-case.json`; opening `index.html` directly from `file:` will use the small built-in fallback fixture instead.

On Windows, double-click [`Launch-WellForgeUI.bat`](Launch-WellForgeUI.bat). It starts the Python HTTP server on port `8765` and opens the UI. Set `WELLFORGE_UI_PORT` before launching to use another port.

## Chart method

`data/chart-method.json` records the extracted reusable method: paired x/y scatter series, measured depth on a reversed y-axis, constant threshold series, and bullet indicators with explicit actual/target/limit text. The implementation is dependency-free SVG so it can be used during local engine testing without a package install.

## Source workbook note

The verified Drive workbook is retained as a read-only provenance artifact at
[`Work tree artifacts/Tenderbid Spec/Tenderbit Spec_Indexed_V21.xlsm`](../Work%20tree%20artifacts/Tenderbid%20Spec/Tenderbit%20Spec_Indexed_V21.xlsm).
Its sheet/chart inventory and purge review are documented in the adjacent
[`Tenderbit Spec_Indexed_V21.ingestion.md`](../Work%20tree%20artifacts/Tenderbid%20Spec/Tenderbit%20Spec_Indexed_V21.ingestion.md).
