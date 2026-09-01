# WellForge HTML UI

This is the local, multi-file browser UI for inspecting the shared WellForge engine exchange case. It follows the visual language of the quote-processing graph viewer found at `I:\^^Move\Projects\Quote DB pocessing\graphify-out\graph.html`: dark graph canvas, left project rail, compact legend/info blocks, and muted community colors. It has tabbed views for overview, trajectory, BHA geometry, hydraulics, torque & drag, and raw exchange data.

All data grids are powered by pinned Tabulator 6.3.1 (loaded from its public distribution URL) with movable columns, pagination, resizing, and a readable fallback for file-only previews.

## Run locally

From the repository root:

```powershell
python -m http.server 8080
```

Open `http://localhost:8080/HTML%20UI/` in a browser. The server is needed because the UI fetches `data/wellforge-mock-case.json`; opening `index.html` directly from `file:` will use the small built-in fallback fixture instead.

## Chart method

`data/chart-method.json` records the extracted reusable method: paired x/y scatter series, measured depth on a reversed y-axis, constant threshold series, and bullet indicators with explicit actual/target/limit text. The implementation is dependency-free SVG so it can be used during local engine testing without a package install.

## Source workbook note

The requested “tenderbid spec” workbook was not present on the available mapped drives during regeneration (`G:` was not mapped). The chart method is therefore recorded as a small, sanitized contract rather than adding an unverified workbook binary.
