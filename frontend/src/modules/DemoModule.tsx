import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { Alert, Badge, Button, Card, MetricCard, PageHeader } from "../components/ui";
import { MODULE_META, type ModuleMeta } from "../api/moduleDemoData";
import {
  addModuleRow,
  getModuleTable,
  runModuleAction,
  toggleModuleRow,
  type ModuleTableRow,
} from "../api/demoData";

function titleCase(s: string): string {
  return s[0].toUpperCase() + s.slice(1);
}

function seedRowFor(name: string): ModuleTableRow | null {
  switch (name) {
    case "payroll":
      return { period: "—", gross: "$0", employees: 0, status: "Draft", paid_on: "—" };
    case "finance":
      return { date: new Date().toISOString().slice(0, 10), account: "Manual", memo: "Adjustment", amount: 0, balance: 0 };
    case "payments":
      return { id: "PAY-" + Math.floor(1000 + Math.random() * 9000), payee: "New payee", method: "Card", amount: "$0", status: "Pending", date: new Date().toISOString().slice(0, 10) };
    case "developer":
      return { name: "New key", key: "sk_live_••••" + Math.random().toString(36).slice(2, 6), created: new Date().toISOString().slice(0, 10), last_used: "—" };
    case "organizations":
      return { name: "New Org", plan: "Starter", seats: 1, status: "Trial" };
    default:
      return null;
  }
}

