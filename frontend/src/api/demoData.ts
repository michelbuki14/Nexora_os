// Seeded in-memory demo data. Activated only when no backend is reachable
// (VITE_API_URL unset / connection refused) so the app is fully usable and
// demoable as a standalone frontend. Mirrors the Rust backend shapes in
// ./types so the demo adapter can satisfy the same call sites.

import type { DocumentResponse } from "./types";
import { SEED_MODULE_TABLES, type ModuleCell } from "./moduleDemoData";

function ulid(): string {
  // Not a real ULID, but stable + unique enough for demo keys.
  return (
    "01" +
    Math.random().toString(36).slice(2, 10).padEnd(12, "0").slice(0, 12) +
    Date.now().toString(36).slice(-4)
  );
}

const now = () => new Date().toISOString();

export interface DemoDepartment {
  ulid: string;
  name: string;
  code: string | null;
  is_active: boolean;
  created_at: string;
}
export interface DemoPosition {
  ulid: string;
  title: string;
  code: string | null;
  job_grade: string | null;
  employment_type: string;
  is_active: boolean;
  created_at: string;
}
export interface DemoLocation {
  ulid: string;
  name: string;
  code: string | null;
  timezone: string;
  is_active: boolean;
  created_at: string;
}
export interface DemoLegalEntity {
  ulid: string;
  name: string;
  registration_number: string | null;
  status: string;
  created_at: string;
}
export interface DemoEmployee {
  ulid: string;
  employee_number: string;
  legal_name: string;
  preferred_name: string | null;
  email: string;
  phone: string | null;
  gender: string | null;
  national_id_last4: string | null;
  status: "onboarding" | "active" | "on_leave" | "terminated";
  hire_date: string | null;
  current_department_ulid: string | null;
  current_position_ulid: string | null;
  current_location_ulid: string | null;
  manager_employee_ulid: string | null;
  created_at: string;
  updated_at: string;
}
export interface DemoCompensation {
  ulid: string;
  gross_amount_minor: number;
  currency_code: string;
  frequency: string;
  effective_date: string;
  created_at: string;
}


class DemoStore {
  departments: DemoDepartment[] = [
    { ulid: "01DEP0000010000acme", name: "Engineering", code: "ENG", is_active: true, created_at: now() },
    { ulid: "01DEP0000020000acme", name: "People Operations", code: "PEO", is_active: true, created_at: now() },
    { ulid: "01DEP0000030000acme", name: "Finance", code: "FIN", is_active: true, created_at: now() },
    { ulid: "01DEP0000040000acme", name: "Sales", code: "SAL", is_active: true, created_at: now() },
  ];
  positions: DemoPosition[] = [
    { ulid: "01POS0000010000acme", title: "Senior Software Engineer", code: "SSE", job_grade: "L5", employment_type: "permanent", is_active: true, created_at: now() },
    { ulid: "01POS0000020000acme", title: "Product Designer", code: "PD", job_grade: "L4", employment_type: "permanent", is_active: true, created_at: now() },
    { ulid: "01POS0000030000acme", title: "People Partner", code: "PP", job_grade: "L4", employment_type: "permanent", is_active: true, created_at: now() },
    { ulid: "01POS0000040000acme", title: "Account Executive", code: "AE", job_grade: "L4", employment_type: "contract", is_active: true, created_at: now() },
  ];
  locations: DemoLocation[] = [
    { ulid: "01LOC0000010000acme", name: "Nairobi HQ", code: "NBO", timezone: "Africa/Nairobi", is_active: true, created_at: now() },
    { ulid: "01LOC0000020000acme", name: "Kinshasa", code: "FIH", timezone: "Africa/Kinshasa", is_active: true, created_at: now() },
    { ulid: "01LOC0000030000acme", name: "Lagos", code: "LOS", timezone: "Africa/Lagos", is_active: true, created_at: now() },
  ];
  legalEntities: DemoLegalEntity[] = [
    { ulid: "01LE000000010000acme", name: "Nexora Africa Ltd", registration_number: "C-123456", status: "active", created_at: now() },
  ];
  employees: DemoEmployee[] = [
    { ulid: "01EMP0000010000acme", employee_number: "EMP-0001", legal_name: "Amara Okafor", preferred_name: "Amara", email: "amara.okafor@nexora.africa", phone: "+254700000001", gender: "female", national_id_last4: "4821", status: "active", hire_date: "2023-02-14", current_department_ulid: "01DEP0000010000acme", current_position_ulid: "01POS0000010000acme", current_location_ulid: "01LOC0000010000acme", manager_employee_ulid: null, created_at: now(), updated_at: now() },
    { ulid: "01EMP0000020000acme", employee_number: "EMP-0002", legal_name: "Jean Mwamba", preferred_name: null, email: "jean.mwamba@nexora.africa", phone: "+243800000002", gender: "male", national_id_last4: "1190", status: "active", hire_date: "2022-09-01", current_department_ulid: "01DEP0000020000acme", current_position_ulid: "01POS0000030000acme", current_location_ulid: "01LOC0000020000acme", manager_employee_ulid: null, created_at: now(), updated_at: now() },
    { ulid: "01EMP0000030000acme", employee_number: "EMP-0003", legal_name: "Fatima Bello", preferred_name: "Fatima", email: "fatima.bello@nexora.africa", phone: "+234800000003", gender: "female", national_id_last4: "7732", status: "on_leave", hire_date: "2024-01-22", current_department_ulid: "01DEP0000040000acme", current_position_ulid: "01POS0000040000acme", current_location_ulid: "01LOC0000030000acme", manager_employee_ulid: "01EMP0000020000acme", created_at: now(), updated_at: now() },
    { ulid: "01EMP0000040000acme", employee_number: "EMP-0004", legal_name: "Kwame Asante", preferred_name: null, email: "kwame.asante@nexora.africa", phone: "+233800000004", gender: "male", national_id_last4: "3365", status: "onboarding", hire_date: "2025-07-10", current_department_ulid: "01DEP0000010000acme", current_position_ulid: "01POS0000020000acme", current_location_ulid: "01LOC0000010000acme", manager_employee_ulid: "01EMP0000010000acme", created_at: now(), updated_at: now() },
  ];
  compensation = new Map<string, DemoCompensation[]>();
  documents = new Map<string, DocumentResponse[]>();
  // Demo-backed "planned" modules (payroll, finance, payments, analytics, ...).
  moduleTables = new Map<string, { columns: { key: string; label: string }[]; rows: Record<string, ModuleCell>[] }>(
    Object.entries(SEED_MODULE_TABLES).map(([k, v]) => [k, JSON.parse(JSON.stringify(v))]),
  );

