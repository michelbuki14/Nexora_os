import { useQuery } from "@tanstack/react-query";
import { API_BASE } from "../api/client";

/**
 * AOS system status. Polls the live service's /health (in Phase 1 dev that's
 * the workforce service at VITE_API_BASE — the gateway at :3000 is not
 * proxying yet and shares the SPA port in dev). No token required; the
 * health router is merged at root.
 */
export function StatusPill() {
  const { data } = useQuery({
    queryKey: ["aos-health"],
    queryFn: async () => {
      const res = await fetch(`${API_BASE}/health`);
      return res.ok;
    },
    refetchInterval: 30_000,
    retry: 1,
  });

  const operational = data === true;

  return (
    <span className="inline-flex items-center gap-1.5 rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs text-slate-600">
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
