import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import {
  listDepartments,
  listEmployees,
  listLegalEntities,
  listLocations,
  listPositions,
  listTeams,
} from "../api/workforce";
import { useHasAnyPermission, usePermission } from "../auth/usePermission";
import { Card, PageHeader } from "../components/ui";
import { WORKFORCE_PERMS } from "../shell/nav";

interface Section {
  label: string;
  to: string;
  description: string;
  readPerm: string;
  queryKey: string;
  countQuery: () => Promise<{ total: number }>;
}

const SECTIONS: Section[] = [
  {
    label: "Employees",
    to: "/workforce/employees",
    description: "Directory, profiles, compensation and documents",
    readPerm: "employee.read",
    queryKey: "employees",
    countQuery: () => listEmployees(1, 1),
  },
  {
    label: "Departments",
    to: "/workforce/departments",
    description: "Org structure",
    readPerm: "department.read",
    queryKey: "departments",
    countQuery: () => listDepartments(1, 1),
  },
  {
    label: "Positions",
    to: "/workforce/positions",
    description: "Roles and grades",
    readPerm: "position.read",
    queryKey: "positions",
    countQuery: () => listPositions(1, 1),
  },
  {
    label: "Locations",
    to: "/workforce/locations",
    description: "Sites and timezones",
    readPerm: "location.read",
    queryKey: "locations",
    countQuery: () => listLocations(1, 1),
  },
  {
    label: "Teams",
    to: "/workforce/teams",
    description: "Cross-functional teams",
    readPerm: "team.read",
    queryKey: "teams",
    countQuery: () => listTeams(1, 1),
  },
  {
    label: "Legal Entities",
    to: "/workforce/legal-entities",
    description: "Registered entities",
    readPerm: "legal_entity.read",
    queryKey: "legal-entities",
    countQuery: () => listLegalEntities(1, 1),
  },
];

export function WorkforceOverview() {
  if (!useHasAnyPermission(WORKFORCE_PERMS)) {
    return (
      <div>
        <PageHeader title="Workforce" description="Employees and organization structure." />
        <Card className="max-w-xl p-6">
          <p className="text-sm text-slate-600">
            Your account does not hold workforce permissions in this tenant. Request the
            appropriate role from your tenant administrator to access employees, org
            structure, compensation, and documents.
          </p>
        </Card>
      </div>
    );
  }

  return (
    <div>
      <PageHeader
        title="Workforce"
        description="Employees, organization structure, compensation and documents — live from the AOS workforce service."
      />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {SECTIONS.map((section) => (
          <SectionCard key={section.to} section={section} />
        ))}
      </div>
    </div>
  );
}

function SectionCard({ section }: { section: Section }) {
  const allowed = usePermission(section.readPerm);
  const count = useQuery({
    queryKey: ["workforce", section.queryKey, "count"],
    queryFn: section.countQuery,
    enabled: allowed,
  });

  if (!allowed) {
    return (
      <Card className="p-4 opacity-70">
        <p className="text-sm font-medium text-slate-900">{section.label}</p>
        <p className="mt-1 text-xs text-slate-500">{section.description}</p>
        <p className="mt-2 text-xs text-slate-400">
          Requires {section.readPerm}
        </p>
      </Card>
    );
  }

  return (
    <Link to={section.to}>
      <Card className="h-full p-4 transition-shadow hover:shadow-md">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium text-slate-900">{section.label}</p>
          <span className="text-2xl font-semibold text-brand-700">
            {count.data?.total ?? "—"}
          </span>
        </div>
        <p className="mt-1 text-xs text-slate-500">{section.description}</p>
        <p className="mt-2 text-xs font-medium text-brand-700">Open →</p>
      </Card>
    </Link>
  );
}
