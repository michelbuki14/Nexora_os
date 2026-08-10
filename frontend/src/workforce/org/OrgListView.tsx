import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ReactNode, useState } from "react";
import { useForm } from "react-hook-form";
import { useNavigate, useParams } from "react-router-dom";
import { ColumnDef } from "@tanstack/react-table";
import { errorMessage } from "../../api/client";
import {
  createDepartment,
  createLegalEntity,
  createLocation,
  createPosition,
  createTeam,
  listDepartments,
  listLegalEntities,
  listLocations,
  listPositions,
  listTeams,
} from "../../api/workforce";
import type {
  DepartmentResponse,
  LegalEntityResponse,
  LocationResponse,
  PositionResponse,
  TeamResponse,
} from "../../api/types";
import { usePermission } from "../../auth/usePermission";
import { Button, PageHeader } from "../../components/ui";
import { Modal } from "../../components/Modal";
import { DataTable } from "../../components/Table";
import { useToast } from "../../components/Toast";
import { Badge, Field, Input, Select } from "../../components/ui";
import { formatDate } from "../../lib/format";

type Row = { ulid: string; name: string; created_at: string };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface ColumnSpec<T = any> {
  header: string;
  cell: (row: T) => ReactNode;
}

interface FieldDef {
  name: string;
  label: string;
  type: "text" | "select";
  required?: boolean;
  options?: { value: string; label: string }[];
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface ResourceDef<T = any> {
  label: string;
  readPerm: string;
  writePerm: string;
  list: (page: number, pageSize: number) => Promise<{ items: T[]; total: number }>;
  create: (req: Record<string, unknown>) => Promise<unknown>;
  columns: ColumnSpec<T>[];
  fields: FieldDef[];
}

const EMPLOYMENT_TYPES = [
  { value: "permanent", label: "Permanent" },
  { value: "contract", label: "Contract" },
  { value: "intern", label: "Intern" },
  { value: "consultant", label: "Consultant" },
];

function activeBadge(active: boolean) {
  return active ? <Badge tone="active">Active</Badge> : <Badge tone="terminated">Inactive</Badge>;
}

const RESOURCES: Record<
  string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ResourceDef<any>
> = {
  departments: {
    label: "Departments",
    readPerm: "department.read",
    writePerm: "department.write",
    list: listDepartments as never,
    create: createDepartment as never,
    columns: [
      { header: "Name", cell: (r) => (r as DepartmentResponse).name },
      {
        header: "Code",
        cell: (r) => (r as DepartmentResponse).code ?? "—",
      },
      {
        header: "Status",
        cell: (r) => activeBadge((r as DepartmentResponse).is_active),
      },
      { header: "Created", cell: (r) => formatDate((r as DepartmentResponse).created_at) },
    ],
    fields: [
      { name: "name", label: "Name", type: "text", required: true },
      { name: "code", label: "Code", type: "text" },
      { name: "parent_department_ulid", label: "Parent department ULID", type: "text" },
      { name: "location_ulid", label: "Location ULID", type: "text" },
    ],
  },
  positions: {
    label: "Positions",
    readPerm: "position.read",
    writePerm: "position.write",
    list: listPositions as never,
    create: createPosition as never,
    columns: [
      { header: "Title", cell: (r) => (r as PositionResponse).title },
      { header: "Code", cell: (r) => (r as PositionResponse).code ?? "—" },
      { header: "Grade", cell: (r) => (r as PositionResponse).job_grade ?? "—" },
      { header: "Type", cell: (r) => (r as PositionResponse).employment_type },
      { header: "Status", cell: (r) => activeBadge((r as PositionResponse).is_active) },
      { header: "Created", cell: (r) => formatDate((r as PositionResponse).created_at) },
    ],
    fields: [
      { name: "title", label: "Title", type: "text", required: true },
      { name: "code", label: "Code", type: "text" },
      { name: "department_ulid", label: "Department ULID", type: "text" },
      { name: "job_grade", label: "Job grade", type: "text" },
      { name: "employment_type", label: "Employment type", type: "select", options: EMPLOYMENT_TYPES },
      { name: "description", label: "Description", type: "text" },
      { name: "responsibilities", label: "Responsibilities", type: "text" },
    ],
  },
  locations: {
    label: "Locations",
    readPerm: "location.read",
    writePerm: "location.write",
    list: listLocations as never,
    create: createLocation as never,
    columns: [
      { header: "Name", cell: (r) => (r as LocationResponse).name },
      { header: "Code", cell: (r) => (r as LocationResponse).code ?? "—" },
      { header: "Timezone", cell: (r) => (r as LocationResponse).timezone },
      { header: "Status", cell: (r) => activeBadge((r as LocationResponse).is_active) },
      { header: "Created", cell: (r) => formatDate((r as LocationResponse).created_at) },
    ],
    fields: [
      { name: "legal_entity_ulid", label: "Legal entity ULID", type: "text", required: true },
      { name: "name", label: "Name", type: "text", required: true },
      { name: "code", label: "Code", type: "text" },
      { name: "timezone", label: "Timezone", type: "text" },
    ],
  },
  teams: {
    label: "Teams",
    readPerm: "team.read",
    writePerm: "team.write",
    list: listTeams as never,
    create: createTeam as never,
    columns: [
      { header: "Name", cell: (r) => (r as TeamResponse).name },
      { header: "Code", cell: (r) => (r as TeamResponse).code ?? "—" },
      { header: "Status", cell: (r) => activeBadge((r as TeamResponse).is_active) },
      { header: "Created", cell: (r) => formatDate((r as TeamResponse).created_at) },
    ],
    fields: [
      { name: "department_ulid", label: "Department ULID", type: "text", required: true },
      { name: "name", label: "Name", type: "text", required: true },
      { name: "code", label: "Code", type: "text" },
    ],
  },
  "legal-entities": {
    label: "Legal Entities",
    readPerm: "legal_entity.read",
    writePerm: "legal_entity.write",
    list: listLegalEntities as never,
    create: createLegalEntity as never,
    columns: [
      { header: "Name", cell: (r) => (r as LegalEntityResponse).name },
      { header: "Registration", cell: (r) => (r as LegalEntityResponse).registration_number ?? "—" },
      {
        header: "Status",
        cell: (r) => (
          <Badge tone={(r as LegalEntityResponse).status}>
            {(r as LegalEntityResponse).status}
          </Badge>
        ),
      },
      { header: "Created", cell: (r) => formatDate((r as LegalEntityResponse).created_at) },
    ],
    fields: [
      { name: "name", label: "Name", type: "text", required: true },
      { name: "registration_number", label: "Registration number", type: "text" },
      { name: "country_ulid", label: "Country ULID", type: "text" },
    ],
  },
};

function toTanStack<T extends Row>(specs: ColumnSpec<T>[]): ColumnDef<T, unknown>[] {
  return specs.map((s, i) => ({
    id: String(i),
    header: s.header,
    cell: (info) => s.cell(info.row.original),
  }));
}

export function OrgListView() {
  const { resource = "" } = useParams();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [page, setPage] = useState(1);
  const [createOpen, setCreateOpen] = useState(false);
  const [creating, setCreating] = useState(false);

  const def = RESOURCES[resource];

  const canRead = usePermission(def?.readPerm);
  const canWrite = usePermission(def?.writePerm);

  const { register, handleSubmit, reset } = useForm<Record<string, string>>();

  const list = useQuery({
    queryKey: ["workforce", resource, "list", page],
    queryFn: () => def.list(page, 20),
    enabled: !!def && canRead,
    placeholderData: (prev) => prev,
  });

  if (!def) {
    navigate("/404", { replace: true });
    return null;
  }

  const onSubmit = handleSubmit(async (values) => {
    setCreating(true);
    try {
      const payload: Record<string, unknown> = {};
      for (const field of def.fields) {
        const v = values[field.name]?.trim();
        if (v) payload[field.name] = v;
      }
      await def.create(payload);
      toast.success(`${def.label.slice(0, -1)} created`);
      setCreateOpen(false);
      reset();
      queryClient.invalidateQueries({ queryKey: ["workforce", resource] });
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setCreating(false);
    }
  });

  return (
    <div>
      <PageHeader
        title={def.label}
        description={`Manage ${def.label.toLowerCase()} in this tenant.`}
        actions={
          canWrite ? (
            <Button onClick={() => setCreateOpen(true)}>Create {def.label.slice(0, -1)}</Button>
          ) : undefined
        }
      />

      {!canRead ? (
        <p className="text-sm text-slate-500">
          You need the <code className="rounded bg-slate-100 px-1">{def.readPerm}</code>{" "}
          permission to view this section.
        </p>
      ) : (
        <DataTable
          columns={toTanStack(def.columns)}
          data={list.data?.items ?? []}
          total={list.data?.total ?? 0}
          page={page}
          pageSize={20}
          onPageChange={setPage}
          loading={list.isPending}
          rowKey={(row) => row.ulid}
          emptyTitle={`No ${def.label.toLowerCase()} yet`}
          emptyDescription="Create one to get started."
        />
      )}

      <Modal open={createOpen} title={`Create ${def.label.slice(0, -1)}`} onClose={() => setCreateOpen(false)}>
        <form onSubmit={onSubmit} className="space-y-3">
          {def.fields.map((field) => (
            <Field key={field.name} label={field.label} htmlFor={field.name}>
              {field.type === "select" ? (
                <Select id={field.name} {...register(field.name)}>
                  <option value="">Select…</option>
                  {field.options?.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </Select>
              ) : (
                <Input id={field.name} required={field.required} {...register(field.name)} />
              )}
            </Field>
          ))}
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" type="button" onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button type="submit" loading={creating}>
              Create
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
