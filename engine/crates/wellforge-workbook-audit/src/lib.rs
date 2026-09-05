//! Static extraction of workbook structure, labels, formulas, and defined names.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use calamine::{Data, Range, Reader, SheetType, SheetVisible, open_workbook_auto};
use serde::Serialize;

/// Version of the JSON audit contract.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// A text-bearing cell used as label and unit evidence.
#[derive(Debug, Serialize)]
pub struct TextCell {
    /// Representative Excel-style cell addresses.
    pub cells: Vec<String>,
    /// Cell text.
    pub text: String,
    /// Number of cells containing the same text.
    pub occurrences: usize,
}

/// A worksheet or macro-sheet formula.
#[derive(Debug, Serialize)]
pub struct FormulaCell {
    /// Excel-style cell address.
    pub cell: String,
    /// Formula text prefixed with `=`.
    pub formula: String,
    /// Whether Calamine decoded the expression rather than returning a diagnostic.
    pub parsed: bool,
}

/// Static metadata and calculation surfaces for one sheet.
#[derive(Debug, Serialize)]
pub struct SheetAudit {
    /// One-based position in workbook order.
    pub sheet_index: usize,
    /// Original sheet name. Raw audit output is intentionally kept private.
    pub sheet_name: String,
    /// Worksheet, macro sheet, chart sheet, dialog sheet, or VBA module.
    pub sheet_kind: String,
    /// Visible, hidden, or very hidden.
    pub visibility: String,
    /// Number of rows spanning the cached-value range.
    pub rows_used: usize,
    /// Number of columns spanning the cached-value range.
    pub columns_used: usize,
    /// Number of non-empty cached-value cells.
    pub populated_cells: usize,
    /// Text cells available for semantic and unit analysis.
    pub text_cells: Vec<TextCell>,
    /// Formula cells parsed without opening Excel.
    pub formulas: Vec<FormulaCell>,
    /// Non-fatal parser limitations for this sheet.
    pub warnings: Vec<String>,
}

/// Workbook-defined expression or range.
#[derive(Debug, Serialize)]
pub struct DefinedNameAudit {
    /// Defined-name identifier.
    pub name: String,
    /// Static formula or range expression.
    pub formula: String,
}

/// Complete static audit emitted by the helper CLI.
#[derive(Debug, Serialize)]
pub struct WorkbookAudit {
    /// JSON contract version.
    pub schema_version: &'static str,
    /// Input filename extension, without a path.
    pub extension: String,
    /// Sheets in workbook order.
    pub sheets: Vec<SheetAudit>,
    /// Workbook-defined names.
    pub defined_names: Vec<DefinedNameAudit>,
}

/// Convert a zero-based row and column into an Excel A1 reference.
#[must_use]
pub fn cell_reference(row: u32, column: u32) -> String {
    let mut dividend = u64::from(column) + 1;
    let mut letters = Vec::new();
    while dividend > 0 {
        let remainder = (dividend - 1) % 26;
        letters.push(char::from(b'A' + u8::try_from(remainder).unwrap_or(0)));
        dividend = (dividend - 1) / 26;
    }
    letters.reverse();
    format!(
        "{}{}",
        letters.into_iter().collect::<String>(),
        u64::from(row) + 1
    )
}

/// Return a stable textual sheet-type value for the JSON contract.
#[must_use]
pub fn sheet_kind(kind: SheetType) -> &'static str {
    match kind {
        SheetType::WorkSheet => "worksheet",
        SheetType::DialogSheet => "dialog-sheet",
        SheetType::MacroSheet => "macro-sheet",
        SheetType::ChartSheet => "chart-sheet",
        SheetType::Vba => "vba-module",
    }
}

/// Return a stable textual visibility value for the JSON contract.
#[must_use]
pub fn sheet_visibility(visibility: SheetVisible) -> &'static str {
    match visibility {
        SheetVisible::Visible => "visible",
        SheetVisible::Hidden => "hidden",
        SheetVisible::VeryHidden => "veryHidden",
    }
}

fn absolute_cell(start: Option<(u32, u32)>, relative_row: usize, relative_column: usize) -> String {
    let (start_row, start_column) = start.unwrap_or((0, 0));
    let row = start_row.saturating_add(u32::try_from(relative_row).unwrap_or(u32::MAX));
    let column = start_column.saturating_add(u32::try_from(relative_column).unwrap_or(u32::MAX));
    cell_reference(row, column)
}

fn formula_with_marker(formula: &str) -> String {
    if formula.starts_with('=') {
        formula.to_owned()
    } else {
        format!("={formula}")
    }
}

fn is_near_formula(row: u32, column: u32, formulas: &HashSet<(u32, u32)>) -> bool {
    (-4_i64..=4).any(|row_delta| {
        (-4_i64..=4).any(|column_delta| {
            let aligned = row_delta == 0
                || column_delta == 0
                || (row_delta.abs() <= 2 && column_delta.abs() <= 2);
            if !aligned {
                return false;
            }
            let candidate_row = i64::from(row) + row_delta;
            let candidate_column = i64::from(column) + column_delta;
            candidate_row >= 0
                && candidate_column >= 0
                && formulas.contains(&(
                    u32::try_from(candidate_row).unwrap_or(u32::MAX),
                    u32::try_from(candidate_column).unwrap_or(u32::MAX),
                ))
        })
    })
}

