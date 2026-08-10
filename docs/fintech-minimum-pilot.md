# Fintech Minimum Pilot — Executable Issue Set

Goal: **first paid pilot in Kinshasa.** One workflow: merchant onboarding →
wallet → mobile-money payout/collection → regulatory report. Every issue is
independently executable, ordered so that money moves end-to-end as early as
possible.

Rules that govern all issues:

- Reuse `aos-common`: `Money` (CDF default), `add_rls_middleware`/`DbConn`/`RlsState`,
  `auth_middleware`, `audit`, `validation`, `ulid`, `error`. Do not rebuild these.
- Every money-mutating endpoint is **idempotent** (client-supplied `Idempotency-Key`,
  unique constraint) — a retry must never double-credit or double-debit.
- Every table ships with RLS enabled + a policy mirroring `migrations/001`, and
  grants to `aos_app` (pattern in `migrations/007`).
- Every mutation writes one `audit_events` row.
- "Done" includes a unit test for the money/state logic. Integration tests that
  need a live postgres run on Linux/CI (Windows npipe is broken for
  testcontainers — see `services/common/tests/tenant_isolation.rs` module docs).

Current state to build on: gateway serves placeholder routers only
(`api-gateway/src/main.rs`); `fintech-service` is a `println!` stub; schema has
tenants/orgs/users/roles/memberships/audit (all RLS) but **no fintech tables**.

---

## Phase 0 — Make fintech-service real (week 1)

### Issue 1: Turn fintech-service into an Axum service on the common stack
- **What:** Replace the `println!` stub in `services/fintech-service/src/main.rs`
  with an Axum binary wired exactly like `tenant-service` (Config::load, db pool,
  RLS middleware, auth middleware, health, tracing, graceful shutdown).
- **Why:** Nothing else in this set is possible until the service can talk to
  postgres under tenant isolation.
- **Done when:** `cargo run -p aos-fintech-service` serves `/health` 200 and a
  placeholder `/api/v1/finance` route; `cargo check --workspace` clean.
- **Verification (DRC):** none needed — pure wiring.

### Issue 2: Route finance traffic through the gateway
- **What:** Mount the fintech router in `api-gateway/src/main.rs` under
  `/api/v1/finance` (replace the `placeholder_router("finance")`), or document
  why the service stays separate. Keep the port model consistent with the other
  services.
- **Why:** The gateway is the single entry point and owns CORS/tracing/timeouts.
- **Done when:** `curl localhost:3000/api/v1/finance/health` returns 200.

### Issue 3: Migration `010_fintech_schema.sql`
- **What:** Fintech tables, all tenant-scoped, RLS-enabled:
  - `merchants` — id, tenant_id, business_name, owner_name, national_id
    (DRC: pick the identifier the buyer actually holds — RCCM registration
    number and/or national ID), phone, mobile_money_number, status, KYC JSONB.
  - `wallets` — id, tenant_id, merchant_id, currency (default CDF), balance
    NUMERIC(19,4), version (optimistic lock).
  - `ledger_entries` — id, wallet_id, direction, amount, balance_after,
    reference_type/reference_id, created_at (append-only).
  - `payouts` — id, tenant_id, wallet_id, amount, provider, target_phone,
    status (`requested|processing|sent|confirmed|failed`), provider_ref,
    idempotency_key UNIQUE, created_at.
  - `collections` — id, tenant_id, wallet_id, amount, provider, payer_phone,
    provider_ref, idempotency_key UNIQUE, status, created_at.
- **Why:** Double-entry via `ledger_entries` is the primitive everything else
  reads. `version` on wallets prevents lost updates.
- **Done when:** `cargo run -p aos-migrate -- run` applies it; ledger table has
  a new success row; RLS blocks cross-tenant reads when run as `aos_app`.

---

## Phase 1 — Merchant onboarding + wallet + payout (weeks 1–3)

### Issue 4: `POST /api/v1/finance/merchants`
- **What:** Create a merchant under the current tenant. Validate KYC fields via
  `validation`, audit the create. Status defaults `pending_verification`.
- **Done when:** Creates a row visible only to the calling tenant; duplicate
  business-name-per-tenant rejected; unit test for validation.

### Issue 5: `GET /api/v1/finance/merchants/{id}` + `GET /api/v1/finance/wallets/{merchant_id}`
- **What:** Read paths with balance.
- **Done when:** 200 with wallet balance; cross-tenant fetch returns 404 (RLS
  works, not just a filter).

### Issue 6: Wallet credit/debit + ledger (core money primitive)
- **What:** `POST /api/v1/finance/wallets/{id}/credit` and `/debit`, both
  idempotent, both posting a `ledger_entries` row, both using `Money` checked
  arithmetic, wallet `version` bumped atomically
  (`UPDATE ... SET balance = balance + $1, version = version + 1 WHERE version = $2`).
- **Done when:** unit test: double-debit with the same idempotency key credits
  once; insufficient-balance debit rejected; balance_after chain is consistent.

