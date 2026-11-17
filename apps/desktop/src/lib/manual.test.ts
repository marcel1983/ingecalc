import { describe, expect, it } from "vitest";
import { getManualContent, searchManual } from "./manual";

describe("in-app manual", () => {
  it("loads Spanish and English content", () => {
    const spanish = getManualContent("es");
    const english = getManualContent("en");

    expect(spanish.title).toBe("Manual de IngeCalc");
    expect(english.title).toBe("IngeCalc Manual");
    expect(spanish.sections.length).toBeGreaterThanOrEqual(15);
    expect(english.sections.length).toBe(spanish.sections.length);
    expect(spanish.sections.map((section) => section.id)).toContain("unidades");
    expect(spanish.sections.map((section) => section.id)).toContain("exportar");
  });

  it("falls back to English for unsupported manual languages", () => {
    expect(getManualContent("fr").title).toBe("IngeCalc Manual");
    expect(getManualContent("ar").title).toBe("IngeCalc Manual");
  });

  it("searches ignoring accents and case", () => {
    const content = getManualContent("es");

    expect(searchManual(content, "LATEX").some((section) => section.id === "exportar")).toBe(
      true
    );
    expect(searchManual(content, "unidades").some((section) => section.id === "unidades")).toBe(
      true
    );
    expect(searchManual(content, "F1").some((section) => section.id === "atajos")).toBe(
      true
    );
    expect(searchManual(content, "símbolo").some((section) => section.id === "matematica")).toBe(
      true
    );
  });

  it("documents the 0.1.5 UX features in Spanish and English", () => {
    for (const language of ["es", "en"]) {
      const content = getManualContent(language);
      const document = content.sections.find((section) => section.id === "documento");
      const shortcuts = content.sections.find((section) => section.id === "atajos");
      const problems = content.sections.find((section) => section.id === "problemas");
      const exportSection = content.sections.find((section) => section.id === "exportar");
      const blocks = [
        ...(document?.blocks ?? []),
        ...(shortcuts?.blocks ?? []),
        ...(problems?.blocks ?? []),
        ...(exportSection?.blocks ?? [])
      ];
      const text = blocks
        .map((block) => {
          const record = block as unknown as {
            items?: string[];
            text?: string;
            rows?: string[][];
          };
          return [
            ...(record.items ?? []),
            record.text ?? "",
            ...(record.rows?.flat() ?? [])
          ].join("\n");
        })
        .join("\n");

      expect(text).toContain("▾");
      expect(text).toContain("●");
      expect(text).toContain(language === "es" ? "Errores (N)" : "Errors (N)");
      expect(text).toContain("Shift+Tab");
      expect(text).toMatch(language === "es" ? /Versiones/ : /Versions/);
      expect(text).toMatch(language === "es" ? /rueda/ : /wheel/);
      expect(text).toContain("Excel");
      expect(text).toContain(language === "es" ? "Cálculos" : "Calculations");
    }
  });

  it("returns every section for an empty query", () => {
    const content = getManualContent("es");
    expect(searchManual(content, "  ")).toHaveLength(content.sections.length);
    expect(searchManual(content, "zzzz-no-existe")).toHaveLength(0);
  });

  it("uses unique section ids", () => {
    for (const language of ["es", "en"]) {
      const ids = getManualContent(language).sections.map((section) => section.id);
      expect(new Set(ids).size).toBe(ids.length);
    }
  });
});
