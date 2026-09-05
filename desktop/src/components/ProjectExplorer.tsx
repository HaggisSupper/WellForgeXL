import { useProjectStore } from "../stores/project";
import type { DocumentInspection } from "../lib/ipc";

export function ProjectExplorer() {
  const activeProject = useProjectStore((state) => state.activeProject);
  const inspection = useProjectStore((state) => state.inspection);
  const selectionState = useProjectStore((state) => state.selectionState);
  const error = useProjectStore((state) => state.error);
  const selectAndInspectProject = useProjectStore((state) => state.selectAndInspectProject);
  const isBusy = selectionState === "selecting";

  return (
    <section aria-label="Project explorer" className="flex min-h-full flex-col">
      <header className="mb-8 flex flex-wrap items-end justify-between gap-4 border-b border-slate-700 pb-5">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Workspace</p>
          <h1 className="mt-1 text-2xl font-semibold text-white">Project explorer</h1>
          <p className="mt-2 text-sm text-slate-400">Select a local project to view backend-verified document facts.</p>
        </div>
        <button
          className="rounded bg-amber-400 px-4 py-2 text-sm font-semibold text-slate-950 hover:bg-amber-300 disabled:cursor-wait disabled:opacity-70"
          disabled={isBusy}
          onClick={() => void selectAndInspectProject()}
          type="button"
        >
          {isBusy ? "Opening…" : "Select project"}
        </button>
      </header>

      <div aria-live="polite" className="mb-5 text-sm text-slate-400">
        {selectionState === "cancelled" && "Project selection was cancelled."}
        {selectionState === "selecting" && "Waiting for project selection…"}
      </div>

      {error && (
        <div className="mb-5 rounded border border-red-500/50 bg-red-950/40 p-4 text-sm text-red-100" role="alert">
          <p className="font-semibold">{error.code}</p>
          <p className="mt-1">{error.message}</p>
        </div>
      )}

      {activeProject && (
        <section className="rounded-lg border border-slate-700 bg-slate-950/50 p-5" aria-label="Selected project">
          <h2 className="text-lg font-semibold text-white">{activeProject.name}</h2>
          {inspection ? <InspectionDetails inspection={inspection} /> : <p className="mt-5 text-sm text-slate-400">No inspection facts are available.</p>}
        </section>
      )}

      {!activeProject && !error && selectionState !== "cancelled" && (
        <section className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-600 bg-slate-900/60 p-8 text-center">
          <p className="max-w-lg text-sm leading-6 text-slate-400">Choose a supported local document to review its type, contents, and diagnostics.</p>
        </section>
      )}
    </section>
  );
}

function InspectionDetails({ inspection }: { inspection: DocumentInspection }) {
  const facts: Array<[string, string | number]> = [
    ["Document type", inspection.documentType],
    ["Root", inspection.rootName],
    ["Caption", inspection.caption ?? "Not provided"],
    ["Surveys", inspection.surveyCount],
    ["Components", inspection.componentCount],
  ];

  return (
    <>
      <dl className="mt-5 grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {facts.map(([label, value]) => <div key={label}><dt className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</dt><dd className="mt-1 text-sm text-slate-100">{value}</dd></div>)}
      </dl>
      <section className="mt-6 border-t border-slate-800 pt-4" aria-label="Diagnostics">
        <h3 className="text-sm font-semibold text-slate-200">Diagnostics ({inspection.diagnostics.length})</h3>
        {inspection.diagnostics.length === 0 ? <p className="mt-2 text-sm text-slate-400">No diagnostics were reported.</p> : (
          <ul className="mt-3 space-y-2">
            {inspection.diagnostics.map((diagnostic) => {
              const label = diagnostic.severity === "error" ? "Error" : "Warning";
              const className = diagnostic.severity === "error"
                ? "border-red-500/50 bg-red-950/30 text-red-100"
                : "border-amber-500/50 bg-amber-950/30 text-amber-100";
              return <li aria-label={`${label} diagnostic: ${diagnostic.code}. ${diagnostic.message}`} className={`rounded border px-3 py-2 text-sm ${className}`} key={`${diagnostic.code}-${diagnostic.message}`}><span className="mr-2 font-semibold">{label}</span><span className="mr-2 font-semibold">{diagnostic.code}</span>{diagnostic.message}</li>;
            })}
          </ul>
        )}
      </section>
    </>
  );
}
