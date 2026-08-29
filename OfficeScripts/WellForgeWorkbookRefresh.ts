/**
 * Optional Office Script companion for the WellForge workbooks.
 * It only requests recalculation and stamps refresh status; VBA/Rust calculation authority
 * remains responsible for the value-only engineering results.
 */
function main(workbook: ExcelScript.Workbook) {
  workbook.getApplication().calculate(ExcelScript.CalculationType.full);
  const checks = workbook.getWorksheet('Checks');
  const summary = workbook.getWorksheet('Summary');
  checks.getRange('A1').setValue('WellForge workbook refreshed');
  checks.getRange('B1').setValue(new Date().toISOString());
  summary.activate();
}