fn summarize_value_range(
    range: &Range<Data>,
    formula_addresses: &HashSet<String>,
    formula_positions: &HashSet<(u32, u32)>,
) -> (usize, usize, usize, Vec<TextCell>) {
    let start = range.start();
    let (rows, columns) = range.get_size();
    let populated = range.used_cells().count();
    let mut labels: BTreeMap<String, (usize, Vec<String>)> = BTreeMap::new();
    for (row, column, value) in range.used_cells() {
        let Data::String(text) = value else {
            continue;
        };
        let cell = absolute_cell(start, row, column);
        if formula_addresses.contains(&cell) {
            continue;
        }
        let absolute_position = start.map(|(start_row, start_column)| {
            (
                start_row.saturating_add(u32::try_from(row).unwrap_or(u32::MAX)),
                start_column.saturating_add(u32::try_from(column).unwrap_or(u32::MAX)),
            )
        });
        let entry = labels.entry(text.clone()).or_insert((0, Vec::new()));
        entry.0 += 1;
        let retain_coordinate = entry.1.len() < 8
            || absolute_position.is_some_and(|(absolute_row, absolute_column)| {
                is_near_formula(absolute_row, absolute_column, formula_positions)
            });
        if retain_coordinate && !entry.1.contains(&cell) {
            entry.1.push(cell);
        }
    }
    let text_cells = labels
        .into_iter()
        .map(|(text, (occurrences, cells))| TextCell {
            cells,
            text,
            occurrences,
        })
        .collect();
    (rows, columns, populated, text_cells)
}

/// Parse a workbook with calamine without launching Microsoft Excel.
///
/// # Errors
///
/// Returns an error when the workbook cannot be opened or its container cannot
/// be parsed far enough to enumerate the workbook metadata.
pub fn audit_workbook(path: &Path) -> Result<WorkbookAudit> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut workbook = open_workbook_auto(path)
        .with_context(|| format!("could not statically open {}", path.display()))?;

    let sheet_metadata = workbook.sheets_metadata().to_vec();
    let defined_names = workbook
        .defined_names()
        .iter()
        .map(|(name, formula)| DefinedNameAudit {
            name: name.clone(),
            formula: formula_with_marker(formula),
        })
        .collect();
    let mut sheets = Vec::with_capacity(sheet_metadata.len());

    for (sheet_index, metadata) in sheet_metadata.into_iter().enumerate() {
        let mut warnings = Vec::new();
        if matches!(extension.as_str(), "xls" | "xlsb") {
            warnings
                .push("binary-shared-and-array-formula-records-are-not-reconstructed".to_owned());
        }
        let formula_range = workbook.worksheet_formula(&metadata.name);
        let mut formulas = Vec::new();
        let mut formula_addresses = HashSet::new();
        let mut formula_positions = HashSet::new();
        match formula_range {
            Ok(range) => {
                let start = range.start();
                for (row, column, formula) in range.used_cells() {
                    if formula.is_empty() {
                        continue;
                    }
                    let cell = absolute_cell(start, row, column);
                    let parsed = !formula.starts_with("Unrecognised formula for cell");
                    if !parsed {
                        warnings.push(format!("unparsed-formula:{cell}"));
                    }
                    formula_addresses.insert(cell.clone());
                    if let Some((start_row, start_column)) = start {
                        formula_positions.insert((
                            start_row.saturating_add(u32::try_from(row).unwrap_or(u32::MAX)),
                            start_column.saturating_add(u32::try_from(column).unwrap_or(u32::MAX)),
                        ));
                    }
                    formulas.push(FormulaCell {
                        cell,
                        formula: formula_with_marker(formula),
                        parsed,
                    });
                }
            }
            Err(error) => warnings.push(format!("formula-range: {error}")),
        }

        let value_range = workbook.worksheet_range(&metadata.name);
        let (rows_used, columns_used, populated_cells, text_cells) = match value_range {
            Ok(range) => summarize_value_range(&range, &formula_addresses, &formula_positions),
            Err(error) => {
                warnings.push(format!("value-range: {error}"));
                (0, 0, 0, Vec::new())
            }
        };

        sheets.push(SheetAudit {
            sheet_index: sheet_index + 1,
            sheet_name: metadata.name,
            sheet_kind: sheet_kind(metadata.typ).to_owned(),
            visibility: sheet_visibility(metadata.visible).to_owned(),
            rows_used,
            columns_used,
            populated_cells,
            text_cells,
            formulas,
            warnings,
        });
    }

    Ok(WorkbookAudit {
        schema_version: SCHEMA_VERSION,
        extension,
        sheets,
        defined_names,
    })
}
