import { ReactNode, useEffect, useState } from "react";
import { decodeJwt, keycloak } from "./keycloak";
import { useSessionStore } from "./session";

/**
 * Bootstraps the Keycloak session. `onLoad: "login-required"` redirects to
 * Keycloak when no session exists, so once `init` resolves there is a session.
 * Tokens live in memory only (never localStorage); refresh is delegated to the
 * Keycloak adapter's silent `updateToken`.
 */
export function AuthProvider({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(false);
  const [failed, setFailed] = useState(false);
  const setSession = useSessionStore((s) => s.setSession);
  const setToken = useSessionStore((s) => s.setToken);

  useEffect(() => {
    let cancelled = false;

    keycloak.onTokenExpired = () => {
      void keycloak.updateToken(30).then((refreshed) => {
        if (refreshed && keycloak.token) setToken(keycloak.token);
      });
    };

    (async () => {
      try {
        // Prefer the current origin so SPA can run on :3000 or :4000 without
        // a rebuild. Fall back to the env value when window is unavailable.
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
        setReady(true);
      } catch (err) {
        if (cancelled) return;
        console.error("Keycloak init failed", err);
        setFailed(true);
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
        <p className="text-sm text-slate-500">Signing you in…</p>
      </div>
    );
  }

  if (failed) {
    return (
      <div className="flex h-screen items-center justify-center bg-slate-50">
        <p className="text-sm text-red-600">
          Authentication could not be initialized. Please check the Keycloak
          server and reload.
        </p>
      </div>
    );
  }

  return <>{children}</>;
}
