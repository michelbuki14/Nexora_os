import { PageHeader } from "../components/ui";
import { PermissionDenied } from "../components/PermissionDenied";

export function Forbidden() {
  return (
    <div>
      <PageHeader title="Forbidden" description="You don’t have access to this area." />
      <PermissionDenied />
    </div>
  );
}