  page<T>(items: T[]): { items: T[]; total: number; page: number; page_size: number } {
    return { items, total: items.length, page: 1, page_size: items.length || 1 };
  }
}

export const demoStore = new DemoStore();

// Persist the demo store to sessionStorage so created/edited records survive a
// full page reload within the tab (in-memory only would reset to seed on reload).
// By design this does NOT survive across tabs or browser restarts, and is scoped
// to demo mode only — the real backend is the source of truth otherwise.
const PERSIST_KEY = "nexora_demo_v1";

export function persistDemo(): void {
  try {
    const state = {
      employees: demoStore.employees,
      departments: demoStore.departments,
      positions: demoStore.positions,
      locations: demoStore.locations,
      legalEntities: demoStore.legalEntities,
      compensation: Object.fromEntries(demoStore.compensation),
      documents: Object.fromEntries(demoStore.documents),
      moduleTables: Object.fromEntries(demoStore.moduleTables),
    };
    sessionStorage.setItem(PERSIST_KEY, JSON.stringify(state));
  } catch {
    // sessionStorage unavailable (private mode / quota) — degrade silently to in-memory.
  }
}

function hydrateDemo(): void {
  try {
    const raw = sessionStorage.getItem(PERSIST_KEY);
    if (!raw) return;
    const s = JSON.parse(raw) as Partial<{
      employees: DemoEmployee[];
      departments: DemoDepartment[];
      positions: DemoPosition[];
      locations: DemoLocation[];
      legalEntities: DemoLegalEntity[];
      compensation: Record<string, DemoCompensation[]>;
      documents: Record<string, DocumentResponse[]>;
      moduleTables: Record<string, { columns: { key: string; label: string }[]; rows: Record<string, ModuleCell>[] }>;
    }>;
    if (Array.isArray(s.employees)) demoStore.employees = s.employees;
    if (Array.isArray(s.departments)) demoStore.departments = s.departments;
    if (Array.isArray(s.positions)) demoStore.positions = s.positions;
    if (Array.isArray(s.locations)) demoStore.locations = s.locations;
    if (Array.isArray(s.legalEntities)) demoStore.legalEntities = s.legalEntities;
    if (s.compensation && typeof s.compensation === "object") {
      demoStore.compensation = new Map(Object.entries(s.compensation));
    }
    if (s.documents && typeof s.documents === "object") {
      demoStore.documents = new Map(Object.entries(s.documents));
    }
    if (s.moduleTables && typeof s.moduleTables === "object") {
      demoStore.moduleTables = new Map(Object.entries(s.moduleTables));
    }
  } catch {
    // Corrupt/unparseable cache — fall back to seed data.
  }
}

hydrateDemo();

// ---- Module demo API (planned modules in demo mode) -------------------------

export type ModuleTableRow = Record<string, ModuleCell>;

export function getModuleTable(name: string): { columns: { key: string; label: string }[]; rows: ModuleTableRow[] } {
  const t = demoStore.moduleTables.get(name);
  if (t) return { columns: t.columns, rows: t.rows.map((r) => ({ ...r })) };
  // Fallback to seed (e.g. first ever load before hydrate ran)
  const seed = SEED_MODULE_TABLES[name];
  return seed ? { columns: seed.columns, rows: seed.rows.map((r) => ({ ...r })) } : { columns: [], rows: [] };
}

export function toggleModuleRow(name: string, index: number, column: string): void {
  const t = demoStore.moduleTables.get(name);
  if (!t) return;
  const row = t.rows[index];
  if (!row) return;
  row[column] = !(row[column] === true);
  demoStore.moduleTables.set(name, t);
  persistDemo();
}

export function addModuleRow(name: string, row: ModuleTableRow): void {
  const t = demoStore.moduleTables.get(name);
  if (!t) return;
  t.rows.unshift(row);
  demoStore.moduleTables.set(name, t);
  persistDemo();
}

export function runModuleAction(name: string, action: string): void {
  const t = demoStore.moduleTables.get(name);
  if (!t) return;
  if (action === "toggle-all") {
    // Flip all boolean "status" Open -> Resolved (security acknowledgement flow)
    t.rows.forEach((r) => {
      if (r.status === "Open") r.status = "Resolved";
    });
  }
  demoStore.moduleTables.set(name, t);
  persistDemo();
}

export { ulid, now };
