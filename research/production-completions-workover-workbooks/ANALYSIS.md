# Production / completions / workover workbook analysis

Analysis date: 2026-09-05

## Verified outcome

A curated adjacent corpus of **19 unique legacy calculators** was statically inspected. The canonical Rust `wellforge-workbook-audit` binary was built in GitHub Actions and then run locally against the private workbook bytes. It successfully parsed 17 readable workbooks and rejected two password-protected BIFF bodies, which are retained only as limited static evidence.

Canonical readable totals are **93 sheets, 2,052 recovered formula records, 1,551 structural family rows, and 981 defined names**. For `.xls`, the canonical reader explicitly warns that shared/array formulas are not fully reconstructed, so formula-record totals are lower bounds rather than exhaustive cell counts.

The corpus also contains 10 workbooks with VBA storage, with 42 modules and 142 procedures statically inventoried. No code was executed.

## Highest-value newly surfaced capability families

- **Production:** flow-rate calculations, choke behavior, orifice/metering, separator/process sizing, skin/productivity and blowdown/system-friction evidence.
- **Completions:** perforating/underbalance support, coiled-tubing data, brine properties/equilibrium and nitrogen-property calculations.
- **Workover/intervention:** U-tubing, cement plugs, volume/displacement and kill-operation calculations.
- **Well control:** kick/kill surfaces, MAASP, FIT/LOT and volumetric-control calculations independent of the earlier drilling set.
- **Tubular/mechanical:** capacity/strength, connection torque and load-vs-torque calculations.

## Cross-corpus challenger evidence

The adjacent set independently implements pressure-loss/ECD calculations, rheology, nozzles, motor performance, thermal support and cementing. These are useful parity/challenger sources for existing or planned drilling engines, not new model authority.

## Encryption and legacy-BIFF limitation

Two sources contain BIFF `FILEPASS` protection. Their worksheet formula/name bodies are excluded from numeric totals because the canonical reader correctly refuses password-protected workbooks. A prior non-canonical compatibility scan was used only to locate static storage/text/VBA evidence and is not used for published formula totals.

The canonical reader additionally reports `binary-shared-and-array-formula-records-are-not-reconstructed` for legacy `.xls`; therefore the 2,052 recovered formula records are a lower bound. The family count is derived only from formulas actually recovered by Rust.

## Architecture implication

The strongest genuinely adjacent capabilities are production flow/choke/metering/separator calculations; dedicated well-control/FIT-LOT; completions/workover fluid/volume/U-tubing; cement-plug utilities; and a versioned tubular-capacity service. These should remain separate capability lanes while sharing WellForge canonical units, wellbore geometry, provenance, and evidence contracts.
