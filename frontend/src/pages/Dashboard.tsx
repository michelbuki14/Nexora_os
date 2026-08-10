import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { listEmployees } from "../api/workforce";
import { useHasAnyPermission } from "../auth/usePermission";
import { Badge, Card, MetricCard, PageHeader } from "../components/ui";
import { WORKFORCE_PERMS } from "../shell/nav";

const PLANNED_TILES = [
  { name: "Payroll", path: "/payroll", blurb: "Payroll computation & statutory reporting" },
  { name: "Finance", path: "/finance", blurb: "Ledger, invoicing & financial reporting" },
  { name: "Payments", path: "/payments", blurb: "Payments orchestration & providers" },
  { name: "Security", path: "/security", blurb: "Access reviews & compliance posture" },
  { name: "Analytics", path: "/analytics", blurb: "Cross-module reporting" },
  { name: "Infrastructure", path: "/infrastructure", blurb: "Service health & capacity" },
];

export function Dashboard() {
  const canReadWorkforce = useHasAnyPermission(["employee.read", "employee.export"]);
  const workforceVisible = useHasAnyPermission(WORKFORCE_PERMS);

  const headcount = useQuery({
    queryKey: ["employees", "dashboard"],
    queryFn: () => listEmployees(1, 1),
    enabled: canReadWorkforce,
  });

  return (
    <div>
      <PageHeader
        title="AOS Overview"
        description="Africa Operating System — a single platform for business, security, and platform operations."
      />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          label="Headcount (live)"
          value={headcount.data?.total ?? "—"}
          hint={headcount.isError ? "Workforce API unavailable" : "Total employees in this tenant"}
        />
        <MetricCard label="Payroll cycles" value="—" hint="Planned" />
        <MetricCard label="Payments volume" value="—" hint="Planned" />
        <MetricCard label="Security posture" value="—" hint="Planned" />
      </div>

      {workforceVisible && (
        <Card className="mt-6 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-sm font-semibold text-slate-900">Workforce</p>
              <p className="text-sm text-slate-500">
                Employees, org structure, compensation and documents — live now.
              </p>
            </div>
            <Link
              to="/workforce"
              className="rounded-md bg-brand-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-brand-700"
            >
              Open Workforce
            </Link>
          </div>
        </Card>
      )}

      {!canReadWorkforce && (
        <Card className="mt-6 p-4">
          <p className="text-sm text-slate-600">
            Your account does not hold workforce read permissions in this tenant.
            Contact your tenant administrator to request the appropriate role.{" "}
            {!workforceVisible &&
              "Workforce navigation is hidden until a workforce permission is granted."}
          </p>
        </Card>
      )}

      <h2 className="mb-3 mt-8 text-sm font-semibold uppercase tracking-wide text-slate-400">
        Modules in planning
      </h2>
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {PLANNED_TILES.map((tile) => (
          <Link key={tile.name} to={tile.path}>
            <Card className="h-full p-4 transition-shadow hover:shadow-md">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium text-slate-900">{tile.name}</p>
                <Badge tone="planned">Planned</Badge>
              </div>
              <p className="mt-1 text-xs text-slate-500">{tile.blurb}</p>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  );
}
