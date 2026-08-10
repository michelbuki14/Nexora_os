import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { ColumnDef } from "@tanstack/react-table";
import { listEmployees } from "../../api/workforce";
import type { EmployeeResponse } from "../../api/types";
import { useHasAnyPermission, usePermission } from "../../auth/usePermission";
import { PermissionDenied } from "../../components/PermissionDenied";
import { DataTable } from "../../components/Table";
import { Badge, Button, PageHeader } from "../../components/ui";
import { formatDate, formatUlidShort } from "../../lib/format";

const columns: ColumnDef<EmployeeResponse, unknown>[] = [
  {
    id: "employee_number",
    header: "Employee #",
    cell: (info) => (
      <span className="font-medium text-slate-900">{info.row.original.employee_number}</span>
    ),
  },
  {
    id: "legal_name",
    header: "Name",
    cell: (info) => info.row.original.legal_name,
  },
  {
    id: "email",
    header: "Email",
    cell: (info) => info.row.original.email,
  },
  {
    id: "status",
    header: "Status",
    cell: (info) => <Badge tone={info.row.original.status}>{info.row.original.status}</Badge>,
  },
  {
    id: "hire_date",
    header: "Hire date",
    cell: (info) => formatDate(info.row.original.hire_date),
  },
  {
    id: "manager",
    header: "Manager",
    cell: (info) => (
      <span title={info.row.original.manager_employee_ulid ?? undefined}>
        {formatUlidShort(info.row.original.manager_employee_ulid)}
      </span>
    ),
  },
  {
    id: "actions",
    header: "",
    cell: (info) => (
      <Link
        to={`/workforce/employees/${info.row.original.ulid}`}
        className="text-xs font-medium text-brand-700 hover:underline"
      >
        View profile
      </Link>
    ),
  },
];

export function EmployeeDirectory() {
  const canList = useHasAnyPermission(["employee.read", "employee.export"]);
  const canCreate = usePermission("employee.write");
  const [page, setPage] = useState(1);

  const query = useQuery({
    queryKey: ["workforce", "employees", "list", page],
    queryFn: () => listEmployees(page, 20),
    enabled: canList,
    placeholderData: (prev) => prev,
  });

  if (!canList) {
    return (
      <div>
        <PageHeader title="Employees" description="Employee directory." />
        <PermissionDenied permission="employee.read" />
      </div>
    );
  }

  return (
    <div>
      <PageHeader
        title="Employees"
        description="Directory of employees in this tenant. Manager-scoped lists are enforced server-side."
        actions={
          canCreate ? (
            <Link to="/workforce/employees/new">
              <Button>New employee</Button>
            </Link>
          ) : undefined
        }
      />
      <DataTable
        columns={columns}
        data={query.data?.items ?? []}
        total={query.data?.total ?? 0}
        page={page}
        pageSize={20}
        onPageChange={setPage}
        loading={query.isPending}
        rowKey={(r) => r.ulid}
        emptyTitle="No employees yet"
        emptyDescription="Create an employee to get started."
      />
    </div>
  );
}
