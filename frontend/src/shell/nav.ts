import {
  Banknote,
  Building2,
  Code2,
  Landmark,
  LayoutGrid,
  LineChart,
  Puzzle,
  Receipt,
  ScrollText,
  Server,
  Settings,
  ShieldCheck,
  ShoppingBag,
  Sparkles,
  Users,
  Wallet,
} from "lucide-react";

// IA from the master prompt §48. Live modules are gated on real token
// permissions; demo-mode modules render seed-backed interactive pages.

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
        description: "Nexora system overview",
      },
    ],
  },
  {
    title: "Business",
    items: [
      {
        label: "Workforce",
        path: "/workforce/employees",
        icon: Users,
        module: "live",
        anyPermission: WORKFORCE_PERMS,
        description: "Employees, org structure, compensation, documents",
      },
      { label: "Payroll", path: "/payroll", icon: Banknote, module: "live" },
      { label: "Finance", path: "/finance", icon: Receipt, module: "live" },
      { label: "Payments", path: "/payments", icon: Wallet, module: "live" },
    ],
  },
  {
    title: "Intelligence",
    items: [
      { label: "Analytics", path: "/analytics", icon: LineChart, module: "live" },
      { label: "AI", path: "/ai", icon: Sparkles, module: "live" },
    ],
  },
  {
    title: "Security",
    items: [
      { label: "Security Center", path: "/security", icon: ShieldCheck, module: "live" },
    ],
  },
  {
    title: "Platform",
    items: [
      { label: "Infrastructure", path: "/infrastructure", icon: Server, module: "live" },
      { label: "Integrations", path: "/integrations", icon: Puzzle, module: "live" },
      { label: "Developer / API", path: "/developer", icon: Code2, module: "live" },
    ],
  },
  {
    title: "Consumers",
    items: [
      { label: "Fintech", path: "/fintech", icon: Wallet, module: "live" },
      { label: "Retail", path: "/retail", icon: ShoppingBag, module: "live" },
      { label: "Government", path: "/gov", icon: Landmark, module: "live" },
    ],
  },
  {
    title: "Administration",
    items: [
      { label: "Organizations", path: "/organizations", icon: Building2, module: "live" },
      { label: "Audit", path: "/audit", icon: ScrollText, module: "live" },
      { label: "Admin", path: "/admin", icon: Settings, module: "live" },
    ],
  },
];

export function flattenNav(): NavItem[] {
  return NAV_GROUPS.flatMap((g) => g.items);
}
