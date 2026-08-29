# Task 5 — Office Script JSON exchange

Run `WellForgeJsonExchange` from Excel's **Automate** tab with one of these calls:

```ts
main(workbook, "Import", jsonText)
main(workbook, "Export", "", true)
main(workbook, "Validate", jsonText)
```

`Import` and `Validate` use `jsonText` when provided; otherwise they use
`Exchange Buffer!B5`. `Export` merges into supplied JSON (or the buffer) and
writes formatted JSON to `Exchange Buffer!B5`; `includeResults = false`
exports mapped inputs only.

Office Scripts cannot open an arbitrary local-file dialog. Copy a JSON file's
contents into the script parameter or `Exchange Buffer!B5`, then copy returned
JSON or that buffer cell into a local file after export.

The script reads only declared `Exchange Map` destinations, preserves imported
quantity units in `Exchange State`, rejects formula destinations and invalid
stable IDs, and restores captured cell/state values if an import write fails.
