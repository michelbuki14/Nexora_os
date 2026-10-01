import { useQuery } from "@tanstack/react-query";
import { useSessionStore } from "../auth/session";
import { API_BASE } from "../api/client";

/**
 * Nexora OS system status. Polls the live service's /health when a backend is
 * configured. In demo mode (no backend reachable) it shows a clear, honest
 * "Demo workspace" state so operators never mistake seeded data for production.
 */
export function StatusPill() {
  const demo = useSessionStore((s) => s.demo);

  const { data } = useQuery({
    queryKey: ["nexora-health"],
    queryFn: async () => {
      const res = await fetch(`${API_BASE}/health`);
      return res.ok;
    },
    refetchInterval: 30_000,
    retry: 1,
    enabled: !demo && !!API_BASE,
  });

  if (demo) {
    return (
      <span className="inline-flex items-center gap-1.5 rounded-full border border-amber-200 bg-amber-50 px-2.5 py-1 text-xs font-medium text-amber-700">
        <span className="h-2 w-2 rounded-full bg-amber-500" aria-hidden />
        Demo workspace
      </span>
    );
  }

  const operational = data === true;
  return (
    <span className="inline-flex items-center gap-1.5 rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs text-secondary">
      <span
        className={`h-2 w-2 rounded-full ${
          operational ? "bg-emerald-500" : "bg-amber-500"
        }`}
        aria-hidden
      />
      {operational ? "Operational" : "Degraded"}
    </span>
  );
}
