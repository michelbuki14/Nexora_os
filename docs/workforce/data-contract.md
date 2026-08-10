# Workforce Data Contract

> Single source of truth for table schemas, endpoint signatures, the
> role/permission matrix, and redaction rules. Consumed by:
> - the future frontend (any React/TS portal, when a frontend crate exists),
> - the Payroll service (Phase 4 — reads Workforce by reference),
> - any external integrator.
>
> This doc describes what is **implemented in Phase 1**. Everything here is
> backed by a real handler in `services/workforce-service/src/handlers.rs`.

All endpoints are mounted under `/api/v1/workforce`, require a valid Keycloak
OIDC bearer token, and are tenant-isolated by RLS. Errors are the shared
`AosError → ErrorResponse` shape from `common::error`.

---

## 1. Tables (migration 008)

| Table | Mutability | Sensitive? | Notes |
|-------|------------|------------|-------|
| `wf_legal_entities` | mutable | no | org-scoped; FK `countries` |
| `wf_locations` | mutable | no | FK `wf_legal_entities`; default tz `Africa/Lubumbashi` |
| `wf_departments` | mutable | no | self-FK `parent_department_id`; `head_employee_id` (deferred FK) |
| `wf_teams` | mutable | no | FK `wf_departments`; `lead_employee_id` (deferred FK) |
| `wf_positions` | mutable | no | `employment_type` CHECK |
| `wf_employees` | mutable | **yes** | `national_id_hash` + `national_id_last4` only (never raw); status CHECK |
| `wf_employment_records` | **append-only** | no | every change = new row; `end_date NULL` = open |
| `wf_compensation_records` | **append-only** | **yes** | `gross_amount_minor BIGINT` (integer minor units); `employee.compensation.read` gated |
| `wf_documents` | soft-delete | no | metadata only — bytes in MinIO at `object_key`; `is_active` flag |

All tables: UUID PK, `CHAR(26)` ULID external id, `tenant_id`/`org_id` FKs, RLS
`WITH CHECK` on tenant (`current_setting('aos.current_tenant_id')`), system bypass
via `current_setting('aos.is_system', true) = 'true'`.

**Object key namespace (documents):** `{tenant_ulid}/{employee_ulid}/{doc_ulid}` —
ULIDs are unguessable; no public bucket path exists.

---

## 2. Endpoints

### Legal entities
```
POST   /legal-entities                        legal_entity.write
GET    /legal-entities?page=&page_size=       legal_entity.read
```

### Locations
```
POST   /locations                             location.write
GET    /locations?page=&page_size=            location.read
```

### Departments
```
POST   /departments                           department.write
GET    /departments?page=&page_size=          department.read
```

### Teams
```
POST   /teams                                 team.write
GET    /teams?page=&page_size=                team.read
```

### Positions
```
POST   /positions                             position.write
GET    /positions?page=&page_size=            position.read
```

### Employees
```
POST   /employees                             employee.write
GET    /employees?page=&page_size=            employee.read OR employee.export
GET    /employees/{employee_ulid}             employee.read | self | manager (IDOR)
PATCH  /employees/{employee_ulid}/status      employee.write (or employee.terminate if → terminated)
PATCH  /employees/{employee_ulid}/employment  employee.write
```

### Compensation
```
POST   /employees/{employee_ulid}/compensation   employee.compensation.write
GET    /employees/{employee_ulid}/compensation   employee.compensation.read
```

### Documents
```
POST   /employees/{employee_ulid}/documents               employee.documents.write
GET    /employees/{employee_ulid}/documents?page=&page_size=  employee.documents.read
GET    /employees/{employee_ulid}/documents/{doc_ulid}/url    employee.documents.read
```

---

## 3. Request/response shapes

### CreateLegalEntityRequest → LegalEntityResponse
```jsonc
// req
{ "name": "string", "registration_number": "string?", "country_ulid": "string?",
  "address": {}? }
// 201
{ "ulid":"string", "name":"string", "registration_number":"string?",
  "status":"active", "created_at":"ISO8601" }
```

### CreateLocationRequest → LocationResponse
```jsonc
// req
{ "legal_entity_ulid":"string", "name":"string", "code":"string?",
  "address":{}?, "timezone":"string?" }   // tz defaults to Africa/Lubumbashi
// 201
{ "ulid":"string", "name":"string", "code":"string?", "timezone":"string",
  "is_active":true, "created_at":"ISO8601" }
```

### CreateDepartmentRequest → DepartmentResponse
```jsonc
// req
{ "name":"string", "code":"string?", "parent_department_ulid":"string?",
  "location_ulid":"string?" }
// 201
{ "ulid":"string", "name":"string", "code":"string?", "is_active":true, "created_at":"ISO8601" }
```

