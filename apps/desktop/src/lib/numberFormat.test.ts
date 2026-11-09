import { describe, expect, it } from "vitest";
import {
  DEFAULT_NUMBER_FORMAT,
  formatPreviewNumber,
  formatPreviewRational,
  normalizeNumberFormat
} from "./numberFormat";

describe("number format", () => {
  it("normalizes legacy and invalid settings", () => {
    expect(normalizeNumberFormat({})).toEqual(DEFAULT_NUMBER_FORMAT);
    expect(
      normalizeNumberFormat({
        notation: "unknown",
        precision: 99,
        fractions: "decimal",
        trimZeros: false
      })
    ).toEqual({
      notation: "auto",
      precision: 12,
      fractions: "decimal",
      trimZeros: false
    });
  });

  it("formats automatic notation with four decimals by default", () => {
    const format = DEFAULT_NUMBER_FORMAT;
    expect(formatPreviewNumber(6627.6535433071, format)).toBe("6627.6535");
    expect(formatPreviewNumber(0.000000009771908, format)).toBe("9.7719e-9");
    expect(formatPreviewNumber(1234567890, format)).toBe("1234567890");
    expect(formatPreviewNumber(1234567890123, format)).toBe("1.2346e12");
    expect(formatPreviewNumber(2.3, format)).toBe("2.3");
  });

  it("respects fixed, scientific and engineering notation", () => {
    expect(
      formatPreviewNumber(2.3, {
        ...DEFAULT_NUMBER_FORMAT,
        notation: "fixed",
        precision: 2,
        trimZeros: false
      })
    ).toBe("2.30");
    expect(
      formatPreviewNumber(12345, {
        ...DEFAULT_NUMBER_FORMAT,
        notation: "scientific",
        precision: 3
      })
    ).toBe("1.234e4");
    expect(
      formatPreviewNumber(0.0012, {
        ...DEFAULT_NUMBER_FORMAT,
        notation: "engineering",
        precision: 2
      })
    ).toBe("1.2e-3");
  });

  it("chooses fraction or decimal output", () => {
    expect(formatPreviewRational(1, 3, DEFAULT_NUMBER_FORMAT)).toBe("1/3");
    expect(
      formatPreviewRational(1, 3, {
        ...DEFAULT_NUMBER_FORMAT,
        fractions: "decimal"
      })
    ).toBe("0.3333");
  });
});
