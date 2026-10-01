import { NavLink } from "react-router-dom";
import { useHasAnyPermission } from "../auth/usePermission";
import { Badge } from "../components/ui";
import { NAV_GROUPS } from "./nav";
import { useUiStore } from "./uiStore";

export function Sidebar({ onNavigate }: { onNavigate?: () => void } = {}) {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  return (
    <aside
      className={`sidebar-surface flex h-full w-64 max-w-[82vw] shrink-0 flex-col border-r border-slate-800 ${
        collapsed ? "md:w-14" : ""
      }`}
    >
      <div className={`flex h-14 items-center ${collapsed ? "md:justify-center" : "px-4"}`}>
        <span className="text-sm font-bold tracking-tight text-white">Nexora</span>
        {!collapsed && <span className="ml-1 text-sm text-slate-300">OS</span>}
      </div>
      <nav className="flex-1 overflow-y-auto px-2 py-2" aria-label="Primary">
        {NAV_GROUPS.map((group) => (
          <Group
            key={group.title}
            title={group.title}
            items={group.items}
            collapsed={collapsed}
            onNavigate={onNavigate}
          />
        ))}
      </nav>
    </aside>
  );
}

function Group({
  title,
  items,
  collapsed,
  onNavigate,
}: {
  title: string;
  items: (typeof NAV_GROUPS)[number]["items"];
  collapsed: boolean;
  onNavigate?: () => void;
}) {
  // Filter live items by permission; planned items always render (honest stubs).
  const live = items.filter((i) => i.module === "live" && i.anyPermission);
  const hasLive = useHasAnyPermission(live.flatMap((i) => i.anyPermission ?? []));
  const shown = items.filter((i) => i.module !== "live" || !i.anyPermission || hasLive);

  if (shown.length === 0) return null;
  return (
    <div className="mb-4">
      {!collapsed && (
        <p className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide sidebar-section">
          {title}
        </p>
      )}
      <ul className="space-y-0.5">
        {shown.map((item) => {
          const Icon = item.icon;
          return (
            <li key={item.path}>
              <NavLink
                to={item.path}
                end={item.path === "/"}
                title={collapsed ? item.label : undefined}
                onClick={onNavigate}
                className={({ isActive }) =>
                  `flex min-h-[44px] items-center gap-2 rounded-md px-2 py-2 text-sm font-medium transition-colors ${
                    collapsed ? "md:justify-center" : ""
                  } ${
                    isActive
                      ? "sidebar-link-active"
                      : "sidebar-link hover:bg-[#1C2030]"
                  }`
                }
              >
                <Icon className="h-4 w-4 shrink-0" />
                {!collapsed && (
                  <>
                    <span className="flex-1 truncate">{item.label}</span>
                    {item.module === "planned" && (
                      <Badge tone="planned">Planned</Badge>
                    )}
                  </>
                )}
              </NavLink>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
