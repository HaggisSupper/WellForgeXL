/**
 * Optional Office Script companion for the WellForge formula workbooks.
 * It never writes engineering results: those remain Excel formulas.
 */
function main(workbook: ExcelScript.Workbook) {
  workbook.getApplication().calculate(ExcelScript.CalculationType.full);
  const checks = workbook.getWorksheet('Checks');
  const summary = workbook.getWorksheet('Summary');
  checks.getRange('A1').setValue('WellForge workbook refreshed');
  checks.getRange('B1').setValue(new Date().toISOString());
  summary.activate();
}

