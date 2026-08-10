import { useParams } from "react-router-dom";
import { Badge, Card, PageHeader } from "../components/ui";

const MODULE_BLURBS: Record<string, string> = {
  payroll:
    "Payroll computation, run cycles, and statutory reporting. The backend is documented but not yet implemented — gated behind the production readiness review.",
  finance:
    "General ledger, invoicing, and financial reporting. Backend in planning; no live endpoints yet.",
  payments:
    "Payments orchestration and provider integrations. Backend in planning; regulatory gating applies.",
  analytics:
    "Cross-module reporting and dashboards. Depends on multiple backends that are not live yet.",
  security:
    "Security center, access reviews, and compliance posture. Backend in planning.",
  ai:
    "AOS AI assistant and document intelligence. Backend in planning.",
  infrastructure:
    "Infrastructure health, deployments, and capacity. Backend in planning.",
  integrations:
    "Third-party integrations and webhooks. Backend in planning.",
  developer:
    "API keys, documentation, and developer tooling. Backend in planning.",
  organizations:
    "Tenant and organization management. Backend in planning.",
  audit:
    "Immutable audit trail and event search. Backend in planning (audit-service exists but has no live public API mounted in Phase 1).",
  admin:
    "Platform administration. Backend in planning.",
};

/**
 * Honest "not enabled in this tenant" state. This module has no live backend
 * in Phase 1, so it renders an explanation instead of fake UI or dead forms.
 * It activates automatically when its service lands (plan Phases 2-6).
 */
export function PlannedModule({ module: moduleProp }: { module?: string }) {
  const { module: moduleParam } = useParams<{ module: string }>();
  const name = moduleProp ?? moduleParam ?? "module";
  const blurb = MODULE_BLURBS[name] ?? "This module is part of the AOS platform roadmap.";

  return (
    <div>
      <PageHeader
        title={name[0].toUpperCase() + name.slice(1)}
        description="AOS module"
      />
      <Card className="max-w-2xl p-8">
        <div className="flex items-center gap-2">
          <Badge tone="planned">Planned</Badge>
          <span className="text-sm text-slate-500">Not enabled in this tenant</span>
        </div>
        <p className="mt-4 text-sm leading-6 text-slate-700">{blurb}</p>
        <p className="mt-3 text-xs text-slate-400">
          AOS renders only what its backends can actually serve. This module will
          become available when its service lands and passes the production
          readiness review. You can continue to use the Workforce module today.
        </p>
      </Card>
    </div>
  );
}
