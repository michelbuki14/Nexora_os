// HTTP client shared by all API modules.
//
// All requests attach the Keycloak bearer token. A 401 does not auto-refresh
// here — the AuthProvider handles token renewal; callers surface the error.
//
// ApiError carries the HTTP status so callers can branch on 403/404 without
// parsing strings. errorMessage() gives a single display-safe string.

import { getValidToken } from "../auth/keycloak";

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export const API_BASE = import.meta.env.VITE_API_URL ?? "";
const BASE_URL = API_BASE;

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  headers?: Record<string, string>,
): Promise<T> {
  const token = await getValidToken();
  const reqHeaders: Record<string, string> = {
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(body !== undefined && !(body instanceof Uint8Array)
      ? { "Content-Type": "application/json" }
      : {}),
    ...headers,
  };
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers: reqHeaders,
    body:
      body === undefined
        ? undefined
        : body instanceof Uint8Array
          ? body
          : JSON.stringify(body),
  });
  if (!res.ok) {
    let message = res.statusText;
    try {
      const json = await res.json();
      message = json.message ?? json.error ?? message;
    } catch {
      // ignore parse error, keep statusText
    }
    throw new ApiError(res.status, message);
  }
  // 204 No Content — return undefined cast to T
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export const api = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body?: unknown) => request<T>("POST", path, body),
  patch: <T>(path: string, body?: unknown) => request<T>("PATCH", path, body),
  delete: <T>(path: string) => request<T>("DELETE", path),
  /** Raw request for custom headers (e.g. document upload). */
  raw: <T>(
    method: string,
    path: string,
    body?: unknown,
    headers?: Record<string, string>,
  ) => request<T>(method, path, body, headers),
};

/** Human-readable error string safe to show in the UI. */
export function errorMessage(err: unknown): string {
  if (err instanceof ApiError) return err.message;
  if (err instanceof Error) return err.message;
  return String(err);
}
