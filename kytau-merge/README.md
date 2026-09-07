# KyTau Merge Foundation

KyTau Merge is a Windows-first, Rust-first agentic development foundation synthesized from the associated conversation code and design work.

This subtree is intended to become either:

1. a standalone `KytauMerge` / `KyTau` repository, or
2. a governed foundation subtree inside a larger WellForge/Veritas-aligned workspace.

## High-level capability

KyTau Merge provides:

- immutable contract schemas;
- Rust workspace structure;
- local LLM provider hooks for `mistral.rs` primary and `llama.cpp` fallback;
- PowerShell bootstrap, validation, test, and packaging scripts;
- agent role briefs for Codex, Claude, Gemini, OpenCode, and global command profiles;
- evidence ledger and manifest discipline;
- Tauri desktop shell scaffold;
- test and validation harnesses.

## WellForge reference alignment

This landing branch references `desktop/WELLFORGE-SOURCE-MIGRATION-CATALOG.md` as the migration-governance anchor. The relevant WellForge lessons carried forward are:

- do not mechanically merge legacy trees;
- extract behavior into small Rust modules with contracts and tests;
- preserve fixture evidence for numerical and format compatibility;
- rebuild desktop UX in Tauri from workflow specifications;
- keep generated/vendor/build output out of source intake.

## Current sync state

This branch currently contains the GitHub landing notes. The full generated package exists as `KytauMerge.zip` from the ChatGPT artifact and should be expanded into this subtree to complete the source sync.

Expected full layout after expansion:

```text
kytau-merge/
  .claude/
  .codex/
  .gemini/
  .promptset.json
  AGENTS.md
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

## Windows completion

```powershell
$Repo = "$env:USERPROFILE\source\repos\WellForgeXL"
$Zip = "$env:USERPROFILE\Downloads\KytauMerge.zip"
cd $Repo
git fetch origin
git checkout feature/kytau-merge-foundation
Expand-Archive -LiteralPath $Zip -DestinationPath .\_kytau_tmp -Force
Copy-Item .\_kytau_tmp\KytauMerge\* .\kytau-merge\ -Recurse -Force
Remove-Item .\_kytau_tmp -Recurse -Force
pwsh .\kytau-merge\scripts\validate.ps1
pwsh .\kytau-merge\scripts\test.ps1
git add .\kytau-merge
git commit -m "feat: add KyTau merge foundation"
git push origin feature/kytau-merge-foundation
```
