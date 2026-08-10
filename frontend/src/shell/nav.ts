import {
  Banknote,
  Building2,
  Code2,
  LayoutGrid,
  LineChart,
  Puzzle,
  Receipt,
  ScrollText,
  Server,
  Settings,
  ShieldCheck,
  Users,
  Wallet,
} from "lucide-react";

// IA from the master prompt §48. Live modules are gated on real token
// permissions; planned modules render honest "not enabled in this tenant"
// states (their backends don't exist yet, so no permission can unlock them).

export const WORKFORCE_PERMS = [
  "employee.read",
  "employee.write",
  "employee.export",
  "employee.compensation.read",
  "location.read",
  "department.read",
  "team.read",
  "position.read",
  "legal_entity.read",
];

export interface NavItem {
  label: string;
  path: string;
  icon: typeof Users;
  module: "live" | "planned";
  /** Show if the caller holds ANY of these permissions (live modules only). */
  anyPermission?: string[];
  description?: string;
}

export interface NavGroup {
  title: string;
  items: NavItem[];
}

export const NAV_GROUPS: NavGroup[] = [
  {
    title: "Overview",
    items: [
      {
        label: "Dashboard",
        path: "/",
        icon: LayoutGrid,
        module: "live",
        description: "AOS system overview",
      },
    ],
  },
  {
    title: "Business",
    items: [
      {
        label: "Workforce",
        path: "/workforce",
        icon: Users,
        module: "live",
        anyPermission: WORKFORCE_PERMS,
        description: "Employees, org structure, compensation, documents",
      },
      { label: "Payroll", path: "/payroll", icon: Banknote, module: "planned" },
      { label: "Finance", path: "/finance", icon: Receipt, module: "planned" },
      { label: "Payments", path: "/payments", icon: Wallet, module: "planned" },
    ],
  },
  {
    title: "Intelligence",
    items: [
      { label: "Analytics", path: "/analytics", icon: LineChart, module: "planned" },
    ],
  },
  {
    title: "Security",
    items: [
      { label: "Security Center", path: "/security", icon: ShieldCheck, module: "planned" },
    ],
  },
  {
    title: "Platform",
    items: [
      { label: "Infrastructure", path: "/infrastructure", icon: Server, module: "planned" },
      { label: "Integrations", path: "/integrations", icon: Puzzle, module: "planned" },
      { label: "Developer / API", path: "/developer", icon: Code2, module: "planned" },
    ],
  },
  {
    title: "Administration",
    items: [
      { label: "Organizations", path: "/organizations", icon: Building2, module: "planned" },
      { label: "Audit", path: "/audit", icon: ScrollText, module: "planned" },
      { label: "Admin", path: "/admin", icon: Settings, module: "planned" },
    ],
  },
];

export function flattenNav(): NavItem[] {
  return NAV_GROUPS.flatMap((g) => g.items);
}
