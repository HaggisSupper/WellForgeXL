export interface SurveyGridRow {
  mdM: number;
  inclinationRad: number;
  azimuthTrueRad: number;
  northM: number;
  eastM: number;
  tvdM: number;
  dlsRadPerM: number;
}

export function SurveyGrid({ rows }: { rows: readonly SurveyGridRow[] }) {
  const columns: Array<[keyof SurveyGridRow, string]> = [
    ["mdM", "MD (m)"], ["inclinationRad", "Inc (rad)"], ["azimuthTrueRad", "Az (rad)"],
    ["northM", "N (m)"], ["eastM", "E (m)"], ["tvdM", "TVD (m)"], ["dlsRadPerM", "DLS (rad/m)"],
  ];
  return (
    <div className="overflow-auto rounded border border-slate-700">
      <table className="w-full text-left text-xs text-slate-300">
        <thead className="bg-slate-800 text-slate-400"><tr>{columns.map(([, label]) => <th className="px-3 py-2 font-medium" key={label}>{label}</th>)}</tr></thead>
        <tbody>{rows.map((row, index) => <tr className="border-t border-slate-800" key={`${row.mdM}-${index}`}>{columns.map(([key]) => <td className="px-3 py-2 tabular-nums" key={key}>{row[key].toFixed(3)}</td>)}</tr>)}</tbody>
      </table>
    </div>
  );
}
