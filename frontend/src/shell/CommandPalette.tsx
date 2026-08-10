import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "./uiStore";
import { flattenNav } from "./nav";

/**
 * ⌘K palette — navigation search for Phase 1. scoped to nav items; employee
 * / payroll object search is a tracked follow-up once their backends allow it.
 */
export function CommandPalette() {
  const open = useUiStore((s) => s.commandPaletteOpen);
  const setOpen = useUiStore((s) => s.setCommandPaletteOpen);
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    return flattenNav().filter((item) =>
      q ? item.label.toLowerCase().includes(q) : true
    );
  }, [query]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setIndex(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [open]);

  if (!open) return null;

  const go = (path: string) => {
    setOpen(false);
    navigate(path);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-slate-900/50 p-4 pt-[15vh]"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) setOpen(false);
      }}
    >
      <div className="w-full max-w-md overflow-hidden rounded-lg bg-white shadow-xl">
        <input
          ref={inputRef}
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setIndex(0);
          }}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setIndex((i) => Math.min(i + 1, results.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setIndex((i) => Math.max(i - 1, 0));
            } else if (e.key === "Enter") {
              const hit = results[index];
              if (hit) go(hit.path);
            }
          }}
          placeholder="Search modules…"
          className="w-full border-b border-slate-200 px-4 py-3 text-sm outline-none"
          role="combobox"
          aria-label="Command palette"
        />
        <ul className="max-h-72 overflow-y-auto py-1" role="listbox">
          {results.length === 0 && (
            <li className="px-4 py-3 text-sm text-slate-500">No matches</li>
          )}
          {results.map((item, i) => {
            const Icon = item.icon;
            return (
              <li key={item.path}>
                <button
                  onMouseEnter={() => setIndex(i)}
                  onClick={() => go(item.path)}
                  className={`flex w-full items-center gap-2 px-4 py-2 text-sm ${
                    i === index ? "bg-brand-50 text-brand-800" : "text-slate-700"
                  }`}
                  role="option"
                  aria-selected={i === index}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  <span>{item.label}</span>
                  {item.module === "planned" && (
                    <span className="ml-auto rounded-full bg-slate-100 px-2 py-0.5 text-xs text-slate-500">
                      Planned
                    </span>
                  )}
                </button>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}
