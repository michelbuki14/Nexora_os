/** Date/ULID display helpers. Backend emits RFC3339 timestamps and NaiveDates. */

/** "2026-08-10" for a date/date-time string; "—" for null/invalid. */
export function formatDate(value: string | null | undefined): string {
  if (!value) return "—";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toISOString().slice(0, 10);
}

/** Local, human-readable date + time. */
export function formatDateTime(value: string | null | undefined): string {
  if (!value) return "—";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

/** Short display form of a ULID; keep the full value in a title attribute. */
export function formatUlidShort(ulid: string | null | undefined): string {
  if (!ulid) return "—";
  return ulid.length > 10 ? `${ulid.slice(0, 10)}…` : ulid;
}
