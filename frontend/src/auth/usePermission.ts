import { useSessionStore } from "./session";

/**
 * Permission checks are UX-only — the backend enforces every permission and
 * every RLS boundary. A hidden control is a courtesy, never a security control.
 *
 * `"*"` in the token's permissions = super-admin (backend rule).
 */
export function hasPermission(
  permissions: string[],
  perm: string
): boolean {
  return permissions.includes(perm) || permissions.includes("*");
}

export function usePermission(perm?: string): boolean {
  const permissions = useSessionStore((s) => s.permissions);
  return !perm || hasPermission(permissions, perm);
}

export function useHasAnyPermission(perms: string[]): boolean {
  const permissions = useSessionStore((s) => s.permissions);
  return perms.some((p) => hasPermission(permissions, p));
}

export function useHasAllPermissions(perms: string[]): boolean {
  const permissions = useSessionStore((s) => s.permissions);
  return perms.every((p) => hasPermission(permissions, p));
}
