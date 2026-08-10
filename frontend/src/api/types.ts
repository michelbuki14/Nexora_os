// API response and request types mirroring the Rust backend models.
// These are plain TypeScript — no runtime validation; Zod schemas
// live in the components that own the form.

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

export type EmployeeStatus = "onboarding" | "active" | "on_leave" | "terminated";
export type EmploymentType = "permanent" | "contract" | "intern" | "consultant";
export type CompensationFrequency = "monthly" | "annual" | "hourly" | "weekly";
export type DocType =
  | "contract"
  | "id_document"
  | "certificate"
  | "offer_letter"
  | "payslip"
  | "other";

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

// ---------------------------------------------------------------------------
// Legal entities
// ---------------------------------------------------------------------------

export interface CreateLegalEntityRequest {
  name: string;
  registration_number?: string | null;
  country_ulid?: string | null;
}

export interface LegalEntityResponse {
  ulid: string;
  name: string;
  registration_number?: string | null;
  status: string;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------

export interface CreateLocationRequest {
  legal_entity_ulid: string;
  name: string;
  code?: string | null;
  timezone?: string | null;
}

export interface LocationResponse {
  ulid: string;
  name: string;
  code?: string | null;
  timezone: string;
  is_active: boolean;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

export interface CreateDepartmentRequest {
  name: string;
  code?: string | null;
  parent_department_ulid?: string | null;
  location_ulid?: string | null;
}

export interface DepartmentResponse {
  ulid: string;
  name: string;
  code?: string | null;
  is_active: boolean;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

export interface CreateTeamRequest {
  department_ulid: string;
  name: string;
  code?: string | null;
}

export interface TeamResponse {
  ulid: string;
  name: string;
  code?: string | null;
  is_active: boolean;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Positions
// ---------------------------------------------------------------------------

export interface CreatePositionRequest {
  title: string;
  code?: string | null;
  department_ulid?: string | null;
  job_grade?: string | null;
  description?: string | null;
  responsibilities?: string | null;
  employment_type?: EmploymentType | null;
}

export interface PositionResponse {
  ulid: string;
  title: string;
  code?: string | null;
  job_grade?: string | null;
  employment_type: string;
  is_active: boolean;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Employees
// ---------------------------------------------------------------------------

export interface CreateEmployeeRequest {
  employee_number: string;
  legal_name: string;
  preferred_name?: string | null;
  email: string;
  phone?: string | null;
  date_of_birth?: string | null;
  gender?: string | null;
  national_id?: string | null;
  hire_date?: string | null;
  department_ulid?: string | null;
  position_ulid?: string | null;
  location_ulid?: string | null;
  manager_employee_ulid?: string | null;
  employment_type?: EmploymentType | null;
}

export interface EmployeeResponse {
  ulid: string;
  employee_number: string;
  legal_name: string;
  preferred_name?: string | null;
  email: string;
  phone?: string | null;
  gender?: string | null;
  national_id_last4?: string | null;
  status: EmployeeStatus;
  hire_date?: string | null;
  current_department_ulid?: string | null;
  current_position_ulid?: string | null;
  current_location_ulid?: string | null;
  manager_employee_ulid?: string | null;
  created_at: string;
  updated_at: string;
}

export interface UpdateEmployeeStatusRequest {
  status: EmployeeStatus;
  termination_date?: string | null;
  termination_reason?: string | null;
}

export interface UpdateEmploymentRequest {
  position_ulid?: string | null;
  department_ulid?: string | null;
  location_ulid?: string | null;
  employment_type?: EmploymentType | null;
  effective_date: string;
  change_reason?: string | null;
}

// ---------------------------------------------------------------------------
// Compensation
// ---------------------------------------------------------------------------

export interface CreateCompensationRequest {
  gross_amount_minor: number;
  currency_code: string;
  frequency: CompensationFrequency;
  effective_date: string;
  change_reason?: string | null;
}

export interface CompensationResponse {
  ulid: string;
  gross_amount_minor: number;
  currency_code: string;
  frequency: string;
  effective_date: string;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

export interface CreateDocumentRequest {
  doc_type: DocType;
  filename: string;
  mime_type: string;
  size_bytes?: number | null;
  sha256?: string | null;
}

export interface DocumentResponse {
  ulid: string;
  doc_type: string;
  filename: string;
  mime_type: string;
  size_bytes?: number | null;
  is_active: boolean;
  created_at: string;
}

export interface DocumentUrlResponse {
  presigned_url: string;
  expires_in_secs: number;
}
