import { useProjectStore } from "../stores/project";
import { useUiStore } from "../stores/ui";
import { SurveyGrid } from "./SurveyGrid";
import { ThreeDViewport } from "./ThreeDViewport";
import { useSceneStore } from "../stores/scene";
import { ProjectExplorer } from "./ProjectExplorer";
import { AuditWorkspace } from "./AuditWorkspace";

export function EmptyCanvas() {
  const activeModule = useUiStore((state) => state.activeModule);
  const activeProject = useProjectStore((state) => state.activeProject);
  const scene = useSceneStore((state) => state.scene);

  if (activeModule === "Project") {
    return <main className="min-w-0 flex-1 bg-slate-900 p-8"><ProjectExplorer /></main>;
  }

  if (activeModule === "Surveys") {
    return <main className="min-w-0 flex-1 bg-slate-900 p-8" aria-label="Survey workspace"><header className="mb-8 border-b border-slate-700 pb-5"><p className="text-xs uppercase tracking-[0.18em] text-slate-500">Workspace</p><h1 className="mt-1 text-2xl font-semibold text-white">Surveys</h1></header><div className="grid gap-5 xl:grid-cols-2"><div><SurveyGrid rows={[]} /><p className="mt-4 text-sm text-slate-400">No stations loaded. Survey values are calculated in Rust and saved in the active project artifact.</p></div><ThreeDViewport scene={scene} /></div></main>;
  }
  if (activeModule === "Audit") {
    return <main className="min-w-0 flex-1 bg-slate-900 p-8"><AuditWorkspace /></main>;
  }
  return (
    <main className="flex min-w-0 flex-1 flex-col bg-slate-900 p-8" aria-label="Workspace canvas">
      <header className="mb-8 border-b border-slate-700 pb-5">
        <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Workspace</p>
        <h1 className="mt-1 text-2xl font-semibold text-white">{activeModule}</h1>
      </header>
      <section className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-600 bg-slate-900/60 p-8 text-center">
        <div>
          <p className="text-lg font-medium text-slate-200">{activeModule} is ready for its engineering module.</p>
          <p className="mt-2 max-w-lg text-sm leading-6 text-slate-400">
            {activeProject
              ? `Active local project: ${activeProject.name}`
              : "Open a local project to begin. Engineering calculations are implemented in their dedicated Rust crates."}
          </p>
        </div>
      </section>
    </main>
  );
}
