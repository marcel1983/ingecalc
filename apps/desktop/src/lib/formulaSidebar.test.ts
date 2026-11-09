import { describe, expect, it } from "vitest";
import type { PaletteItem } from "./palettes";
import { ASSIGNMENT_ITEM, buildFormulaSections } from "./formulaSidebar";

describe("formula sidebar catalog", () => {
  it("keeps the assignment operator in the first section", () => {
    const sections = buildFormulaSections([]);
    expect(sections[0].id).toBe("assignment");
    expect(sections[0].items[0]).toEqual(ASSIGNMENT_ITEM);
    expect(sections[0].items[0].latex).toBe(":=");
  });

  it("includes supported functions, matrices and custom units", () => {
    const customUnit: PaletteItem = {
      id: "unit-mi_pulgada",
      label: "mi_pulgada",
      latex: "\\textcolor{blue}{\\text{mi_pulgada}}",
      sample: "\\textcolor{blue}{\\text{mi_pulgada}}",
      description: "Pulgada propia",
      category: "Propias"
    };
    const sections = buildFormulaSections([customUnit]);
    const functions = sections.find((section) => section.id === "functions");
    const matrices = sections.find((section) => section.id === "matrices");
    const units = sections.find((section) => section.id === "units");

    expect(functions?.items.map((item) => item.id)).toContain("sin");
    expect(functions?.items.map((item) => item.id)).toContain("round");
    expect(matrices?.items.map((item) => item.id)).toContain("solve");
    expect(units?.items).toContainEqual(customUnit);
  });

  it("includes lowercase and uppercase Greek letters", () => {
    const sections = buildFormulaSections([]);
    const lower = sections.find((section) => section.id === "greek-lower");
    const upper = sections.find((section) => section.id === "greek-upper");

    expect(lower?.items).toHaveLength(24);
    expect(upper?.items).toHaveLength(24);
    expect(lower?.items.map((item) => item.label)).toContain("α");
    expect(lower?.items.map((item) => item.label)).toContain("ω");
    expect(upper?.items.map((item) => item.label)).toContain("Γ");
    expect(upper?.items.map((item) => item.label)).toContain("Ω");
  });
});