### CreateTeamRequest → TeamResponse
```jsonc
{ "department_ulid":"string", "name":"string", "code":"string?" }
→ { "ulid","name","code?","is_active","created_at" }
```

### CreatePositionRequest → PositionResponse
```jsonc
// req
{ "title":"string", "code":"string?", "department_ulid":"string?",
  "job_grade":"string?", "description":"string?", "responsibilities":"string?",
  "employment_type":"permanent|contract|intern|consultant?" }
// 201 — employment_type defaults to "permanent"
{ "ulid","title","code?","job_grade?","employment_type":"string","is_active":true,"created_at" }
```

### CreateEmployeeRequest → EmployeeResponse
```jsonc
// req — national_id is RAW, hashed server-side, never stored
{ "employee_number":"string", "legal_name":"string", "preferred_name":"string?",
  "email":"string", "phone":"string?", "date_of_birth":"YYYY-MM-DD?",
  "gender":"male|female|other|prefer_not_to_say?", "national_id":"string?",
  "hire_date":"YYYY-MM-DD?", "department_ulid":"string?", "position_ulid":"string?",
  "location_ulid":"string?", "manager_employee_ulid":"string?",
  "employment_type":"permanent|contract|intern|consultant?" }
// 201 — status starts "onboarding"; NO compensation fields
{ "ulid","employee_number","legal_name","preferred_name?","email","phone?",
  "gender?","national_id_last4":"string?",   // never the full id
  "status":"onboarding","hire_date?",
  "current_department_ulid?","current_position_ulid?","current_location_ulid?",
  "manager_employee_ulid?","created_at","updated_at" }
```
> Note: in Phase 1 the `*_ulid` fields of the employee response are populated by
> stringifying the FK UUIDs from the row (UUID→string), not by re-resolving to the
> underlying ULID, until a JOIN helper is added (marked `ponytail:` in code).
> They are non-authoritative for cross-references; resolve via the list endpoints.

### UpdateEmployeeStatusRequest → EmployeeResponse
```jsonc
// req
{ "status":"onboarding|active|on_leave|terminated",
  "termination_date":"YYYY-MM-DD?", "termination_reason":"string?" }
// 200 — re-fetched EmployeeResponse
```
> If `status==terminated`, the handler requires `employee.terminate` instead of
> `employee.write`.

### UpdateEmploymentRequest → 200 (no body)
```jsonc
{ "position_ulid":"string?", "department_ulid":"string?", "location_ulid":"string?",
  "employment_type":"...?", "effective_date":"YYYY-MM-DD", "change_reason":"string?" }
```
> Closes the prior open `wf_employment_records` row (`end_date = effective_date`),
> appends a new row, and mirrors `current_*` on the employee.

### CreateCompensationRequest → CompensationResponse
```jsonc
// req
{ "gross_amount_minor": <int>, "currency_code":"CDF|USD|EUR|ZAR|KES|NGN|GHS",
  "frequency":"monthly|annual|hourly|weekly", "effective_date":"YYYY-MM-DD",
  "change_reason":"string?" }
// 201
{ "ulid":"string", "gross_amount_minor": <int>, "currency_code":"string",
  "frequency":"string", "effective_date":"YYYY-MM-DD", "created_at":"ISO8601" }
```
> `gross_amount_minor` is an integer of minor currency units (centimes/cents).
> Never a float. Emits the `salary.changed` security-monitoring event **without**
> the amount in the audit payload.

### Documents — upload (`POST .../documents`)
The request body is **not** JSON. It is a single binary payload:
```
<one JSON line of CreateDocumentMetadataRequest>\n<raw file bytes>
```
```jsonc
// metadata line
{ "doc_type":"contract|id_document|certificate|offer_letter|payslip|other",
  "filename":"string", "mime_type":"string", "size_bytes": <int>?, "sha256":"string?" }
```
`size_bytes`/`sha256` in the metadata are advisory; the handler recomputes both
from the actual bytes (SHA-256) and stores the computed values. → 201
`DocumentResponse { ulid, doc_type, filename, mime_type, size_bytes, is_active, created_at }`.

> `ponytail:` base64-in-body avoids a multipart parser for MVP. Real multipart is
> the long-term path when a frontend crate exists.

### Documents — list (`GET .../documents`) → PaginatedResponse<DocumentResponse>

### Documents — presigned URL (`GET .../documents/{doc_ulid}/url`)
```jsonc
// 200
{ "presigned_url":"https://...", "expires_in_secs": 60 }
```
The URL is a MinIO presigned GET, TTL from `config.s3.presign_ttl_secs` (default
60s). The handler emits `document.accessed`. There is **no** public/anonymous
download path — the URL is minted only after RBAC + IDOR + tenant checks pass.

---

## 4. Role / permission matrix

