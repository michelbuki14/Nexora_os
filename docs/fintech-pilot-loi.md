# LETTER OF INTENT — Mobile-Money Settlement & Reconciliation Pilot (Kinshasa)

**To:** [Counterparty — mobile-money agent-network operator / MFI / PSP / institution]
**From:** [Your organization, legal entity, DRC presence if any] · **Contact:** [name, title, email, phone] · **Date:** ________ · **LOI version 0.1**

---

**Positioning.** A settlement and reconciliation spine for mobile-money merchant networks in the DRC — multi-tenant from the database up, sovereign-data-ready, built so a merchant's daily CDF flow reconciles in one place.

**The problem.** Your merchants take M-Pesa RDC, Airtel Money RDC, and Orange Money RDC payments all day, and takings arrive as unlabeled phone transactions. Reconcile and settle them and you face manual work: CSV files pulled from each operator, re-keyed into Excel, night-before settlement, disputes nobody can prove — lost float and a reporting burden you cannot automate. And no option today keeps that data resident in-country; cloud tools default to moving Congolese transaction data abroad. (Residency requirements, and where each PSP stores data, must be confirmed with counsel and the BCC.)

**Pilot scope — exactly one workflow, end to end.** Merchant onboarding → CDF wallet + double-entry ledger → M-Pesa RDC payout → callback reconciliation → collection → CSV statement. We build to these acceptance properties: money-mutating calls are idempotent (a retried request or callback can never double-debit or double-credit), failed payouts refund, and callbacks are verified per M-Pesa RDC's actual API terms (to be confirmed with the provider). Multi-tenant isolation is enforced by Row-Level Security at the database layer. A second PSP (Airtel/Orange Money RDC) can be added behind the same provider interface without touching workflow logic. CSV beats a dashboard here — DRC merchants run Excel-first. Regulatory reports (e.g. a BCC CB02-style interbank report) are deliberately deferred: **you name the exact format, we match it.** We will not build to a guessed spec.

**What is real today.** A working, code-reviewable foundation: RLS tenant isolation, Keycloak-issued JWT authentication, an append-only per-tenant SHA-256 hash-chained audit log, checked `Money` arithmetic with CDF as default currency, migrations applied under a ledger, CI-gated. The product layer is a written 15-issue execution plan. We are not selling a finished product or existing customers — that honesty is the point: DRC has seen enough vaporware.

**Deliverables and timeline (8–12 weeks).** Weeks 1–3: merchant onboarding, wallets, ledger. Weeks 3–5: M-Pesa RDC sandbox payout adapter, callbacks, reconciliation, collection. Weeks 5–8: transaction history, CSV statements, one report in the format you sign off. Week 8+: provisioning in your environment, pilot go-live. Offline behavior is in scope: on network loss, operations queue and retry, reconciling on reconnect. We run phase gates: (1) sandbox proves money moves end-to-end; (2) your team signs the acceptance criteria; (3) live pilot on low-value merchant float. Assumptions the timeline rests on, to be confirmed at kickoff: sandbox access, your KYC/test-merchant records, connectivity and power at the pilot node.

**Commercial model — indicative, negotiable ranges.** Implementation/integration fee: USD 15,000–40,000. Monthly platform fee: USD 500–2,000 per tenant (or a per-network flat rate), discounted during the pilot. Per-transaction share: 0.1–0.5% of settled CDF volume. CDF-equivalent pricing available. Fees begin only under a signed pilot agreement.

**Why hybrid / sovereign data.** Tenant financial data stays resident in-country — on-prem in Kinshasa or on Azure Stack Edge — with a cloud control plane. Control-plane contents, backup/DR, and data return/erasure at pilot end are defined in the pilot agreement. This answers the data-residency question before it is asked.

**What you bring.** Two named contacts (business + technical); M-Pesa RDC sandbox/test access through your existing Vodacom relationship (or we help secure it); KYC records and one test merchant; the pilot report format; the settlement float and settlement-account arrangement (the wallet is a ledger balance — real payout needs a funded account and, where required, a licensed PSP/BCC posture, confirmed with counsel before any live franc moves).

**The ask.** Sign this Letter of Intent to pilot. It commits you to nothing beyond the items above; it commits us to a working settlement and reconciliation workflow in your environment within 8–12 weeks. No exclusivity, no upfront license purchase. Dates and contacts are intentionally blank; we propose a Kinshasa working session within two weeks to pin scope, the report format, and acceptance criteria, and we hold this LOI open for [60/90] days. French version available for signature.

**Signed — [Your organization]:** ________ Name/Title: ________
**Signed — [AOS / your entity]:** ________ Name/Title: ________

---
*External facts — BCC licensing mechanics, PSP sandbox/production API terms, exact law references (including DRC data-protection law), and report formats — must be verified with counsel and the counterparty before any binding agreement. This LOI is confidential, creates no license, exclusivity, or regulatory-status claim, and is governed by law to be agreed at signature.*
