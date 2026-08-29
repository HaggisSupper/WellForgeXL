import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildApi7gWorkbook } from './build_api7g.mjs';
import { buildHydraulicsWorkbook } from './build_hydraulics.mjs';
import { buildTorqueDragWorkbook } from './build_torque_drag.mjs';
import { buildBhaWorkbook } from './build_bha.mjs';
import { buildDirectionalWorkbook } from './build_directional.mjs';
import { exportExchangeXlsx } from './exchange/export_exchange_xlsx.mjs';

const sourceDir = path.dirname(fileURLToPath(import.meta.url));
const outputDir = path.join(sourceDir, '..', 'outputs');
await fs.mkdir(outputDir, { recursive: true });
const builds = [
  ['API_7G_Drill_String_Strength_and_Torque_SI.xlsx', buildApi7gWorkbook],
  ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', buildHydraulicsWorkbook],
  ['Torque_Drag_and_Buckling_SI.xlsx', buildTorqueDragWorkbook],
  ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', buildBhaWorkbook],
  ['Directional_Drilling_Wellplan_and_Survey_SI.xlsx', buildDirectionalWorkbook],
];
for (const [name, build] of builds) {
  const exported = await exportExchangeXlsx(build());
  await exported.save(path.join(outputDir, name));
  process.stdout.write(`${name}\n`);
}
