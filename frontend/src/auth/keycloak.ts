// Keycloak OIDC bootstrap for the AOS public client.
//
// Matches the backend contract (services/common/src/auth_middleware.rs):
// the token carries `sub`, `tenant_id`, `org_id` (ULIDs), `roles[]`,
// `permissions[]`. The backend validates `aud` == "aos-api"; if the
// dev-portal token is rejected, add a Keycloak client "audience" mapper
// (plan Prerequisite — realm config, no Rust change).

import Keycloak from "keycloak-js";

export interface SessionClaims {
  sub?: string;
  tenant_id?: string;
  org_id?: string;
  roles?: string[];
  permissions?: string[];
  preferred_username?: string;
  email?: string;
  name?: string;
}

function b64urlDecode(s: string): string {
  const b64 = s.replace(/-/g, "+").replace(/_/g, "/");
  const padded = b64.padEnd(b64.length + ((4 - (b64.length % 4)) % 4), "=");
  // atob is fine for JWT payloads (UTF-8 handled via decodeURIComponent).
  const binary = atob(padded);
  const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

/** Decode the JWT payload without verifying — verification happens server-side. */
export function decodeJwt(token: string): SessionClaims {
  const payload = token.split(".")[1];
  if (!payload) return {};
  try {
    return JSON.parse(b64urlDecode(payload)) as SessionClaims;
  } catch {
    return {};
  }
}

export const keycloak = new Keycloak({
  url: import.meta.env.VITE_KEYCLOAK_URL,
  realm: import.meta.env.VITE_KEYCLOAK_REALM,
  clientId: import.meta.env.VITE_KEYCLOAK_CLIENT_ID,
});

/** Return a fresh access token, silently refreshing if near expiry. */
export async function getValidToken(minValidity = 30): Promise<string | null> {
  if (!keycloak.token) return null;
  if (keycloak.isTokenExpired(minValidity)) {
    try {
      await keycloak.updateToken(minValidity);
    } catch {
      return null;
    }
  }
  return keycloak.token;
}

export async function logout(): Promise<void> {
  await keycloak.logout({ redirectUri: window.location.origin });
}
