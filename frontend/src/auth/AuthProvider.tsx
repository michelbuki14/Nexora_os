import { ReactNode, useEffect, useState } from "react";
import { decodeJwt } from "./keycloak";
import { useSessionStore } from "./session";

/**
 * Bootstraps authentication. When Keycloak is unavailable (not running,
 * network error), we enter demo mode immediately so the app renders without
 * blocking on a missing auth server.
 *
 * Tokens live in memory only (never localStorage).
 */
export function AuthProvider({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(false);
  const setSession = useSessionStore((s) => s.setSession);
  const setToken = useSessionStore((s) => s.setToken);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        // Check if Keycloak server is reachable before attempting init
        const kcUrl = import.meta.env.VITE_KEYCLOAK_URL || "http://localhost:9090";
        const realm = import.meta.env.VITE_KEYCLOAK_REALM || "nexora_os";

        // Quick reachability check with short timeout
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 3000);

        try {
          await fetch(`${kcUrl}/realms/${realm}`, {
            method: "HEAD",
            signal: controller.signal,
          });
        } catch {
          // Keycloak not reachable — enter demo mode with full perms
          clearTimeout(timeout);
          if (cancelled) return;
          setSession("", {
            sub: "demo-user",
            tenant_id: "demo-tenant",
            org_id: "demo-org",
            roles: ["PLATFORM_ADMIN"],
            permissions: ["*"],
            preferred_username: "demo",
            email: "demo@nexora.africa",
            name: "Demo Admin",
          }, true);
          setReady(true);
          return;
        }
        clearTimeout(timeout);

        // Keycloak IS reachable — proceed with normal init
        const { keycloak } = await import("./keycloak");
        const redirectUri =
          typeof window !== "undefined"
            ? `${window.location.origin}/callback`
            : import.meta.env.VITE_KEYCLOAK_REDIRECT_URI;

        const authenticated = await keycloak.init({
          onLoad: "login-required",
          pkceMethod: "S256",
          redirectUri,
        });

        if (cancelled) return;

        if (authenticated && keycloak.token) {
          setSession(keycloak.token, decodeJwt(keycloak.token));
        }

        // Set up token refresh
        keycloak.onTokenExpired = () => {
          void keycloak.updateToken(30).then((refreshed) => {
            if (refreshed && keycloak.token) setToken(keycloak.token);
          });
        };

        setReady(true);
      } catch {
        if (cancelled) return;
        // Any error — enter demo mode with full perms
        setSession("", {
          sub: "demo-user",
          tenant_id: "demo-tenant",
          org_id: "demo-org",
          roles: ["PLATFORM_ADMIN"],
          permissions: ["*"],
          preferred_username: "demo",
          email: "demo@nexora.africa",
          name: "Demo Admin",
        }, true);
        setReady(true);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [setSession, setToken]);

  if (!ready) {
    return (
      <div className="flex h-screen items-center justify-center bg-slate-50">
        <p className="text-sm text-secondary">Loading…</p>
      </div>
    );
  }

  return <>{children}</>;
}
