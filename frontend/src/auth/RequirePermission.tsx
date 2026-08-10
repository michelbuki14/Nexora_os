import { ReactNode } from "react";
import { PermissionDenied } from "../components/PermissionDenied";
import { usePermission } from "./usePermission";

/** Route-element guard: renders PermissionDenied when the caller lacks a permission. */
export function RequirePermission({
  perm,
  children,
}: {
  perm: string;
  children: ReactNode;
}) {
  const allowed = usePermission(perm);
  if (!allowed) return <PermissionDenied permission={perm} />;
  return <>{children}</>;
}
