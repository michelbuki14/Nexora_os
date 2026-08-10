# Workforce Portal Map

> AOS has **no frontend crate today** (verified — no `package.json`, no design
> system). The Workforce prompt's "My-AOS / Manager / HR-Admin portals
> integrated into the existing AOS design system" cannot be satisfied literally
> in Phase 1. This doc is the **forward contract**: it sketches the three
> portals against the Phase-1 endpoints each calls, so when a frontend crate
> (likely React/TS, TBD) is created it can be built straight from this map and
> `data-contract.md`.
>
> Backend-only Phase 1 = the OpenAPI spec + this map. No UI is built.

---

## Shared shell (all three portals)

- Auth: Keycloak OIDC bearer token in `Authorization: Bearer …`; same identity,
  organizations, permissions as the rest of AOS.
- Error surface: `ErrorResponse { code, message, details? }` (shared `AosError`).
- Pagination: `?page=&page_size=` → `{ items, total, page, page_size }`.
- Tenant/org: implicit from the token (RLS); never sent in the request body.
- Compensation redaction: the employee list/detail responses contain **no**
  compensation fields — a separate gated call is required.

---

## Portal 1 — My-AOS (the employee self-service portal)

**Role:** EMPLOYEE. Sees only their own record.

| View | Endpoint(s) called | Notes |
|------|--------------------|-------|
| My profile | `GET /employees/{my_ulid}` | Self-view path in `check_employee_access` passes without `employee.read`. Shows `national_id_last4`, not the full ID. |
| My employment history | (Phase 3 — no list-employment endpoint yet; current `current_*` fields from the profile) | `wf_employment_records` is append-only but not yet exposed as a list. Map notes the gap. |
| My compensation | `GET /employees/{my_ulid}/compensation` | Gated on `employee.compensation.read` — **EMPLOYEE does not hold it** in the Phase-1 grant matrix, so this tile must be hidden/disabled for the EMPLOYEE role. Visible only to PAYROLL_ADMIN/FINANCE_ADMIN/HR_ADMIN/AUDITOR viewing the employee. |
| My documents | `GET /employees/{my_ulid}/documents` | Self-view branch (EMPLOYEE may list own docs). |
| Download a document | `GET /employees/{my_ulid}/documents/{doc_ulid}/url` → then `GET {presigned_url}` | EMPLOYEE self-view passes the IDOR check. Presigned URL TTL 60s. |
| My status | from the profile `status` field | Read-only for EMPLOYEE; lifecycle transitions are HR-only. |

**Design note:** because the EMPLOYEE role lacks `employee.compensation.read`,
the portal's "My compensation" tile renders only for roles that hold it. The
backend never returns compensation in the profile response, so the frontend is
**structurally unable** to leak it — exactly matching the "never rely solely on
frontend filtering" rule (defense is at the API, the frontend hide is UX only).

---

## Portal 2 — Manager portal

**Role:** MANAGER. Sees direct reports.

| View | Endpoint(s) | Notes |
|------|-------------|-------|
| My direct reports | `GET /employees` (manager scope) | The list handler auto-scopes to `manager_employee_id = <self>` when the caller has `employee.read` but not `employee.write` and is a MANAGER. `ponytail:` full org-chart traversal deferred — direct reports only in MVP. |
| A report's profile | `GET /employees/{report_ulid}` | `check_employee_access` passes because `row.manager_employee_id == self.id`. |
| A report's status update | `PATCH /employees/{report_ulid}/status` (non-terminal transitions) | MANAGER holds `employee.write`? In the Phase-1 matrix, MANAGER has **read** only. Terminal → needs `employee.terminate`, which MANAGER lacks. So a MANAGER **cannot** change status in Phase 1 — the tile is disabled. HR approves. |
| A report's documents | `GET /employees/{report_ulid}/documents` | `employee.documents.read` — MANAGER gets `(reports)`. |
| A report's compensation | `GET .../compensation` | **Not** granted to MANAGER — tile hidden. |

**Design note:** the manager portal is a read-mostly surface in Phase 1. The
manager can surface a report's profile and documents; compensation and status
changes route to HR. This is enforced by missing permissions, not by UI hiding.

---

## Portal 3 — HR-Admin portal

**Role:** HR_ADMIN (or ORG_ADMIN/SUPER_ADMIN). Full tenant scope.

| View / action | Endpoint(s) | Permission |
|---------------|-------------|------------|
| Org structure: legal entities | `GET/POST /legal-entities` | `legal_entity.read/write` |
| Org structure: locations | `GET/POST /locations` | `location.read/write` |
| Departments tree | `GET/POST /departments` | `department.read/write` |
| Teams | `GET/POST /teams` | `team.read/write` |
| Position catalog | `GET/POST /positions` | `position.read/write` |
| All employees | `GET /employees` (full scope) | `employee.read` |
| Create employee | `POST /employees` | `employee.write` |
| Employee detail | `GET /employees/{id}` | `employee.read` (broad) |
| Status lifecycle | `PATCH .../status` | `employee.write` / `employee.terminate` |
| Employment change | `PATCH .../employment` | `employee.write` |
| Set compensation | `POST .../compensation` | `employee.compensation.write` — emits `salary.changed` |
| View compensation | `GET .../compensation` | `employee.compensation.read` |
| Upload document | `POST .../documents` (JSON metadata `\n` bytes) | `employee.documents.write` |
| List documents | `GET .../documents` | `employee.documents.read` |
| Generate download URL | `GET .../documents/{id}/url` | `employee.documents.read` |
| Bulk export | `GET /employees?…` | `employee.export` — emits `employee.exported` audit event |

**Design note:** the HR-Admin portal is the only surface with the write
permissions. Its "Set compensation" action is the one that triggers the
`salary.changed` security-monitoring event (amount omitted from the audit
trail). The "Bulk export" tab triggers `employee.exported`.

---

## Cross-cutting UI rules (for whoever builds the frontend)

1. **Tenant is never a form field.** RLS derives it from the token; ULIDs
   in paths are the only identifiers.
2. **Compensation is always a separate, permission-gated call** — never bundled
   into the profile card. If `employee.compensation.read` is absent, no call is
   made (not "call and hide").
3. **Documents never preview inline** in Phase 1; the only way to see content
   is the presigned-URL round-trip, which the backend gates and audits
   (`document.accessed`).
4. **Status badges** map to the four `EmployeeStatus` values
   (`onboarding`, `active`, `on_leave`, `terminated`); the terminal transition
   is visually distinct and requires the `employee.terminate` permission.
5. **No client-side filtering of another tenant's data is trusted** — the rule
   "never rely solely on frontend filtering" is satisfied because RLS makes
   cross-tenant rows invisible at the DB; the frontend never sees them.

---

## Gaps to close before the portals can be fully built

- `GET /employees/{id}/employment` (list `wf_employment_records`) — not exposed in Phase 1.
- Real multipart for document upload (current body = `JSON\nbytes`; `ponytail:`).
- `current_*_ulid` fields in the employee response are UUID-stringified, not
  re-resolved to ULIDs (marked `ponytail:` in `employee_row_to_response`).
- Notifications provider (Mailhog is in compose; no Rust mailer yet — Phase 3).
- Full org-chart traversal for managers (direct reports only in MVP).
- A design system / component library — does not exist in AOS yet.
