import { useEffect, useState } from "react";
import { wellforgeIpc, type ApiError, type ProjectAudit } from "../lib/ipc";
import { useProjectStore } from "../stores/project";

type AuditState =
  | { status: "idle" | "loading" }
  | { status: "ready"; audit: ProjectAudit }
  | { status: "error"; error: ApiError };

export function AuditWorkspace() {
  const activeProject = useProjectStore((state) => state.activeProject);
  const [auditState, setAuditState] = useState<AuditState>({ status: "idle" });

  useEffect(() => {
    let current = true;
    if (!activeProject) {
      setAuditState({ status: "idle" });
      return () => { current = false; };
    }
    setAuditState({ status: "loading" });
    void wellforgeIpc.getProjectAudit()
      .then((audit) => { if (current) setAuditState({ status: "ready", audit }); })
      .catch((error: unknown) => { if (current) setAuditState({ status: "error", error: displayError(error) }); });
    return () => { current = false; };
  }, [activeProject]);

  return (
    <section aria-label="Project audit" className="flex min-h-full flex-col">
      <header className="mb-8 border-b border-slate-700 pb-5">
        <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Workspace</p>
        <h1 className="mt-1 text-2xl font-semibold text-white">Audit</h1>
        <p className="mt-2 text-sm text-slate-400">Immutable revision lineage and calculation provenance for the active project.</p>
      </header>

      {!activeProject && <EmptyAuditState message="Open a local project to review its immutable audit metadata." />}
      {activeProject && auditState.status === "loading" && <p aria-live="polite" className="text-sm text-slate-400">Loading audit metadata…</p>}
      {auditState.status === "error" && <div className="rounded border border-red-500/50 bg-red-950/40 p-4 text-sm text-red-100" role="alert"><p className="font-semibold">{auditState.error.code}</p><p className="mt-1">Audit metadata could not be loaded.</p></div>}
      {auditState.status === "ready" && <AuditDetails audit={auditState.audit} />}
    </section>
  );
}

function AuditDetails({ audit }: { audit: ProjectAudit }) {
  if (audit.revisions.length === 0 && audit.calculationReceipts.length === 0) {
    return <EmptyAuditState message="No immutable revisions or calculation receipts are available for this project." />;
  }
  return <div className="space-y-6">
    <section aria-label="Revision lineage" className="rounded-lg border border-slate-700 bg-slate-950/50 p-5">
      <h2 className="text-lg font-semibold text-white">Revision lineage</h2>
      {audit.revisions.length === 0 ? <p className="mt-3 text-sm text-slate-400">No revisions are available.</p> : <ul className="mt-4 space-y-3">{audit.revisions.map((revision) => <li className="rounded border border-slate-800 p-3 text-sm text-slate-200" key={revision.id}><p className="font-medium text-white">{revision.id}</p><dl className="mt-2 grid gap-2 sm:grid-cols-2"><Fact label="Created" value={revision.createdAt} /><Fact label="Actor" value={revision.actorId} /><Fact label="Parent revision" value={revision.parentRevisionId ?? "Root revision"} /><Fact label="Content SHA-256" value={revision.contentSha256} /></dl></li>)}</ul>}
    </section>
    <section aria-label="Calculation receipts" className="rounded-lg border border-slate-700 bg-slate-950/50 p-5">
      <h2 className="text-lg font-semibold text-white">Calculation receipts</h2>
      {audit.calculationReceipts.length === 0 ? <p className="mt-3 text-sm text-slate-400">No calculation receipts are available.</p> : <ul className="mt-4 space-y-3">{audit.calculationReceipts.map((receipt) => <li className="rounded border border-slate-800 p-3 text-sm text-slate-200" key={receipt.id}><p className="font-medium text-white">{receipt.algorithm}</p><dl className="mt-2 grid gap-2 sm:grid-cols-2"><Fact label="Receipt ID" value={receipt.id} /><Fact label="Version" value={receipt.algorithmVersion} /><Fact label="Recorded" value={receipt.recordedAt} /><Fact label="Actor" value={receipt.actorId} /><Fact label="Project revision" value={receipt.projectRevisionId} /><Fact label="Revision content SHA-256" value={receipt.projectRevisionContentSha256} /><Fact label="Receipt SHA-256" value={receipt.contentSha256} /><Fact label="Output SHA-256" value={receipt.outputSha256} /></dl>{receipt.warningCount > 0 && <p className="mt-3 text-sm text-amber-200">{receipt.warningCount} warning{receipt.warningCount === 1 ? "" : "s"} recorded</p>}</li>)}</ul>}
    </section>
  </div>;
}

function Fact({ label, value }: { label: string; value: string }) {
  return <div><dt className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</dt><dd className="mt-1 break-all text-slate-100">{value}</dd></div>;
}

function EmptyAuditState({ message }: { message: string }) {
  return <section className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-600 bg-slate-900/60 p-8 text-center"><p className="max-w-lg text-sm leading-6 text-slate-400">{message}</p></section>;
}

function displayError(_value: unknown): ApiError {
  return { code: "AUDIT_LOAD_FAILED", message: "Audit metadata could not be loaded." };
}
