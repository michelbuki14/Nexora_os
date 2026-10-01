// Seed data + presentation metadata for the demo-backed "planned" modules.
// These modules have no live backend in Phase 1; in demo mode they render real,
// interactive, seed-backed pages so the whole product is explorable. Every
// mutation persists to the same sessionStorage store as the Workforce demo.

export type ModuleCell = string | number | boolean;

export interface ModuleTable {
  columns: { key: string; label: string }[];
  rows: Record<string, ModuleCell>[];
}

export interface ModuleKpi {
  label: string;
  value: string;
  hint?: string;
}

export interface ModuleMeta {
  title: string;
  description: string;
  kind: "table" | "analytics";
  /** KPIs derived from the current rows. */
  kpis: (rows: Record<string, ModuleCell>[]) => ModuleKpi[];
  /** Action buttons shown in the header. */
  actions?: { label: string; type: "add" | "toggle-all" }[];
  /** Column whose boolean values render as inline toggle switches. */
  toggleColumn?: string;
  /** Column to sum for a headline currency KPI (optional). */
  sumColumn?: string;
}

function ym(offset = 0): string {
  const d = new Date();
  d.setMonth(d.getMonth() + offset);
  return d.toISOString().slice(0, 7);
}
function today(): string {
  return new Date().toISOString().slice(0, 10);
}

