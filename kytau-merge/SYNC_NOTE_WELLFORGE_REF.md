# KyTau Merge sync note

This branch is the GitHub sync landing branch for the generated KyTau Merge package.

Generated artifact available in the ChatGPT sandbox:

- `/mnt/data/KytauMerge.zip`
- SHA256: `2c2f414df51af5d8662ba6d5b0149829eef637716a03c4e15dee539c08ff0e34`

WellForge reference material used for this sync:

- `desktop/WELLFORGE-SOURCE-MIGRATION-CATALOG.md`

Reference-derived constraints preserved:

- Clean migration over mechanical legacy merge.
- Contract-first, fixture-driven intake.
- Rust-first domain and service implementation.
- Tauri UI rebuild from workflow specifications rather than direct WPF/XAML conversion.
- Generated, vendored, binary, and build artifacts excluded from migration input.

Connector limitation:

The available GitHub connector can create text files, branches, commits, PRs, trees, and issues inside already-accessible repositories. It cannot create a brand-new repository and does not directly ingest a local ZIP artifact as an expanded tree. This branch is therefore prepared as the sync landing point inside `HaggisSupper/WellForgeXL`; the generated package should be pushed from a local Git client or Work/Codex environment to complete full source-tree synchronization.

Recommended target layout:

```text
kytau-merge/
  .promptset.json
  Cargo.toml
  crates/
  apps/desktop-tauri/
  contracts/
  config/
  docs/
  opencode-global/
  scripts/
  tests/
  tools/
  work-packages/
```

Recommended local completion command from Windows PowerShell:

```powershell
$Repo = "$env:USERPROFILE\source\repos\WellForgeXL"
$Zip = "$env:USERPROFILE\Downloads\KytauMerge.zip"
cd $Repo
git fetch origin
git checkout feature/kytau-merge-foundation
Expand-Archive -LiteralPath $Zip -DestinationPath .\_kytau_tmp -Force
New-Item -ItemType Directory -Force .\kytau-merge | Out-Null
Copy-Item .\_kytau_tmp\KytauMerge\* .\kytau-merge\ -Recurse -Force
Remove-Item .\_kytau_tmp -Recurse -Force
git add .\kytau-merge
git commit -m "feat: add KyTau merge foundation"
git push origin feature/kytau-merge-foundation
```
