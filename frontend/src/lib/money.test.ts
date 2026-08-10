import { describe, expect, it } from "vitest";
import { formatMoney, minorToDecimal, parseMoneyToMinor } from "./money";

describe("money", () => {
  it("renders minor units as decimal strings without floats", () => {
    expect(minorToDecimal(123456)).toBe("1234.56");
    expect(minorToDecimal(5)).toBe("0.05");
    expect(minorToDecimal(0)).toBe("0.00");
    expect(minorToDecimal(-250)).toBe("-2.50");
  });

  it("formats with the currency code", () => {
    expect(formatMoney(123456, "USD")).toBe("1234.56 USD");
  });

  it("parses user input into minor units", () => {
    expect(parseMoneyToMinor("1234.56")).toBe(123456);
    expect(parseMoneyToMinor("1234")).toBe(123400);
    expect(parseMoneyToMinor("1234.5")).toBe(123450);
    expect(parseMoneyToMinor("0.05")).toBe(5);
  });

  it("rejects bad input", () => {
    expect(parseMoneyToMinor("")).toBeNull();
    expect(parseMoneyToMinor("abc")).toBeNull();
    expect(parseMoneyToMinor("1.234")).toBeNull();
    expect(parseMoneyToMinor("1.2.3")).toBeNull();
    expect(parseMoneyToMinor("-5")).toBeNull();
    expect(parseMoneyToMinor("99999999999999")).toBeNull(); // too large
  });
});
