// Money helpers for the workforce compensation contract.
//
// `gross_amount_minor` is an i64 in minor currency units (centimes/cents).
// Backend contract rule: never let a float near an amount. Display strings are
// built with integer math; parsing goes the other way with string math only.

/** Convert minor units to a "1234.56" display string (integer math, no floats). */
export function minorToDecimal(minor: number): string {
  const sign = minor < 0 ? "-" : "";
  const abs = Math.abs(minor);
  const whole = Math.floor(abs / 100);
  const frac = String(abs % 100).padStart(2, "0");
  return `${sign}${whole}.${frac}`;
}

/** "1234.56 USD" — display only. */
export function formatMoney(minor: number, currency: string): string {
  return `${minorToDecimal(minor)} ${currency}`;
}

/**
 * Parse a user-entered amount ("1234" or "1234.5" or "1234.56") into minor
 * units. Returns null for anything non-numeric, more than 2 decimals, or a
 * value too large to be safe. Never touches a float for the money itself.
 */
export function parseMoneyToMinor(input: string): number | null {
  const trimmed = input.trim();
  if (trimmed === "") return null;
  const m = /^(\d{1,12})(?:\.(\d{1,2}))?$/.exec(trimmed);
  if (!m) return null;
  const whole = Number(m[1]);
  const frac = m[2] ? m[2].padEnd(2, "0") : "00";
  if (!Number.isSafeInteger(whole)) return null;
  const minor = whole * 100 + Number(frac);
  return Number.isSafeInteger(minor) ? minor : null;
}
