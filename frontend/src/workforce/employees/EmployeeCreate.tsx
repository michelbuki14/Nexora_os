import { useQuery } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { z } from "zod";
import { errorMessage } from "../../api/client";
import {
  createEmployee,
  listDepartments,
  listLocations,
  listPositions,
} from "../../api/workforce";
import type { CreateEmployeeRequest } from "../../api/types";
import { usePermission } from "../../auth/usePermission";
import { PermissionDenied } from "../../components/PermissionDenied";
import { Button, Card, Field, Input, PageHeader, Select } from "../../components/ui";
import { useToast } from "../../components/Toast";

const schema = z.object({
  employee_number: z.string().min(1, "Employee number is required"),
  legal_name: z.string().min(1, "Legal name is required"),
  email: z.string().email("Enter a valid email"),
  preferred_name: z.string().optional(),
  phone: z.string().optional(),
  gender: z.string().optional(),
  date_of_birth: z.string().optional(),
  national_id: z.string().optional(),
  hire_date: z.string().optional(),
  department_ulid: z.string().optional(),
  position_ulid: z.string().optional(),
  location_ulid: z.string().optional(),
  manager_employee_ulid: z.string().optional(),
  employment_type: z.string().optional(),
});

type FormValues = z.infer<typeof schema>;

const EMPLOYMENT_TYPES = [
  { value: "permanent", label: "Permanent" },
  { value: "contract", label: "Contract" },
  { value: "intern", label: "Intern" },
  { value: "consultant", label: "Consultant" },
];

export function EmployeeCreate() {
  const canCreate = usePermission("employee.write");
  const toast = useToast();
  const navigate = useNavigate();
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<FormValues>({ resolver: zodResolver(schema) });

  // Lookup options for org assignments. Failures (missing perms) degrade to
  // empty selects — never blocks the create form.
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

  if (!canCreate) {
    return <PermissionDenied permission="employee.write" />;
  }

  const onSubmit = handleSubmit(async (values: FormValues) => {
    setSubmitting(true);
    try {
      const req: Record<string, unknown> = {
        employee_number: values.employee_number,
        legal_name: values.legal_name,
        email: values.email,
      };
      // Omit empty optionals so the backend defaults apply.
      const optional: (keyof FormValues)[] = [
        "preferred_name",
        "phone",
        "gender",
        "date_of_birth",
        "national_id",
        "hire_date",
        "department_ulid",
        "position_ulid",
        "location_ulid",
        "manager_employee_ulid",
        "employment_type",
      ];
      for (const key of optional) {
        const v = values[key]?.trim();
        if (v) req[key] = v;
      }
      const created = await createEmployee(req as unknown as CreateEmployeeRequest);
      toast.success(`Employee ${created.legal_name} created`);
      navigate(`/workforce/employees/${created.ulid}`);
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  });

  return (
    <div className="mx-auto max-w-2xl">
      <PageHeader title="New employee" description="Create an employee record in this tenant." />
      <Card className="p-6">
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Employee number" htmlFor="employee_number" error={errors.employee_number?.message}>
              <Input id="employee_number" placeholder="EMP-0001" {...register("employee_number")} />
            </Field>
            <Field label="Legal name" htmlFor="legal_name" error={errors.legal_name?.message}>
              <Input id="legal_name" placeholder="Full legal name" {...register("legal_name")} />
            </Field>
            <Field label="Email" htmlFor="email" error={errors.email?.message}>
              <Input id="email" type="email" placeholder="name@org.africa" {...register("email")} />
            </Field>
            <Field label="Preferred name" htmlFor="preferred_name">
              <Input id="preferred_name" {...register("preferred_name")} />
            </Field>
            <Field label="Phone" htmlFor="phone">
              <Input id="phone" {...register("phone")} />
            </Field>
            <Field label="Gender" htmlFor="gender">
              <Input id="gender" {...register("gender")} />
            </Field>
            <Field label="Date of birth" htmlFor="date_of_birth">
              <Input id="date_of_birth" type="date" {...register("date_of_birth")} />
            </Field>
            <Field label="Hire date" htmlFor="hire_date">
              <Input id="hire_date" type="date" {...register("hire_date")} />
            </Field>
            <Field
              label="National ID"
              htmlFor="national_id"
              hint="Hashed on write; only the last 4 digits are ever returned."
            >
              <Input id="national_id" {...register("national_id")} />
            </Field>
            <Field label="Employment type" htmlFor="employment_type">
              <Select id="employment_type" {...register("employment_type")}>
                <option value="">Select…</option>
                {EMPLOYMENT_TYPES.map((o) => (
                  <option key={o.value} value={o.value}>
                    {o.label}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Department" htmlFor="department_ulid">
              <Select id="department_ulid" {...register("department_ulid")}>
                <option value="">Select…</option>
                {departments.data?.map((d) => (
                  <option key={d.ulid} value={d.ulid}>
                    {d.name}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Position" htmlFor="position_ulid">
              <Select id="position_ulid" {...register("position_ulid")}>
                <option value="">Select…</option>
                {positions.data?.map((p) => (
                  <option key={p.ulid} value={p.ulid}>
                    {p.title}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Location" htmlFor="location_ulid">
              <Select id="location_ulid" {...register("location_ulid")}>
                <option value="">Select…</option>
                {locations.data?.map((l) => (
                  <option key={l.ulid} value={l.ulid}>
                    {l.name}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Manager (employee ULID)" htmlFor="manager_employee_ulid">
              <Input id="manager_employee_ulid" placeholder="01HXZ…" {...register("manager_employee_ulid")} />
            </Field>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" type="button" onClick={() => navigate(-1)}>
              Cancel
            </Button>
            <Button type="submit" loading={submitting}>
              Create employee
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}
