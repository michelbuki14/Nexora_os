// Workforce API — thin wrappers around the HTTP client.
//
// All paths are relative to VITE_API_URL (set to /api/v1/workforce in dev).
// Pagination follows the backend convention: 1-indexed page, page_size items.

import { api } from "./client";
import type {
  CompensationResponse,
  CreateCompensationRequest,
  CreateDepartmentRequest,
  CreateDocumentRequest,
  CreateEmployeeRequest,
  CreateLegalEntityRequest,
  CreateLocationRequest,
  CreatePositionRequest,
  CreateTeamRequest,
  DepartmentResponse,
  DocumentResponse,
  DocumentUrlResponse,
  EmployeeResponse,
  LegalEntityResponse,
  LocationResponse,
  PaginatedResponse,
  PositionResponse,
  TeamResponse,
  UpdateEmployeeStatusRequest,
  UpdateEmploymentRequest,
} from "./types";

const WF = "/api/v1/workforce";

// ---------------------------------------------------------------------------
// Legal entities
// ---------------------------------------------------------------------------

export function listLegalEntities(
  page = 1,
  pageSize = 20,
): Promise<PaginatedResponse<LegalEntityResponse>> {
  return api.get(`${WF}/legal-entities?page=${page}&page_size=${pageSize}`);
}

export function createLegalEntity(
  req: CreateLegalEntityRequest | Record<string, unknown>,
): Promise<LegalEntityResponse> {
  return api.post(`${WF}/legal-entities`, req);
}

// ---------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------

export function listLocations(
  page = 1,
  pageSize = 20,
): Promise<PaginatedResponse<LocationResponse>> {
  return api.get(`${WF}/locations?page=${page}&page_size=${pageSize}`);
}

export function createLocation(
  req: CreateLocationRequest | Record<string, unknown>,
): Promise<LocationResponse> {
  return api.post(`${WF}/locations`, req);
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

export function listDepartments(
  page = 1,
  pageSize = 20,
): Promise<PaginatedResponse<DepartmentResponse>> {
  return api.get(`${WF}/departments?page=${page}&page_size=${pageSize}`);
}

export function createDepartment(
  req: CreateDepartmentRequest | Record<string, unknown>,
): Promise<DepartmentResponse> {
  return api.post(`${WF}/departments`, req);
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

export function listTeams(page = 1, pageSize = 20): Promise<PaginatedResponse<TeamResponse>> {
  return api.get(`${WF}/teams?page=${page}&page_size=${pageSize}`);
}

export function createTeam(
  req: CreateTeamRequest | Record<string, unknown>,
): Promise<TeamResponse> {
  return api.post(`${WF}/teams`, req);
}

// ---------------------------------------------------------------------------
// Positions
// ---------------------------------------------------------------------------

export function listPositions(
  page = 1,
  pageSize = 20,
): Promise<PaginatedResponse<PositionResponse>> {
  return api.get(`${WF}/positions?page=${page}&page_size=${pageSize}`);
}

export function createPosition(
  req: CreatePositionRequest | Record<string, unknown>,
): Promise<PositionResponse> {
  return api.post(`${WF}/positions`, req);
}

// ---------------------------------------------------------------------------
// Employees
// ---------------------------------------------------------------------------

export function listEmployees(
  page = 1,
  pageSize = 20,
): Promise<PaginatedResponse<EmployeeResponse>> {
  return api.get(`${WF}/employees?page=${page}&page_size=${pageSize}`);
}

export function getEmployee(ulid: string): Promise<EmployeeResponse> {
  return api.get(`${WF}/employees/${ulid}`);
}

export function createEmployee(req: CreateEmployeeRequest): Promise<EmployeeResponse> {
  return api.post(`${WF}/employees`, req);
}

export function updateEmployeeStatus(
  ulid: string,
  req: UpdateEmployeeStatusRequest,
): Promise<EmployeeResponse> {
  return api.patch(`${WF}/employees/${ulid}/status`, req);
}

export function updateEmployment(
  ulid: string,
  req: UpdateEmploymentRequest,
): Promise<void> {
  return api.patch(`${WF}/employees/${ulid}/employment`, req);
}

// ---------------------------------------------------------------------------
// Compensation
// ---------------------------------------------------------------------------

export function getCompensation(employeeUlid: string): Promise<CompensationResponse> {
  return api.get(`${WF}/employees/${employeeUlid}/compensation`);
}

export function createCompensation(
  employeeUlid: string,
  req: CreateCompensationRequest,
): Promise<CompensationResponse> {
  return api.post(`${WF}/employees/${employeeUlid}/compensation`, req);
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

export function listDocuments(
  employeeUlid: string,
  page = 1,
  pageSize = 50,
): Promise<PaginatedResponse<DocumentResponse>> {
  return api.get(
    `${WF}/employees/${employeeUlid}/documents?page=${page}&page_size=${pageSize}`,
  );
}

/**
 * Upload a document.
 *
 * The backend expects: JSON metadata line \n <raw file bytes>.
 * We encode the metadata as a JSON line, concatenate with the file bytes,
 * and send the combined buffer as the request body.
 */
export async function uploadDocument(
  employeeUlid: string,
  meta: CreateDocumentRequest,
  file: File,
): Promise<DocumentResponse> {
  const metaLine = JSON.stringify(meta) + "\n";
  const metaBytes = new TextEncoder().encode(metaLine);
  const fileBytes = new Uint8Array(await file.arrayBuffer());

  const combined = new Uint8Array(metaBytes.length + fileBytes.length);
  combined.set(metaBytes, 0);
  combined.set(fileBytes, metaBytes.length);

  return api.raw<DocumentResponse>(
    "POST",
    `${WF}/employees/${employeeUlid}/documents`,
    combined,
    { "Content-Type": "application/octet-stream" },
  );
}

export function getDocumentUrl(
  employeeUlid: string,
  docUlid: string,
): Promise<DocumentUrlResponse> {
  return api.get(`${WF}/employees/${employeeUlid}/documents/${docUlid}/url`);
}
