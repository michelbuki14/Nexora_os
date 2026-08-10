import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { errorMessage, ApiError } from "../../api/client";
import { createCompensation, getCompensation } from "../../api/workforce";
import type { CompensationFrequency } from "../../api/types";
import { usePermission } from "../../auth/usePermission";
import { ConfirmDialog, Modal } from "../../components/Modal";
import { useToast } from "../../components/Toast";
import { Button, Card, EmptyState, Field, Input, Select } from "../../components/ui";
import { formatDate } from "../../lib/format";
import { formatMoney, minorToDecimal, parseMoneyToMinor } from "../../lib/money";

const CURRENCIES = ["XAF", "XOF", "KES", "NGN", "ZAR", "GHS", "MAD", "EGP", "TZS", "USD", "EUR"];
const FREQUENCIES: { value: CompensationFrequency; label: string }[] = [
  { value: "monthly", label: "Monthly" },
  { value: "annual", label: "Annual" },
  { value: "hourly", label: "Hourly" },
  { value: "weekly", label: "Weekly" },
];

const schema = z.object({
  amount: z.string().min(1, "Enter an amount"),
  currency_code: z.string().min(1, "Currency is required"),
  frequency: z.string().min(1, "Frequency is required"),
  effective_date: z.string().min(1, "Effective date is required"),
  change_reason: z.string().optional(),
});
type FormValues = z.infer<typeof schema>;

/**
 * Compensation is a separate, permission-gated call. This panel never fetches
 * compensation unless the caller holds employee.compensation.read — no
 * "fetch and hide". Changes emit a salary.changed audit event (amount omitted).
 */
export function CompensationPanel({ employeeUlid }: { employeeUlid: string }) {
  const canRead = usePermission("employee.compensation.read");
  const canWrite = usePermission("employee.compensation.write");
  const toast = useToast();
  const queryClient = useQueryClient();
  const [formOpen, setFormOpen] = useState(false);
  const [pending, setPending] = useState<FormValues | null>(null);

  const query = useQuery({
    queryKey: ["workforce", "employees", employeeUlid, "compensation"],
    queryFn: () => getCompensation(employeeUlid),
    enabled: canRead,
  });

  const mutation = useMutation({
    mutationFn: (values: FormValues) => {
      const minor = parseMoneyToMinor(values.amount);
      if (minor === null) throw new Error("Invalid amount");
      return createCompensation(employeeUlid, {
        gross_amount_minor: minor,
        currency_code: values.currency_code,
        frequency: values.frequency as CompensationFrequency,
        effective_date: values.effective_date,
        change_reason: values.change_reason?.trim() || null,
      });
    },
    onSuccess: () => {
      toast.success("Compensation change recorded (salary.changed audit event)");
      setPending(null);
      setFormOpen(false);
      queryClient.invalidateQueries({ queryKey: ["workforce", "employees", employeeUlid] });
    },
    onError: (err) => toast.error(errorMessage(err)),
  });

  const { register, handleSubmit, reset } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      currency_code: "XAF",
      frequency: "monthly",
      effective_date: new Date().toISOString().slice(0, 10),
    },
  });

  if (!canRead) {
    return (
      <Card className="p-4">
        <p className="text-sm text-slate-500">
          Compensation is restricted. You need the{" "}
          <code className="rounded bg-slate-100 px-1">employee.compensation.read</code>{" "}
          permission to view salary details.
        </p>
      </Card>
    );
  }

  const openForm = () => {
    reset();
    setFormOpen(true);
  };

  return (
    <Card className="p-4">
      <div className="mb-3 flex items-center justify-between">
        <p className="text-sm font-medium text-slate-900">Current compensation</p>
        {canWrite && (
          <Button variant="secondary" size="sm" onClick={openForm}>
            Set compensation
          </Button>
        )}
      </div>

      {query.isPending && <p className="text-sm text-slate-400">Loading…</p>}
      {query.isError && !(query.error instanceof ApiError && query.error.status === 404) && (
        <p className="text-sm text-red-600">Could not load compensation: {errorMessage(query.error)}</p>
      )}
      {query.data ? (
        <dl className="grid gap-x-8 gap-y-3 sm:grid-cols-3">
          <div>
            <dt className="text-xs font-medium uppercase tracking-wide text-slate-400">Gross amount</dt>
            <dd className="text-sm font-semibold text-slate-900">
              {formatMoney(query.data.gross_amount_minor, query.data.currency_code)}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase tracking-wide text-slate-400">Frequency</dt>
            <dd className="text-sm text-slate-800">{query.data.frequency}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase tracking-wide text-slate-400">Effective</dt>
            <dd className="text-sm text-slate-800">{formatDate(query.data.effective_date)}</dd>
          </div>
        </dl>
      ) : (
        !query.isPending && (
          <EmptyState
            title="No compensation on record"
            description={canWrite ? "Set the first compensation for this employee." : "No compensation has been recorded."}
          />
        )
      )}

      <Modal open={formOpen} title="Set compensation" onClose={() => setFormOpen(false)}>
        <form
          onSubmit={handleSubmit((v) => setPending(v))}
          className="space-y-3"
        >
          <Field label="Gross amount" htmlFor="comp-amount">
            <Input id="comp-amount" placeholder="0.00" inputMode="decimal" {...register("amount")} />
          </Field>
          <div className="grid grid-cols-2 gap-3">
            <Field label="Currency" htmlFor="comp-currency">
              <Select id="comp-currency" {...register("currency_code")}>
                {CURRENCIES.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </Select>
            </Field>
            <Field label="Frequency" htmlFor="comp-frequency">
              <Select id="comp-frequency" {...register("frequency")}>
                {FREQUENCIES.map((f) => (
                  <option key={f.value} value={f.value}>{f.label}</option>
                ))}
              </Select>
            </Field>
          </div>
          <Field label="Effective date" htmlFor="comp-effective">
            <Input id="comp-effective" type="date" required {...register("effective_date")} />
          </Field>
          <Field label="Change reason" htmlFor="comp-reason">
            <Input id="comp-reason" placeholder="e.g. annual review" {...register("change_reason")} />
          </Field>
          <p className="text-xs text-slate-400">
            The change is written to the immutable audit trail as{" "}
            <code>salary.changed</code>; the amount itself is omitted from the
            audit event.
          </p>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" type="button" onClick={() => setFormOpen(false)}>
              Cancel
            </Button>
            <Button type="submit">Review</Button>
          </div>
        </form>
      </Modal>

      <ConfirmDialog
        open={pending !== null}
        title="Confirm compensation change"
        confirmLabel="Confirm change"
        loading={mutation.isPending}
        body={
          pending ? (
            <>
              Set gross compensation to{" "}
              <strong>
                {minorToDecimal(parseMoneyToMinor(pending.amount) ?? 0)} {pending.currency_code}
              </strong>{" "}
              ({pending.frequency}), effective {pending.effective_date}? This is recorded in the
              audit trail.
            </>
          ) : null
        }
        onConfirm={() => pending && mutation.mutate(pending)}
        onCancel={() => setPending(null)}
      />
    </Card>
  );
}
