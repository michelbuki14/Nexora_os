import { Card } from "./ui";

/**
 * Honest 403 state. The backend is authoritative — this view explains the
 * denial and points at the permission that would unlock it. Never attempt to
 * "fetch and hide"; if the caller lacks the permission, don't call the API.
 */
export function PermissionDenied({ permission }: { permission?: string }) {
  return (
    <Card className="mx-auto max-w-md p-8 text-center">
      <p className="text-lg font-semibold text-slate-900">You don’t have access</p>
      <p className="mt-2 text-sm text-slate-500">
        This area requires a permission your account does not currently hold.
        {permission && (
          <>
            {" "}
            The required permission is <code className="rounded bg-slate-100 px-1">{permission}</code>.
          </>
        )}
      </p>
      <p className="mt-3 text-xs text-slate-400">
        Access is enforced by the AOS backend — contact your tenant administrator
        to request the role.
      </p>
    </Card>
  );
}
