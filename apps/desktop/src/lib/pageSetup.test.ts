import { describe, expect, it } from "vitest";
import { DEFAULT_PAGE_SETUP, formatPageText, getPageSetup } from "./pageSetup";

describe("page setup", () => {
  it("falls back to the default setup for older documents", () => {
    expect(getPageSetup(undefined)).toBe(DEFAULT_PAGE_SETUP);
    expect(
      getPageSetup({
        units: "metric",
        precision: 4,
        fractions: "auto"
      } as never)
    ).toBe(DEFAULT_PAGE_SETUP);
  });

  it("replaces the page token case-insensitively", () => {
    expect(formatPageText("Página {page} de {PAGE}", 7)).toBe(
      "Página 7 de 7"
    );
    expect(formatPageText("Sin marcador", 3)).toBe("Sin marcador");
  });
});
