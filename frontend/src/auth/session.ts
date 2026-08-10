import { create } from "zustand";
import type { SessionClaims } from "./keycloak";

// Session state mirrors the backend `AuthContext` (tenant/org/roles/perms).
// This is UI state only — the backend remains authoritative for authorization.

export interface Session {
  authenticated: boolean;
  token: string | null;
  userId: string | null;
  tenantId: string | null;
  orgId: string | null;
  roles: string[];
  permissions: string[];
  isSystem: boolean;
  preferredUsername: string | null;
  email: string | null;
  name: string | null;
  setSession: (token: string, claims: SessionClaims) => void;
  setToken: (token: string) => void;
  clear: () => void;
}

function isSystemRole(roles: string[]): boolean {
  // Matches backend is_system derivation (auth_middleware.rs).
  return roles.includes("PLATFORM_ADMIN") || roles.includes("SUPER_ADMIN");
}

export const useSessionStore = create<Session>((set) => ({
  authenticated: false,
  token: null,
  userId: null,
  tenantId: null,
  orgId: null,
  roles: [],
  permissions: [],
  isSystem: false,
  preferredUsername: null,
  email: null,
  name: null,

  setSession: (token, claims) =>
    set({
      authenticated: true,
      token,
      userId: claims.sub ?? null,
      tenantId: claims.tenant_id ?? null,
      orgId: claims.org_id ?? null,
      roles: claims.roles ?? [],
      permissions: claims.permissions ?? [],
      isSystem: isSystemRole(claims.roles ?? []),
      preferredUsername: claims.preferred_username ?? null,
      email: claims.email ?? null,
      name: claims.name ?? null,
    }),

  setToken: (token) => set({ token }),

  clear: () =>
    set({
      authenticated: false,
      token: null,
      userId: null,
      tenantId: null,
      orgId: null,
      roles: [],
      permissions: [],
      isSystem: false,
      preferredUsername: null,
      email: null,
      name: null,
    }),
}));
