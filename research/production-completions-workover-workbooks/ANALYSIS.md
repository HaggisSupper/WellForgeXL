# Production / completions / workover workbook analysis

Analysis date: 2026-09-05

## Outcome

The adjacent corpus materially broadens WellForge beyond the original drilling-only workbook inventory. The selected 19 workbooks contain 2,650 readable formula records compacted into 1,302 structural families. Unlike the three very large drilling hydraulics parity tables, this corpus has a high family-to-formula ratio, indicating diverse small engineering calculators rather than repeated batch tables.

Two source workbooks are BIFF-encrypted. Their encrypted worksheet record streams are not counted as formulas or sheets; readable compound-file/VBA/text metadata is retained only as limited static evidence.

## New first-class capability evidence

The strongest genuinely new surfaces are:

- **Production:** flow-rate calculations, choke behavior, orifice/metering, separator/process sizing, skin/productivity and blowdown/system-friction evidence.
- **Completions:** perforating/underbalance support, coiled-tubing data, brine properties/equilibrium and nitrogen-property calculations.
- **Workover/intervention:** U-tubing, cement plugs, volume/displacement and kill-operation calculations.
- **Well control:** kick/kill surfaces, MAASP, FIT/LOT and volumetric-control calculations independent of the earlier drilling workbook set.
- **Tubular/mechanical:** capacity/strength, connection torque, and load-vs-torque calculations.

## Cross-corpus evidence

The adjacent corpus also provides independent workbook implementations for hydraulics pressure loss/ECD, rheology, nozzles, motor performance, thermal support and cementing. Those are valuable parity/challenger sources but do not create new model authority.

## Implementation implications

Highest-value capability additions exposed by this corpus are: production flow/choke/metering; dedicated well-control and FIT/LOT; completions/workover fluid/volume/U-tubing; cement-plug utilities; and a versioned tubular-capacity service. These should remain separate from the deterministic drilling kernels while sharing canonical units, trajectory/wellbore geometry, provenance, and evidence contracts.

## Limitations

This initial tranche is deliberately curated rather than exhaustive across the 3,981-file Technical Reference Tools collection. It establishes a clean second-corpus pipeline and high-value seed set. Encrypted worksheet bodies require an approved static decryption path before formula-level claims. Raw binaries stay outside the public repository.