Permissions seeded by migration 008 into the shared `permissions` table
(`category='workforce'`, `is_system=false`). Roles are Keycloak realm roles
mapped into `AuthContext.roles`; the granular keys carry the actual grants.
`has_permission("*")` matches everything (super-admin).

| Permission key | SUPER_ADMIN | ORG_ADMIN | HR_ADMIN | PAYROLL_ADMIN | FINANCE_ADMIN | MANAGER | EMPLOYEE | AUDITOR |
|----------------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `employee.read` | ✅ | ✅ | ✅ | ✅ | ✅ | (reports only) | (self only) | ✅ |
| `employee.write` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `employee.terminate` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `employee.export` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `employee.compensation.read` | ✅ | ✅ | ✅ | ✅ | ✅ | – | – | ✅ |
| `employee.compensation.write` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `employee.documents.read` | ✅ | ✅ | ✅ | – | – | (reports) | (self) | – |
| `employee.documents.write` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `department.read` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | – | ✅ |
| `department.write` | ✅ | ✅ | ✅ | – | – | – | – | – |
| `position.read` / `position.write` | ✅ | ✅ | ✅/✅ | ✅/– | ✅/– | ✅/– | –/– | ✅/– |
| `location.read` / `location.write` | ✅ | ✅ | ✅/✅ | –/– | –/– | ✅/– | –/– | ✅/– |
| `legal_entity.read` / `legal_entity.write` | ✅ | ✅ | ✅/✅ | –/– | –/– | –/– | –/– | ✅/– |
| `team.read` / `team.write` | ✅ | ✅ | ✅/✅ | –/– | –/– | ✅/– | –/– | ✅/– |

`(reports only)` = scope-to-direct-reports (MANAGER list path).
`(self only)` = EMPLOYEE may `GET /employees/{own_ulid}` via the IDOR self-view
branch, without `employee.read`.

> Final role→permission wiring lives in the Keycloak realm export
> (`keycloak/realm-export.json`) and/or a future RBAC seed migration; the table
> above is the intended grant set for Phase 1.

---

## 5. Redaction rules (security)

The following **never** appear in any response, audit payload, or tracing span:

| Field | Stored as | Returned as | In audit `changes` | In tracing |
|-------|-----------|-------------|--------------------|------------|
| national_id (raw) | **never** | – | – | – |
| national_id_hash | SHA-256 hex | – | – | – |
| national_id_last4 | last 4 chars | `national_id_last4` | – | – |
| gross_amount_minor | BIGINT | only via `/compensation` (`employee.compensation.read`) | **omitted** from `salary.changed` | – |
| document object_key | TEXT | – | – | DEBUG only |
| document bytes | MinIO only | via presigned URL (TTL 60s) | – | – |

Rules enforced in code:
- `emit_workforce_event` constructs the `changes` JSON explicitly per call site —
  sensitive values are simply not inserted. Auditing cannot "leak" by accident
  because the payload is hand-built, not a serde dump of the row.
- `tracing::info!`/`warn!` calls carry only ULIDs, statuses, and the action verb.
- The `salary.changed` event records `currency`, `frequency`, `effective_date` —
  **never** the amount.
- The `document.uploaded`/`document.accessed` events record `doc_type`,
  `filename` (upload only), `employee_ulid` — **never** the object key.

---

## 6. Audit + outbox events

Every lifecycle write dual-writes (same transaction, same `DbConn`):
1. a hash-chained row in `audit_events` (`compute_chain_hash(prev_hash, canonical_payload)`,
   canonical = lexicographically-sorted-keys JSON), and
2. an `outbox_events` row (`aggregate_type='workforce'`,
   `event_type=<action>`, `published_at=NULL`) for the future dispatcher job.

| `event_type` (also `audit_events.action`) | Trigger |
|--------------------------------------------|---------|
| `legal_entity.created` | POST /legal-entities |
| `location.created` | POST /locations |
| `department.created` | POST /departments |
| `team.created` | POST /teams |
| `position.created` | POST /positions |
| `employee.lifecycle.created` | POST /employees |
| `employee.lifecycle.{status}` | PATCH .../status (status = onboarding/active/on_leave/terminated) |
| `employee.lifecycle.employment_changed` | PATCH .../employment |
| `salary.changed` | POST .../compensation (amount omitted) |
| `employee.exported` | GET /employees when caller has `employee.export` |
| `document.uploaded` | POST .../documents (object_key omitted) |
| `document.accessed` | GET .../documents/{id}/url (object_key omitted) |

> `privilege.escalated` and `mass_employee_update` are reserved security-monitoring
> events. `privilege.escalated` is emitted by tenant-service (role assignment),
> not the workforce service. `mass_employee_update` is not yet implemented (a
> bulk-write path does not exist in Phase 1); both are flagged in code for the
> security feed.