export const SEED_MODULE_TABLES: Record<string, ModuleTable> = {
  payroll: {
    columns: [
      { key: "period", label: "Period" },
      { key: "gross", label: "Gross pay" },
      { key: "employees", label: "Employees" },
      { key: "status", label: "Status" },
      { key: "paid_on", label: "Paid" },
    ],
    rows: [
      { period: ym(-2), gross: "$482,300", employees: 4, status: "Paid", paid_on: today() },
      { period: ym(-1), gross: "$489,120", employees: 4, status: "Paid", paid_on: today() },
      { period: ym(0), gross: "$491,770", employees: 4, status: "Processing", paid_on: "—" },
    ],
  },
  finance: {
    columns: [
      { key: "date", label: "Date" },
      { key: "account", label: "Account" },
      { key: "memo", label: "Memo" },
      { key: "amount", label: "Amount (USD)" },
      { key: "balance", label: "Balance" },
    ],
    rows: [
      { date: ym(-2) + "-01", account: "Revenue", memo: "Subscription — Acme", amount: 120000, balance: 120000 },
      { date: ym(-1) + "-15", account: "OpEx", memo: "Payroll run", amount: -489120, balance: -369120 },
      { date: ym(0) + "-01", account: "Revenue", memo: "Subscription — Globex", amount: 84000, balance: -285120 },
    ],
  },
  payments: {
    columns: [
      { key: "id", label: "Ref" },
      { key: "payee", label: "Payee" },
      { key: "method", label: "Method" },
      { key: "amount", label: "Amount" },
      { key: "status", label: "Status" },
      { key: "date", label: "Date" },
    ],
    rows: [
      { id: "PAY-1001", payee: "Safaricom", method: "M-Pesa", amount: "$1,200", status: "Settled", date: today() },
      { id: "PAY-1002", payee: "Flutterwave", method: "Card", amount: "$3,450", status: "Settled", date: today() },
      { id: "PAY-1003", payee: "Airtel Money", method: "Mobile", amount: "$780", status: "Pending", date: today() },
    ],
  },
  analytics: {
    columns: [
      { key: "month", label: "Month" },
      { key: "active_users", label: "Active users" },
      { key: "sessions", label: "Sessions" },
      { key: "retention", label: "Retention %" },
    ],
    rows: [
      { month: ym(-5), active_users: 1820, sessions: 9400, retention: 71 },
      { month: ym(-4), active_users: 2110, sessions: 11200, retention: 73 },
      { month: ym(-3), active_users: 2480, sessions: 13600, retention: 74 },
      { month: ym(-2), active_users: 2905, sessions: 15800, retention: 76 },
      { month: ym(-1), active_users: 3310, sessions: 18400, retention: 78 },
      { month: ym(0), active_users: 3680, sessions: 20100, retention: 79 },
    ],
  },
  security: {
    columns: [
      { key: "title", label: "Finding" },
      { key: "severity", label: "Severity" },
      { key: "status", label: "Status" },
      { key: "owner", label: "Owner" },
    ],
    rows: [
      { title: "MFA not enforced for 2 admins", severity: "High", status: "Open", owner: "Platform" },
      { title: "Stale API key in CI", severity: "Medium", status: "Open", owner: "Security" },
      { title: "Over-broad IAM role on staging", severity: "Medium", status: "Resolved", owner: "Platform" },
      { title: "Outdated dependency (lodash)", severity: "Low", status: "Open", owner: "AppSec" },
    ],
  },
  infrastructure: {
    columns: [
      { key: "service", label: "Service" },
      { key: "status", label: "Status" },
      { key: "uptime", label: "Uptime 30d" },
      { key: "region", label: "Region" },
    ],
    rows: [
      { service: "api-gateway", status: "Healthy", uptime: "99.98%", region: "africa-south-1" },
      { service: "workforce-svc", status: "Healthy", uptime: "99.95%", region: "africa-south-1" },
      { service: "postgres", status: "Degraded", uptime: "99.40%", region: "africa-south-1" },
      { service: "redis-cache", status: "Healthy", uptime: "100.0%", region: "africa-west-1" },
    ],
  },
  integrations: {
    columns: [
      { key: "name", label: "Integration" },
      { key: "category", label: "Category" },
      { key: "enabled", label: "Enabled" },
    ],
    rows: [
      { name: "Slack", category: "Messaging", enabled: true },
      { name: "Okta", category: "SSO", enabled: true },
      { name: "Stripe", category: "Payments", enabled: false },
      { name: "Zoom", category: "Meetings", enabled: false },
      { name: "Salesforce", category: "CRM", enabled: true },
    ],
  },
  developer: {
    columns: [
      { key: "name", label: "Key name" },
      { key: "key", label: "Key" },
      { key: "created", label: "Created" },
      { key: "last_used", label: "Last used" },
    ],
    rows: [
      { name: "Production", key: "sk_live_••••3f9a", created: ym(-3) + "-02", last_used: today() },
      { name: "CI Pipeline", key: "sk_live_••••7b21", created: ym(-1) + "-14", last_used: ym(0) + "-01" },
    ],
  },
  organizations: {
    columns: [
      { key: "name", label: "Organization" },
      { key: "plan", label: "Plan" },
      { key: "seats", label: "Seats" },
      { key: "status", label: "Status" },
    ],
    rows: [
      { name: "Nexora Africa Ltd", plan: "Enterprise", seats: 4, status: "Active" },
      { name: "Congo Logistics SARL", plan: "Growth", seats: 12, status: "Active" },
      { name: "Kivu Foods", plan: "Starter", seats: 3, status: "Trial" },
    ],
  },
  audit: {
    columns: [
      { key: "ts", label: "Timestamp" },
      { key: "actor", label: "Actor" },
      { key: "action", label: "Action" },
      { key: "target", label: "Target" },
    ],
    rows: [
      { ts: today() + " 09:14", actor: "demo@nexora.africa", action: "employee.create", target: "EMP-0005" },
      { ts: today() + " 08:02", actor: "system", action: "payroll.run", target: ym(0) },
      { ts: ym(-1) + "-28 17:41", actor: "admin@nexora.africa", action: "integration.toggle", target: "Okta" },
      { ts: ym(-1) + "-21 11:09", actor: "demo@nexora.africa", action: "document.upload", target: "EMP-0001" },
    ],
  },
  admin: {
    columns: [
      { key: "name", label: "Setting" },
      { key: "description", label: "Description" },
      { key: "enabled", label: "Enabled" },
    ],
    rows: [
      { name: "Enforce MFA", description: "Require MFA for all members", enabled: true },
      { name: "Public API", description: "Allow API access from outside the tenant", enabled: false },
      { name: "Audit logging", description: "Record all privileged actions", enabled: true },
      { name: "Maintenance mode", description: "Temporarily disable member sign-in", enabled: false },
    ],
  },
  ai: {
    columns: [
      { key: "agent", label: "Agent" },
      { key: "task", label: "Capability" },
      { key: "status", label: "Status" },
      { key: "usage", label: "Calls (30d)" },
    ],
    rows: [
      { agent: "Nexora Assistant", task: "Q&A over tenant docs", status: "Active", usage: 1240 },
      { agent: "Doc Intel", task: "Contract extraction", status: "Active", usage: 312 },
      { agent: "Insights", task: "Anomaly detection", status: "Beta", usage: 88 },
      { agent: "Summarizer", task: "Meeting notes", status: "Disabled", usage: 0 },
    ],
  },
  fintech: {
    columns: [
      { key: "wallet", label: "Wallet" },
      { key: "currency", label: "Currency" },
      { key: "balance", label: "Balance" },
      { key: "status", label: "Status" },
    ],
    rows: [
      { wallet: "WAL-0001", currency: "CDF", balance: "1,250,000", status: "Active" },
      { wallet: "WAL-0002", currency: "USD", balance: "4,820", status: "Active" },
      { wallet: "WAL-0003", currency: "CDF", balance: "320,500", status: "Frozen" },
    ],
  },
  retail: {
    columns: [
      { key: "sku", label: "SKU" },
      { key: "name", label: "Product" },
      { key: "price", label: "Price" },
      { key: "stock", label: "In stock" },
    ],
    rows: [
      { sku: "PHONE-01", name: "Smartphone X", price: "$189", stock: 42 },
      { sku: "SOLAR-02", name: "Solar lamp", price: "$24", stock: 310 },
      { sku: "DATA-03", name: "1GB data bundle", price: "$1.20", stock: 9800 },
    ],
  },
  gov: {
    columns: [
      { key: "ref", label: "Ref" },
      { key: "type", label: "Type" },
      { key: "applicant", label: "Applicant" },
      { key: "status", label: "Status" },
    ],
    rows: [
      { ref: "PRM-0007", type: "Building permit", applicant: "Kivu Foods", status: "Submitted" },
      { ref: "LIC-0021", type: "Driver license", applicant: "A. Mbuyi", status: "Approved" },
      { ref: "REC-0004", type: "Birth record", applicant: "J. Tshibanda", status: "Issued" },
    ],
  },
};