function AnalyticsBars({ rows }: { rows: ModuleTableRow[] }) {
  const metric = "active_users";
  const seed = rows.map((r) => Number(r[metric]) || 0);
  const [series, setSeries] = useState<number[]>(() => {
    const padded = [...seed, ...Array.from({ length: 12 }, () => 0)].slice(-24);
    return padded.map((v, i) => (v === 0 ? Math.round(3000 + Math.sin(i / 2) * 400 + Math.random() * 300) : v));
  });
  const [last, setLast] = useState<number>(seed[seed.length - 1] || 3200);

  // Live tick: append a new point and slide the window, every 2s.
  useEffect(() => {
    const id = setInterval(() => {
      setLast((prev) => {
        const next = Math.max(800, Math.round(prev + (Math.random() - 0.45) * 320));
        setSeries((s) => [...s.slice(1), next]);
        return next;
      });
    }, 2000);
    return () => clearInterval(id);
  }, []);

  const W = 640;
  const H = 200;
  const max = Math.max(...series, 1) * 1.1;
  const min = Math.min(...series, 0) * 0.9;
  const range = Math.max(1, max - min);
  const pts = series.map((v, i) => {
    const x = (i / (series.length - 1)) * W;
    const y = H - ((v - min) / range) * H;
    return [x, y] as const;
  });
  const line = pts.map((p, i) => `${i === 0 ? "M" : "L"}${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(" ");
  const area = `${line} L${W},${H} L0,${H} Z`;

  return (
    <Card className="p-5">
      <div className="mb-3 flex items-center justify-between">
        <p className="text-sm font-medium text-secondary">Active users — live</p>
        <span className="inline-flex items-center gap-1.5 text-xs text-secondary" aria-live="polite">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500" />
          </span>
          {last.toLocaleString()} now
        </span>
      </div>
      <svg viewBox={`0 0 ${W} ${H}`} className="h-52 w-full" aria-hidden="true">
        <defs>
          <linearGradient id="nexoraArea" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#0066FF" stopOpacity="0.35" />
            <stop offset="100%" stopColor="#0066FF" stopOpacity="0" />
          </linearGradient>
        </defs>
        {[0.25, 0.5, 0.75].map((g) => (
          <line key={g} x1="0" x2={W} y1={H * g} y2={H * g} stroke="#E2E5EA" strokeWidth="1" strokeDasharray="3 3" />
        ))}
        <path d={area} fill="url(#nexoraArea)" />
        <path d={line} fill="none" stroke="#0066FF" strokeWidth="2.5" strokeLinejoin="round" strokeLinecap="round" />
        <circle cx={pts[pts.length - 1][0]} cy={pts[pts.length - 1][1]} r="4" fill="#0066FF" />
      </svg>
    </Card>
  );
}

function Cell({ value }: { value: ModuleTableRow[string] }) {
  if (value === true) return <Badge tone="active">Enabled</Badge>;
  if (value === false) return <Badge tone="terminated">Disabled</Badge>;
  const s = String(value);
  const statusTone: Record<string, string> = {
    Paid: "active",
    Settled: "active",
    Healthy: "active",
    Active: "active",
    Resolved: "active",
    Processing: "planned",
    Pending: "planned",
    Degraded: "planned",
    Open: "planned",
    Trial: "planned",
    High: "planned",
    Medium: "planned",
    Low: "planned",
  };
  if (statusTone[s]) return <Badge tone={statusTone[s]}>{s}</Badge>;
  return <span className="text-slate-700">{s}</span>;
}

export function DemoModule({ module: moduleProp }: { module?: string }) {
  const { module: moduleParam } = useParams<{ module: string }>();
  const name = moduleProp ?? moduleParam ?? "payroll";
  const meta: ModuleMeta | undefined = MODULE_META[name];
  const [version, setVersion] = useState(0);
  // Bump version to force re-read from the store after a mutation.
  const [notice, setNotice] = useState<string | null>(null);

  if (!meta) {
    return (
      <div>
        <PageHeader title={titleCase(name)} description="Nexora module" />
        <Card className="max-w-2xl p-8">
          <p className="text-sm text-secondary">This module has no demo data configured.</p>
        </Card>
      </div>
    );
  }

  const data = getModuleTable(name);
  void version;

  const onToggle = (index: number, col: string) => {
    toggleModuleRow(name, index, col);
    setVersion((v) => v + 1);
    setNotice(`${titleCase(col)} updated.`);
  };

  const onAction = (action: string) => {
    if (action === "toggle-all") {
      runModuleAction(name, "toggle-all");
      setNotice("All open findings acknowledged (demo).");
    } else {
      const row = seedRowFor(name);
      if (row) {
        addModuleRow(name, row);
        setNotice("New record added (demo — persists for this session).");
      }
    }
    setVersion((v) => v + 1);
  };

  const kpis = meta.kpis(data.rows);

  return (
    <div>
      <PageHeader
        title={meta.title}
        description={meta.description}
        actions={
          meta.actions && (
            <>
              {meta.actions.map((a) => (
                <Button key={a.label} variant={a.type === "toggle-all" ? "secondary" : "primary"} onClick={() => onAction(a.type)}>
                  {a.label}
                </Button>
              ))}
            </>
          )
        }
      />

      {notice && (
        <Alert kind="success" >
          <span onClick={() => setNotice(null)} className="cursor-pointer">{notice} ✕</span>
        </Alert>
      )}

      <div className="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        {kpis.map((k) => (
          <MetricCard key={k.label} label={k.label} value={k.value} hint={k.hint} />
        ))}
      </div>

      {meta.kind === "analytics" && <AnalyticsBars rows={data.rows} />}

      <Card className="mt-6 overflow-hidden">
        <div className="overflow-x-auto" tabIndex={0} role="region" aria-label={`${meta.title} records`}>
          <table className="min-w-full divide-y divide-slate-200 text-sm">
            <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-secondary">
              <tr>
                {data.columns.map((c) => (
                  <th key={c.key} className="px-4 py-2.5 font-medium">
                    {c.label}
                  </th>
                ))}
                {meta.toggleColumn && <th className="px-4 py-2.5 font-medium">Toggle</th>}
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 bg-white">
              {data.rows.map((row, i) => (
                <tr key={i} className="hover:bg-slate-50">
                  {data.columns.map((c) => (
                    <td key={c.key} className="px-4 py-2.5 align-middle">
                      <Cell value={row[c.key]} />
                    </td>
                  ))}
                  {meta.toggleColumn && (
                    <td className="px-4 py-2.5">
                      <button
                        type="button"
                        role="switch"
                        aria-checked={row[meta.toggleColumn] === true}
                        aria-label={`Toggle ${row[data.columns[0]?.key] ?? ""}`}
                        onClick={() => onToggle(i, meta.toggleColumn!)}
                        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 ${
                          row[meta.toggleColumn] === true ? "bg-brand-600" : "bg-slate-300"
                        }`}
                      >
                        <span
                          className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                            row[meta.toggleColumn] === true ? "translate-x-4" : "translate-x-0.5"
                          }`}
                        />
                      </button>
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {data.rows.length === 0 && (
          <p className="px-4 py-3 text-sm text-secondary">No records.</p>
        )}
      </Card>

      <p className="mt-3 text-xs text-secondary">
        Demo data — interactive and persisted for this browser tab. Connect a backend to
        serve live values; nothing here is sent off-device.
      </p>
    </div>
  );
}
