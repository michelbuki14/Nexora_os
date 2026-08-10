import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useParams } from "react-router-dom";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { ApiError, errorMessage } from "../../api/client";
import {
  getEmployee,
  listDepartments,
  listLocations,
  listPositions,
  updateEmployeeStatus,
  updateEmployment,
} from "../../api/workforce";
import type { EmployeeResponse, EmployeeStatus } from "../../api/types";
import { usePermission } from "../../auth/usePermission";
import { ConfirmDialog, Modal } from "../../components/Modal";
import { PermissionDenied } from "../../components/PermissionDenied";
import { Tabs } from "../../components/Tabs";
import { useToast } from "../../components/Toast";
import { Alert, Avatar, Badge, Button, Card, Field, Input, PageHeader, Select, Skeleton } from "../../components/ui";
import { formatDate, formatDateTime, formatUlidShort } from "../../lib/format";
import { CompensationPanel } from "./CompensationPanel";
import { DocumentsPanel } from "./DocumentsPanel";

export function EmployeeProfile() {
  const { ulid = "" } = useParams();
  const [tab, setTab] = useState("overview");

  const query = useQuery({
    queryKey: ["workforce", "employees", ulid],
    queryFn: () => getEmployee(ulid),
  });

  if (query.isPending) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  if (query.isError) {
    if (query.error instanceof ApiError && query.error.status === 403) {
      return <PermissionDenied permission="employee.read" />;
    }
    return <Alert kind="danger">Could not load employee: {errorMessage(query.error)}</Alert>;
  }

  const employee = query.data;

  return (
    <div>
      <PageHeader
        title={employee.legal_name}
        description={`${employee.employee_number} · ${employee.email}`}
        actions={<StatusControl employee={employee} />}
      />
      <Card className="mb-6 p-4">
        <div className="flex flex-wrap items-center gap-4">
          <Avatar name={employee.legal_name} className="h-12 w-12 text-base" />
          <div>
            <p className="text-sm font-medium text-slate-900">
              {employee.preferred_name ?? employee.legal_name}
            </p>
            <p className="text-xs text-slate-500">
              Hired {formatDate(employee.hire_date)} · updated {formatDateTime(employee.updated_at)}
            </p>
          </div>
          <div className="ml-auto">
            <Badge tone={employee.status}>{employee.status}</Badge>
          </div>
        </div>
      </Card>

      <Tabs
        active={tab}
        onChange={setTab}
        tabs={[
          { key: "overview", label: "Overview", content: <OverviewTab employee={employee} /> },
          { key: "employment", label: "Employment", content: <EmploymentTab employee={employee} /> },
          { key: "compensation", label: "Compensation", content: <CompensationPanel employeeUlid={ulid} /> },
          { key: "documents", label: "Documents", content: <DocumentsPanel employeeUlid={ulid} /> },
        ]}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------

function OverviewTab({ employee }: { employee: EmployeeResponse }) {
  const rows: [string, string][] = [
    ["Employee number", employee.employee_number],
    ["Email", employee.email],
    ["Phone", employee.phone ?? "—"],
    ["Preferred name", employee.preferred_name ?? "—"],
    ["Gender", employee.gender ?? "—"],
    ["Hire date", formatDate(employee.hire_date)],
    ["National ID (last 4)", employee.national_id_last4 ?? "—"],
    ["Manager", formatUlidShort(employee.manager_employee_ulid)],
    ["Created", formatDateTime(employee.created_at)],
  ];
  return (
    <Card className="p-4">
      <dl className="grid gap-x-8 gap-y-3 sm:grid-cols-2">
        {rows.map(([label, value]) => (
          <div key={label} className="flex items-baseline justify-between gap-4">
            <dt className="text-xs font-medium uppercase tracking-wide text-slate-400">{label}</dt>
            <dd className="text-sm text-slate-800">{value}</dd>
          </div>
        ))}
      </dl>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Status lifecycle
// ---------------------------------------------------------------------------

const STATUS_FLOWS: { from: EmployeeStatus; to: EmployeeStatus; label: string; terminal?: boolean }[] = [
  { from: "onboarding", to: "active", label: "Activate" },
  { from: "active", to: "on_leave", label: "Put on leave" },
  { from: "on_leave", to: "active", label: "Return from leave" },
  { from: "active", to: "terminated", label: "Terminate", terminal: true },
  { from: "onboarding", to: "terminated", label: "Terminate", terminal: true },
];

function StatusControl({ employee }: { employee: EmployeeResponse }) {
  const toast = useToast();
  const queryClient = useQueryClient();
  const [target, setTarget] = useState<(typeof STATUS_FLOWS)[number] | null>(null);
  const canWrite = usePermission("employee.write");
  const canTerminate = usePermission("employee.terminate");

  const mutation = useMutation({
    mutationFn: (status: EmployeeStatus) =>
      updateEmployeeStatus(employee.ulid, { status }),
    onSuccess: () => {
      toast.success("Employee status updated");
      queryClient.invalidateQueries({ queryKey: ["workforce", "employees"] });
    },
    onError: (err) => toast.error(errorMessage(err)),
  });

  const flow = STATUS_FLOWS.find((f) => f.from === employee.status);
  if (!flow) return null;

  // Termination requires employee.terminate; other transitions employee.write.
  const permitted = flow.terminal ? canTerminate : canWrite;
  if (!permitted) return null;

  return (
    <>
      <Button variant={flow.terminal ? "danger" : "secondary"} onClick={() => setTarget(flow)}>
        {flow.label}
      </Button>
      <ConfirmDialog
        open={target !== null}
        title={flow.label}
        danger={flow.terminal}
        confirmLabel={flow.label}
        loading={mutation.isPending}
        body={
          flow.terminal ? (
            <>
              Terminating <strong>{employee.legal_name}</strong> is final and is recorded in the
              audit trail. This requires the <code>employee.terminate</code> permission.
            </>
          ) : (
            <>Change <strong>{employee.legal_name}</strong> from <code>{employee.status}</code> to <code>{flow.to}</code>.</>
          )
        }
        onConfirm={() => {
          if (target) mutation.mutate(target.to);
          setTarget(null);
        }}
        onCancel={() => setTarget(null)}
      />
    </>
  );
}

// ---------------------------------------------------------------------------
// Employment
// ---------------------------------------------------------------------------

const employmentSchema = z.object({
  department_ulid: z.string().optional(),
  position_ulid: z.string().optional(),
  location_ulid: z.string().optional(),
  employment_type: z.string().optional(),
  effective_date: z.string().min(1, "Effective date is required"),
  change_reason: z.string().optional(),
});
type EmploymentValues = z.infer<typeof employmentSchema>;

function EmploymentTab({ employee }: { employee: EmployeeResponse }) {
  const canEdit = usePermission("employee.write");
  const [editOpen, setEditOpen] = useState(false);
  const toast = useToast();
  const queryClient = useQueryClient();

  const departments = useQuery({
    queryKey: ["workforce", "departments", "list", 1],
    queryFn: async () => (await listDepartments(1, 100)).items,
    retry: 0,
  });
  const positions = useQuery({
    queryKey: ["workforce", "positions", "list", 1],
    queryFn: async () => (await listPositions(1, 100)).items,
    retry: 0,
  });
  const locations = useQuery({
    queryKey: ["workforce", "locations", "list", 1],
    queryFn: async () => (await listLocations(1, 100)).items,
    retry: 0,
  });

  const mutation = useMutation({
    mutationFn: (values: EmploymentValues) =>
      updateEmployment(employee.ulid, {
        department_ulid: values.department_ulid?.trim() || null,
        position_ulid: values.position_ulid?.trim() || null,
        location_ulid: values.location_ulid?.trim() || null,
        employment_type: (values.employment_type?.trim() || null) as never,
        effective_date: values.effective_date,
        change_reason: values.change_reason?.trim() || null,
      }),
    onSuccess: () => {
      toast.success("Employment record updated");
      setEditOpen(false);
      queryClient.invalidateQueries({ queryKey: ["workforce", "employees"] });
    },
    onError: (err) => toast.error(errorMessage(err)),
  });

  const { register, handleSubmit, reset } = useForm<EmploymentValues>({
    resolver: zodResolver(employmentSchema),
    defaultValues: {
      department_ulid: employee.current_department_ulid ?? "",
      position_ulid: employee.current_position_ulid ?? "",
      location_ulid: employee.current_location_ulid ?? "",
      effective_date: new Date().toISOString().slice(0, 10),
    },
  });

  const rows: [string, string][] = [
    ["Department", formatUlidShort(employee.current_department_ulid)],
    ["Position", formatUlidShort(employee.current_position_ulid)],
    ["Location", formatUlidShort(employee.current_location_ulid)],
  ];

  return (
    <Card className="p-4">
      <div className="mb-3 flex items-center justify-between">
        <p className="text-sm font-medium text-slate-900">Current assignments</p>
        {canEdit && (
          <Button variant="secondary" size="sm" onClick={() => { reset(); setEditOpen(true); }}>
            Edit employment
          </Button>
        )}
      </div>
      <dl className="grid gap-x-8 gap-y-3 sm:grid-cols-3">
        {rows.map(([label, value]) => (
          <div key={label}>
            <dt className="text-xs font-medium uppercase tracking-wide text-slate-400">{label}</dt>
            <dd className="text-sm text-slate-800">{value}</dd>
          </div>
        ))}
      </dl>
      <p className="mt-3 text-xs text-slate-400">
        Employment history is a tracked follow-up; Phase 1 shows current assignments.
      </p>

      <Modal open={editOpen} title="Edit employment" onClose={() => setEditOpen(false)}>
        <form
          onSubmit={handleSubmit((v) => mutation.mutate(v))}
          className="space-y-3"
        >
          <Field label="Department" htmlFor="edit-department">
            <Select id="edit-department" {...register("department_ulid")}>
              <option value="">Unassigned</option>
              {departments.data?.map((d) => (
                <option key={d.ulid} value={d.ulid}>{d.name}</option>
              ))}
            </Select>
          </Field>
          <Field label="Position" htmlFor="edit-position">
            <Select id="edit-position" {...register("position_ulid")}>
              <option value="">Unassigned</option>
              {positions.data?.map((p) => (
                <option key={p.ulid} value={p.ulid}>{p.title}</option>
              ))}
            </Select>
          </Field>
          <Field label="Location" htmlFor="edit-location">
            <Select id="edit-location" {...register("location_ulid")}>
              <option value="">Unassigned</option>
              {locations.data?.map((l) => (
                <option key={l.ulid} value={l.ulid}>{l.name}</option>
              ))}
            </Select>
          </Field>
          <Field label="Employment type" htmlFor="edit-type">
            <Select id="edit-type" {...register("employment_type")}>
              <option value="">Unassigned</option>
              {["permanent", "contract", "intern", "consultant"].map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </Select>
          </Field>
          <Field label="Effective date" htmlFor="edit-effective">
            <Input id="edit-effective" type="date" required {...register("effective_date")} />
          </Field>
          <Field label="Change reason" htmlFor="edit-reason">
            <Input id="edit-reason" placeholder="e.g. promotion" {...register("change_reason")} />
          </Field>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" type="button" onClick={() => setEditOpen(false)}>
              Cancel
            </Button>
            <Button type="submit" loading={mutation.isPending}>Save changes</Button>
          </div>
        </form>
      </Modal>
    </Card>
  );
}