### Issue 7: `POST /api/v1/finance/payouts` (create)
- **What:** Debit wallet, freeze the amount into status `requested`, store
  provider + target phone + idempotency key. Validation: amount > 0, merchant
  has a mobile-money number, wallet has balance.
- **Done when:** Unit test on the state machine: only `requested` payouts can
  move to `processing`; idempotent create returns the same payout.

---

## Phase 2 — Mobile-money integration (weeks 3–5) ← revenue starts here

### Issue 8: `MobileMoneyProvider` trait + one real adapter
- **What:** A thin trait (`submit_payout`, `query_status`, `verify_callback`)
  with **one** implementation. Anchor on **M-Pesa RDC (Vodacom)** — the largest
  PSP in DRC by mobile-money volume — and keep the adapter behind a config
  flag so Airtel/Orange Money RDC can be added as a second implementation
  without touching the workflow. Start on the provider's sandbox.
- **Why:** The workflow (payout → callback → reconcile) must not depend on which
  PSP is behind it. One adapter first; the trait is the seam, not a framework.
- **Done when:** Sandbox payout submits and returns a provider_ref.

### Issue 9: Payout executor worker
- **What:** A background task (or Redis-backed queue — Redis is already in the
  stack) that picks `processing` payouts, calls the adapter, records
  `provider_ref`, moves to `sent`. Retries with backoff on network error only —
  never re-submit a payout whose final state is unknown without a provider-side
  query first.
- **Done when:** Worker drives a payout `requested → processing → sent` and
  leaves it in `sent` awaiting callback.

### Issue 10: Callback receiver + reconciliation
- **What:** `POST /api/v1/finance/providers/{provider}/callbacks` (signature
  verified against the provider's webhook secret). On `confirmed`: wallet
  already debited at create, so just mark payout `confirmed` and post the
  matching ledger entry. On `failed`: **credit the wallet back** atomically.
- **Why:** This is where money safety lives. A failed payout that doesn't
  refund is the fastest way to lose a pilot.
- **Done when:** Unit test: callback marks `confirmed`; failed callback refunds
  the exact debited amount once, even if the callback arrives twice.

### Issue 11: Collection into merchant wallet
- **What:** `POST /api/v1/finance/collections` — receive a mobile-money payment
  reference and credit the merchant wallet (mirror of Issue 10, reversed).
  Idempotent on provider_ref.
- **Done when:** End-to-end in sandbox: customer pays → callback → merchant
  wallet balance increases → ledger row exists. **This is the demo that sells
  the pilot.**

---

## Phase 3 — Merchant-facing minimum + reports (weeks 5–8)

### Issue 12: `GET /api/v1/finance/transactions?merchant_id=&from=&to=`
- **What:** Read from `ledger_entries` (paginated). This is the statement
  endpoint merchants actually use day-to-day.
- **Done when:** Paginated, tenant-scoped, balance-summing unit test.

### Issue 13: CSV statement export
- **What:** `GET /api/v1/finance/merchants/{id}/statement.csv` — same query,
  CSV response. Merchants in the DRC run Excel-first businesses; CSV beats a
  dashboard.
- **Done when:** Valid CSV with a CDF balance row.

### Issue 14: Regulatory reporting (TAIL — build after confirming the spec)
- **What:** **Do NOT build this blind.** Confirm with the pilot buyer / BCC
  which exact report they need and in what format (CB02 is the BCC
  interbank-clearing reference; the deliverable format is defined by the
  counterparty, not by the code). Then implement one export endpoint from the
  `payouts`/`collections`/ledger data.
- **Why:** It is the user-visible "compliance-ready" signal that wins the
  *contract*, but it is not what makes the first franc flow — the
  collection/payout loop is. Build it only once the format is pinned.
- **Done when:** Report matches the buyer-signed format spec, exactly.

---

## Phase 4 — Pilot provisioning (week 8+)

### Issue 15: Tenant + user provisioning script
- **What:** A script/CLI that creates an org + tenant + user for the pilot,
  provisions their Keycloak realm client, and seeds the demo merchant.
  Reuse `tenant-service` handlers.
- **Why:** Manual provisioning is the thing that makes "onboard the pilot"
  take a week instead of an afternoon.
- **Done when:** Script is idempotent and produces a login + a funded demo
  merchant in under five minutes.

---

## Sequencing truth

| What produces first revenue | What wins the contract |
|----------------------------|------------------------|
| Issues 1–11: money moves from a merchant's customer into their wallet and out to mobile money, safely | Issue 14 + a signed LOI |

CB02 (Issue 14) is deferred *on purpose*: it only matters once there is real
interbank volume, and its exact format must come from the counterparty. Do not
let it block the pilot.

## External facts to verify with the counterparty (not invented here)
- M-Pesa RDC / Airtel Money RDC / Orange Money RDC: exact sandbox + production
  API mechanics, fees, and settlement currency.
- BCC licensing position for a payment-service provider operating this stack.
- CB02 report format and submission channel.
