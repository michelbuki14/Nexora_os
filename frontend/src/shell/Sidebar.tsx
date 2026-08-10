import { NavLink } from "react-router-dom";
import { useHasAnyPermission } from "../auth/usePermission";
import { Badge } from "../components/ui";
import { NAV_GROUPS } from "./nav";
import { useUiStore } from "./uiStore";

export function Sidebar() {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  return (
    <aside
      className={`flex h-full shrink-0 flex-col border-r border-slate-200 bg-white transition-[width] ${
        collapsed ? "w-14" : "w-64"
      }`}
    >
      <div className={`flex h-14 items-center ${collapsed ? "justify-center" : "px-4"}`}>
        <span className="text-sm font-bold tracking-tight text-slate-900">AOS</span>
        {!collapsed && <span className="ml-1 text-sm text-slate-400">Africa OS</span>}
      </div>
      <nav className="flex-1 overflow-y-auto px-2 py-2">
        {NAV_GROUPS.map((group) => (
          <Group
            key={group.title}
            title={group.title}
            items={group.items}
            collapsed={collapsed}
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
}: {
  title: string;
  items: (typeof NAV_GROUPS)[number]["items"];
  collapsed: boolean;
}) {
  // Filter live items by permission; planned items always render (honest stubs).
  const live = items.filter(
    (i) => i.module === "live" && i.anyPermission
  );
  const hasLive = useHasAnyPermission(live.flatMap((i) => i.anyPermission ?? []));
  const shown = items.filter(
    (i) => i.module !== "live" || !i.anyPermission || hasLive
  );

  if (shown.length === 0) return null;
  return (
    <div className="mb-4">
      {!collapsed && (
        <p className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-slate-400">
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
                className={({ isActive }) =>
                  `flex items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium transition-colors ${
                    collapsed ? "justify-center" : ""
                  } ${
                    isActive
                      ? "bg-brand-50 text-brand-700"
                      : "text-slate-600 hover:bg-slate-100 hover:text-slate-900"
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
