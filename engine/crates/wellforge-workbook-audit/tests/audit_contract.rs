//! Contract tests for the static workbook audit helper.

use calamine::{SheetType, SheetVisible};
use wellforge_workbook_audit::{cell_reference, sheet_kind, sheet_visibility};

#[test]
fn cell_references_are_one_based_excel_addresses() {
    assert_eq!(cell_reference(0, 0), "A1");
    assert_eq!(cell_reference(27, 702), "AAA28");
}

#[test]
fn macro_and_hidden_sheet_metadata_remain_explicit() {
    assert_eq!(sheet_kind(SheetType::MacroSheet), "macro-sheet");
    assert_eq!(sheet_kind(SheetType::WorkSheet), "worksheet");
    assert_eq!(sheet_visibility(SheetVisible::VeryHidden), "veryHidden");
    assert_eq!(sheet_visibility(SheetVisible::Hidden), "hidden");
}
