/**
 * Shared dashboard types.
 * Single source of truth for data structures used across dashboard components.
 */

/** Payroll run data for timeline visualization */
export interface PayrollRun {
  id: string;
  period: string;
  status: "draft" | "confirmed" | "paid";
  amount: number;
}

/** Country/region tenant distribution data */
export interface CountryData {
  country: string;
  count: number;
  color: string;
}