export const MODULE_META: Record<string, ModuleMeta> = {
  payroll: {
    title: "Payroll",
    description: "Payroll computation, run cycles, and statutory reporting (demo data).",
    kind: "table",
    sumColumn: "gross",
    kpis: (rows) => [
      { label: "Runs this quarter", value: String(rows.length) },
      { label: "Latest gross", value: String(rows[rows.length - 1]?.gross ?? "—") },
      { label: "Pending runs", value: String(rows.filter((r) => r.status === "Processing").length) },
    ],
    actions: [{ label: "Run payroll", type: "add" }],
  },
  finance: {
    title: "Finance",
    description: "General ledger, invoicing, and financial reporting (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Ledger entries", value: String(rows.length) },
      { label: "Net balance", value: "$" + rows.reduce((s, r) => s + (Number(r.balance) || 0), 0).toLocaleString() },
      { label: "Last entry", value: String(rows[rows.length - 1]?.date ?? "—") },
    ],
    actions: [{ label: "Add entry", type: "add" }],
  },
  payments: {
    title: "Payments",
    description: "Payments orchestration and provider integrations (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Transactions", value: String(rows.length) },
      { label: "Settled", value: String(rows.filter((r) => r.status === "Settled").length) },
      { label: "Pending", value: String(rows.filter((r) => r.status === "Pending").length) },
    ],
    actions: [{ label: "Record payment", type: "add" }],
  },
  analytics: {
    title: "Analytics",
    description: "Cross-module reporting and dashboards (demo data).",
    kind: "analytics",
    kpis: (rows) => [
      { label: "Active users (mo)", value: String(rows[rows.length - 1]?.active_users ?? "—") },
      { label: "Sessions (mo)", value: String(rows[rows.length - 1]?.sessions ?? "—") },
      { label: "Retention", value: String(rows[rows.length - 1]?.retention ?? "—") + "%" },
    ],
  },
  security: {
    title: "Security Center",
    description: "Security findings, access reviews, and compliance posture (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Open findings", value: String(rows.filter((r) => r.status === "Open").length) },
      { label: "High severity", value: String(rows.filter((r) => r.severity === "High" && r.status === "Open").length) },
      { label: "Resolved", value: String(rows.filter((r) => r.status === "Resolved").length) },
    ],
    actions: [{ label: "Acknowledge all", type: "toggle-all" }],
  },
  infrastructure: {
    title: "Infrastructure",
    description: "Service health, deployments, and capacity (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Services", value: String(rows.length) },
      { label: "Healthy", value: String(rows.filter((r) => r.status === "Healthy").length) },
      { label: "Degraded", value: String(rows.filter((r) => r.status === "Degraded").length) },
    ],
  },
  integrations: {
    title: "Integrations",
    description: "Third-party integrations and webhooks (demo data).",
    kind: "table",
    toggleColumn: "enabled",
    kpis: (rows) => [
      { label: "Connected", value: String(rows.filter((r) => r.enabled === true).length) },
      { label: "Available", value: String(rows.length) },
      { label: "Disabled", value: String(rows.filter((r) => r.enabled === false).length) },
    ],
  },
  developer: {
    title: "Developer / API",
    description: "API keys, documentation, and developer tooling (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "API keys", value: String(rows.length) },
      { label: "Active", value: String(rows.length) },
      { label: "Last used", value: String(rows[rows.length - 1]?.last_used ?? "—") },
    ],
    actions: [{ label: "Generate key", type: "add" }],
  },
  organizations: {
    title: "Organizations",
    description: "Tenant and organization management (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Tenants", value: String(rows.length) },
      { label: "Active", value: String(rows.filter((r) => r.status === "Active").length) },
      { label: "Total seats", value: String(rows.reduce((s, r) => s + (Number(r.seats) || 0), 0)) },
    ],
    actions: [{ label: "Add organization", type: "add" }],
  },
  audit: {
    title: "Audit",
    description: "Immutable audit trail and event search (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Events", value: String(rows.length) },
      { label: "Today", value: String(rows.filter((r) => String(r.ts).startsWith(today())).length) },
      { label: "Actors", value: String(new Set(rows.map((r) => r.actor)).size) },
    ],
  },
  admin: {
    title: "Admin",
    description: "Platform administration and tenant settings (demo data).",
    kind: "table",
    toggleColumn: "enabled",
    kpis: (rows) => [
      { label: "Settings", value: String(rows.length) },
      { label: "Enabled", value: String(rows.filter((r) => r.enabled === true).length) },
      { label: "Disabled", value: String(rows.filter((r) => r.enabled === false).length) },
    ],
  },
  ai: {
    title: "Nexora AI",
    description: "AI assistant and document intelligence agents (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Agents", value: String(rows.length) },
      { label: "Active", value: String(rows.filter((r) => r.status === "Active").length) },
      { label: "Calls (30d)", value: String(rows.reduce((s, r) => s + (Number(r.usage) || 0), 0)) },
    ],
  },
  fintech: {
    title: "Fintech (Consumers)",
    description: "Consumer wallets, transactions, and payments (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Wallets", value: String(rows.length) },
      { label: "Active", value: String(rows.filter((r) => r.status === "Active").length) },
      { label: "Frozen", value: String(rows.filter((r) => r.status === "Frozen").length) },
    ],
    actions: [{ label: "New wallet", type: "add" }],
  },
  retail: {
    title: "Retail (Consumers)",
    description: "Consumer catalog, orders, and inventory (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Products", value: String(rows.length) },
      { label: "In stock", value: String(rows.reduce((s, r) => s + (Number(r.stock) || 0), 0)) },
      { label: "SKUs", value: String(new Set(rows.map((r) => r.sku)).size) },
    ],
    actions: [{ label: "Add product", type: "add" }],
  },
  gov: {
    title: "Government (Consumers)",
    description: "Consumer-facing public services: permits, licenses, records (demo data).",
    kind: "table",
    kpis: (rows) => [
      { label: "Applications", value: String(rows.length) },
      { label: "Approved/Issued", value: String(rows.filter((r) => r.status === "Approved" || r.status === "Issued").length) },
      { label: "Submitted", value: String(rows.filter((r) => r.status === "Submitted").length) },
    ],
    actions: [{ label: "New application", type: "add" }],
  },
};
