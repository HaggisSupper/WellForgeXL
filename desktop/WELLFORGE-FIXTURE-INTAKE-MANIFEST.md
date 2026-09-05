# WellForge Fixture Intake Manifest — Initial Candidate Set

## Status

This is an inventory only. Nothing has been copied from the supplied archive.
Each candidate requires de-identification, license/provenance review, and an
approved fixture name before it enters the WellForge repository.

## Candidate fixtures

| ID | Purpose | Type | Size | SHA-256 |
| --- | --- | --- | ---: | --- |
| FX-001 | Small XML project in one measurement system | XML | 5,336 B | `5C43542171B663D719B1585184B490D5FBE203D71594A1F5628680FB039649CE` |
| FX-002 | Equivalent small XML project in a second measurement system | XML | 5,274 B | `DF6C8D995AB1CE9EBCE4850FA6E894FBC3CDB4F17685EA5053BE1FF8940BB5D5` |
| FX-003 | Complete portable project document | project XML | 188,982 B | `CAE59A3CBF4CB5F5FED80128271D1551DC8C7DA74963495B6A5B17AFA0E7958B` |
| FX-004 | Broad BHA component coverage | BHA XML | 76,752 B | `898D0EAC16CADE14B22B05BE61F02C28386E14A3AD586B2ACF12F28D3559BB1B` |
| FX-005 | RSS BHA variant | BHA XML | 41,960 B | `4AED24356B3E94B48FE3D256DA685D8AE58C5C6B92C49E77384BBDC02A8FDB67` |
| FX-006 | Valid trajectory-import case | CSV | 185 B | `DA5885F8106E3D697BDE413D04F279456BB7ED254422AEE2B2393912DC325F7A` |
| FX-007 | Invalid trajectory-import case | CSV | 178 B | `A678024E4C0DB07481B2CEB3482FFD8C8ACE1CBEBAA370E39A6728A1FF88E707` |
| FX-008 | Unit-system import/map | workbook | 90,165 B | `B74BB74FED56B6CB0DA23C5728BE3DD38D45187FB9DA1F3E87670AB32949657E` |
| FX-009 | Unit mapping import | CSV | 3,135 B | `27879A1BF3C215844D4BA196FF23EC04295F4B1B88074FBEF87F175485C273EF` |
| FX-010 | Catalog tool-list import | workbook | 821,730 B | `10CEF964F944AFF7723F6A069CBE7A10112D442DBC8B664EAF023738A0FCE9E9` |

## Required fixture metadata

Each approved fixture must have:

- fixture ID and stable sanitized filename;
- origin class and approval record;
- original source hash and approved-copy hash;
- anonymization record, if needed;
- expected parse/validation result;
- expected semantic-round-trip result;
- expected calculations and tolerance, if applicable; and
- expected report snapshot or catalog-import outcome, if applicable.

## First test matrix

| Test | Fixtures | Pass condition |
| --- | --- | --- |
| XML parse and validation | FX-001 to FX-005 | Valid input becomes an explicit typed model; unsupported content is retained where required |
| Measurement-system equivalence | FX-001, FX-002 | Equivalent physical values agree after conversion |
| XML semantic round trip | FX-003 to FX-005 | Re-read result is semantically identical to the source model |
| BHA hierarchy | FX-004, FX-005 | Component order, properties, and specialized variants remain intact |
| Trajectory import | FX-006, FX-007 | Valid file imports; invalid file produces a deterministic validation error |
| Unit import | FX-008, FX-009 | Unit identities and conversion relationships are verified |
| Catalog staging | FX-010 | Import is validated into staging before any production promotion |

## Next gate

Approve the selected data for de-identification. After approval, place only the
sanitized copies in `wellforge-fixtures`, add a machine-readable manifest, and
begin the Rust XML and unit test harness.
