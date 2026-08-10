import { useEffect, useRef, useState } from "react";
import { LogOut, Menu, Search } from "lucide-react";
import { logout } from "../auth/keycloak";
import { useSessionStore } from "../auth/session";
import { Avatar, Button } from "../components/ui";
import { StatusPill } from "./StatusPill";
import { useUiStore } from "./uiStore";

export function Topbar() {
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const setCommandPaletteOpen = useUiStore((s) => s.setCommandPaletteOpen);
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const name = useSessionStore((s) => s.name ?? s.preferredUsername);
  const tenantId = useSessionStore((s) => s.tenantId);
  const clear = useSessionStore((s) => s.clear);

  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, []);

  const handleLogout = async () => {
    clear();
    await logout();
  };

  return (
    <header className="flex h-14 shrink-0 items-center gap-3 border-b border-slate-200 bg-white px-4">
      <Button variant="ghost" size="sm" onClick={toggleSidebar} aria-label="Toggle sidebar">
        <Menu className="h-4 w-4" />
      </Button>

      <button
        onClick={() => setCommandPaletteOpen(true)}
        className="flex w-full max-w-xs items-center gap-2 rounded-md border border-slate-300 bg-slate-50 px-3 py-1.5 text-sm text-slate-400 hover:border-slate-400"
      >
        <Search className="h-4 w-4" />
        <span className="flex-1 text-left">Search…</span>
        <kbd className="rounded border border-slate-300 bg-white px-1 text-xs text-slate-500">
          ⌘K
        </kbd>
      </button>

      <div className="ml-auto flex items-center gap-3">
        <StatusPill />
        <div className="relative" ref={menuRef}>
          <button
            onClick={() => setMenuOpen((o) => !o)}
            className="flex items-center gap-2 rounded-full p-1 hover:bg-slate-100"
            aria-label="Account menu"
          >
            <Avatar name={name} />
          </button>
          {menuOpen && (
            <div className="absolute right-0 z-40 mt-1 w-56 rounded-md border border-slate-200 bg-white py-1 shadow-lg">
              <div className="border-b border-slate-100 px-3 py-2">
                <p className="truncate text-sm font-medium text-slate-900">{name ?? "AOS user"}</p>
                {tenantId && (
                  <p className="truncate text-xs text-slate-500">Tenant {tenantId.slice(0, 8)}…</p>
                )}
              </div>
              <button
                onClick={handleLogout}
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-red-600 hover:bg-red-50"
              >
                <LogOut className="h-4 w-4" />
                Sign out
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
}
