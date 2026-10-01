// Demo adapter — in-memory implementation of the workforce API surface.
// Used transparently by ./client when no backend is reachable, so the entire
// Workforce module (directory, profile, comp, documents) works end-to-end as a
// standalone frontend. Keeps the same response shapes as ./types.

import { demoStore, ulid, now, persistDemo } from "./demoData";
import type {
  CompensationResponse,
  CreateCompensationRequest,
  CreateDocumentRequest,
  CreateEmployeeRequest,
  DepartmentResponse,
  DocumentResponse,
  EmployeeResponse,
  LocationResponse,
  PaginatedResponse,
  PositionResponse,
} from "./types";

// A tiny artificial latency so loading states are exercised in the UI.
const delay = (ms = 180) => new Promise((r) => setTimeout(r, ms));

function paginate<T>(items: T[]): PaginatedResponse<T> {
  return { items, total: items.length, page: 1, page_size: items.length || 1 };
}

function toEmployeeResponse(e: typeof demoStore.employees[number]): EmployeeResponse {
  return {
    ulid: e.ulid,
    employee_number: e.employee_number,
    legal_name: e.legal_name,
    preferred_name: e.preferred_name,
    email: e.email,
    phone: e.phone,
    gender: e.gender,
    national_id_last4: e.national_id_last4,
    status: e.status,
    hire_date: e.hire_date,
    current_department_ulid: e.current_department_ulid,
    current_position_ulid: e.current_position_ulid,
    current_location_ulid: e.current_location_ulid,
    manager_employee_ulid: e.manager_employee_ulid,
    created_at: e.created_at,
    updated_at: e.updated_at,
  };
}

export const demoAdapter = {
  // Legal entities
  listLegalEntities: async (): Promise<PaginatedResponse<{ ulid: string; name: string; registration_number: string | null; status: string; created_at: string }>> => {
    await delay();
    return paginate(demoStore.legalEntities);
  },

  // Locations
  listLocations: async (): Promise<PaginatedResponse<LocationResponse>> => {
    await delay();
    return paginate(demoStore.locations);
  },

  // Departments
  listDepartments: async (): Promise<PaginatedResponse<DepartmentResponse>> => {
    await delay();
    return paginate(demoStore.departments);
  },

  // Positions
  listPositions: async (): Promise<PaginatedResponse<PositionResponse>> => {
    await delay();
    return paginate(demoStore.positions);
  },

  // Employees
  listEmployees: async (): Promise<PaginatedResponse<EmployeeResponse>> => {
    await delay();
    return paginate(demoStore.employees.map(toEmployeeResponse));
  },

  getEmployee: async (id: string): Promise<EmployeeResponse> => {
    await delay();
    const e = demoStore.employees.find((x) => x.ulid === id);
    if (!e) throw new DemoError(404, "Employee not found");
    return toEmployeeResponse(e);
  },

  createEmployee: async (req: CreateEmployeeRequest): Promise<EmployeeResponse> => {
    await delay();
    const e = {
      ulid: ulid(),
      employee_number: req.employee_number,
      legal_name: req.legal_name,
      preferred_name: req.preferred_name ?? null,
      email: req.email,
      phone: req.phone ?? null,
      gender: req.gender ?? null,
      national_id_last4: req.national_id ? req.national_id.slice(-4) : null,
      status: "active" as const,
      hire_date: req.hire_date ?? null,
      current_department_ulid: req.department_ulid ?? null,
      current_position_ulid: req.position_ulid ?? null,
      current_location_ulid: req.location_ulid ?? null,
      manager_employee_ulid: req.manager_employee_ulid ?? null,
      created_at: now(),
      updated_at: now(),
    };
    demoStore.employees.unshift(e);
    persistDemo();
    return toEmployeeResponse(e);
  },

  updateEmployeeStatus: async (
    id: string,
    req: { status: EmployeeResponse["status"]; termination_date?: string | null; termination_reason?: string | null },
  ): Promise<EmployeeResponse> => {
    await delay();
    const e = demoStore.employees.find((x) => x.ulid === id);
    if (!e) throw new DemoError(404, "Employee not found");
    e.status = req.status;
    e.updated_at = now();
    persistDemo();
    return toEmployeeResponse(e);
  },

  updateEmployment: async (
    id: string,
    req: { position_ulid?: string | null; department_ulid?: string | null; location_ulid?: string | null; employment_type?: string | null; effective_date: string; change_reason?: string | null },
  ): Promise<void> => {
    await delay();
    const e = demoStore.employees.find((x) => x.ulid === id);
    if (!e) throw new DemoError(404, "Employee not found");
    if (req.department_ulid !== undefined) e.current_department_ulid = req.department_ulid;
    if (req.position_ulid !== undefined) e.current_position_ulid = req.position_ulid;
    if (req.location_ulid !== undefined) e.current_location_ulid = req.location_ulid;
    e.updated_at = now();
    persistDemo();
    return;
  },

  // Compensation
  getCompensation: async (employeeUlid: string): Promise<CompensationResponse> => {
    await delay();
    const list = demoStore.compensation.get(employeeUlid) ?? [];
    if (list.length === 0) {
      const seed: CompensationResponse = {
        ulid: ulid(),
        gross_amount_minor: 9_200_00,
        currency_code: "USD",
        frequency: "monthly",
        effective_date: "2024-01-01",
        created_at: now(),
      };
      demoStore.compensation.set(employeeUlid, [seed]);
      persistDemo();
      return seed;
    }
    return list[list.length - 1];
  },

  createCompensation: async (
    employeeUlid: string,
    req: CreateCompensationRequest,
  ): Promise<CompensationResponse> => {
    await delay();
    const c: CompensationResponse = {
      ulid: ulid(),
      gross_amount_minor: req.gross_amount_minor,
      currency_code: req.currency_code,
      frequency: req.frequency,
      effective_date: req.effective_date,
      created_at: now(),
    };
    const list = demoStore.compensation.get(employeeUlid) ?? [];
    list.push(c);
    demoStore.compensation.set(employeeUlid, list);
    persistDemo();
    return c;
  },

  // Documents
  listDocuments: async (employeeUlid: string): Promise<PaginatedResponse<DocumentResponse>> => {
    await delay();
    return paginate(demoStore.documents.get(employeeUlid) ?? []);
  },

  uploadDocument: async (
    employeeUlid: string,
    meta: CreateDocumentRequest,
  ): Promise<DocumentResponse> => {
    await delay();
    const d: DocumentResponse = {
      ulid: ulid(),
      doc_type: meta.doc_type,
      filename: meta.filename,
      mime_type: meta.mime_type,
      size_bytes: meta.size_bytes ?? null,
      is_active: true,
      created_at: now(),
    };
    const list = demoStore.documents.get(employeeUlid) ?? [];
    list.push(d);
    demoStore.documents.set(employeeUlid, list);
    persistDemo();
    return d;
  },

  getDocumentUrl: async (): Promise<{ presigned_url: string; expires_in_secs: number }> => {
    await delay();
    // No object storage in demo — return a placeholder data URL the UI treats as unavailable.
    return { presigned_url: "", expires_in_secs: 0 };
  },
};

export class DemoError extends Error {
  constructor(public readonly status: number, message: string) {
    super(message);
    this.name = "DemoError";
  }
}
