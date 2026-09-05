import { wellforgeModules, useUiStore } from "../stores/ui";

export function Sidebar() {
  const activeModule = useUiStore((state) => state.activeModule);
  const setActiveModule = useUiStore((state) => state.setActiveModule);

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-slate-700 bg-slate-950 px-3 py-5">
      <div className="mb-8 px-3">
        <p className="text-xs font-semibold uppercase tracking-[0.24em] text-amber-400">WellForge</p>
        <p className="mt-1 text-sm text-slate-400">Drilling engineering</p>
      </div>
      <nav aria-label="WellForge modules" className="space-y-1">
        {wellforgeModules.map((module) => {
          const isActive = activeModule === module;
          return (
            <button
              aria-current={isActive ? "page" : undefined}
              className={`w-full rounded px-3 py-2 text-left text-sm transition ${
                isActive
                  ? "bg-slate-700 font-semibold text-white"
                  : "text-slate-300 hover:bg-slate-800 hover:text-white"
              }`}
              key={module}
              onClick={() => setActiveModule(module)}
              type="button"
            >
              {module}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
