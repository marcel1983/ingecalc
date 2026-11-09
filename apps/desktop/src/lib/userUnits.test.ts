import { describe, expect, it } from "vitest";
import { newDocument } from "../stores/document";
import { customUnitPalette, extractUserUnits } from "./userUnits";

describe("user units", () => {
  it("extracts unit definitions from math regions", () => {
    const document = newDocument();
    document.pages[0].regions.push({
      id: "unit-region",
      type: "math",
      x: 60,
      y: 300,
      w: 360,
      h: 48,
      source: "unit(\\text{mi_pulgada}) := 2.54 cm",
      ascii: 'unit("mi_pulgada") := 2.54 cm'
    });

    expect(extractUserUnits(document)).toEqual(["mi_pulgada"]);
    expect(customUnitPalette(["mi_pulgada"])[0]).toMatchObject({
      label: "mi_pulgada",
      latex: "\\textcolor{blue}{\\text{mi_pulgada}}",
      category: "Propias"
    });
  });

  it("ignores invalid unit names", () => {
    const document = newDocument();
    document.pages[0].regions.push({
      id: "unit-region",
      type: "math",
      x: 60,
      y: 300,
      w: 360,
      h: 48,
      source: "unit(\\text{bad-name}) := 1 cm",
      ascii: 'unit("bad-name") := 1 cm'
    });

    expect(extractUserUnits(document)).toEqual([]);
  });
});
