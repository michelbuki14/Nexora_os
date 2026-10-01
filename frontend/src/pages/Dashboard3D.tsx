import { Suspense, lazy } from "react";
import { useIsMobile } from "../utils/is-mobile";
import { useToast } from "../components/Toast";

const DesignCanvas = lazy(() =>
  import("../components/DesignCanvas").then((m) => ({ default: m.DesignCanvas })),
);

export const Dashboard3D = () => {
  const isMobile = useIsMobile();
  const toast = useToast();

  const handleMilestone = (name: string) => {
    toast.info(`${name} — coming soon`);
  };

  const items = [
    { title: "Workforce", color: "#4F46E5", onClick: () => handleMilestone("Workforce") },
    { title: "Payroll", color: "#10B981", onClick: () => handleMilestone("Payroll") },
    { title: "Payments", color: "#F59E0B", onClick: () => handleMilestone("Payments") },
    { title: "Analytics", color: "#8B5CF6", onClick: () => handleMilestone("Analytics") },
    { title: "Security", color: "#EF4444", onClick: () => handleMilestone("Security") },
    { title: "Admin", color: "#0EA5E9", onClick: () => handleMilestone("Admin") },
  ];

  return (
    <div className="min-h-screen bg-slate-50">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded bg-brand-600 text-sm font-bold text-white">
              N
            </div>
            <span className="text-sm font-bold text-slate-900">Nexora OS</span>
          </div>
          <nav className="hidden items-center gap-6 text-sm font-medium text-secondary sm:flex">
            <a href="/workforce/employees" className="hover:text-slate-900">Workforce</a>
            <a href="/payroll" className="text-secondary hover:text-slate-900">Payroll</a>
            <a href="/analytics" className="text-secondary hover:text-slate-900">Analytics</a>
          </nav>
          <div className="flex items-center gap-3">
            <button className="flex w-full max-w-xs items-center gap-2 rounded-md border border-slate-300 bg-slate-50 px-3 py-1.5 text-sm text-secondary hover:border-slate-400 sm:hidden">
              <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
              Search…
            </button>
            <div className="hidden items-center gap-2 sm:flex">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-brand-600 text-xs font-bold text-white">
                DA
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="mx-auto max-w-6xl p-6">
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-slate-900">Nexora OS Dashboard</h1>
          <p className="mt-1 text-sm text-secondary">
            {isMobile
              ? "3D Interface — Tap cards to interact"
              : "3D Interface — Drag to rotate, Scroll to zoom"}
          </p>
        </div>

        <div className="overflow-hidden rounded-xl border border-slate-200 bg-white">
          <Suspense fallback={<div className="flex h-[60vh] items-center justify-center text-sm text-secondary">Loading 3D view…</div>}>
            <DesignCanvas items={items} cameraPosition={[0, 0, 7]} />
          </Suspense>
        </div>

        <div className="mt-8 rounded-xl border border-slate-200 bg-white p-6">
          <h2 className="text-lg font-semibold text-slate-900">Data Visualizations</h2>
          <p className="mt-1 text-sm text-secondary">
            Workforce distribution, payroll timeline, and tenant analytics will appear here once loaded.
          </p>
          <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-secondary">
            <span className="rounded-full bg-slate-100 px-3 py-1">📊 Workforce Bar Chart</span>
            <span className="rounded-full bg-slate-100 px-3 py-1">📈 Payroll Timeline</span>
            <span className="rounded-full bg-slate-100 px-3 py-1">🥧 Tenant Distribution</span>
          </div>
        </div>
      </div>
    </div>
  );
};
