import type { Document } from "../types/document";
import type { PaletteItem } from "./palettes";

const UNIT_NAME = /^[\p{L}\p{N}_]+$/u;

export function extractUserUnits(document: Document): string[] {
  const names = new Set<string>();

  for (const page of document.pages) {
    for (const region of page.regions) {
      if (region.type !== "math") {
        continue;
      }

      for (const match of region.ascii.matchAll(/unit\("([^"]+)"\)/g)) {
        if (UNIT_NAME.test(match[1])) {
          names.add(match[1]);
        }
      }
      for (const match of region.source.matchAll(/unit\(\\text\{([^}]+)\}\)/g)) {
        if (UNIT_NAME.test(match[1])) {
          names.add(match[1]);
        }
      }
    }
  }

  return [...names];
}

export function customUnitPalette(names: string[]): PaletteItem[] {
  return names.map((name) => ({
    id: `custom-unit-${name}`,
    label: name,
    latex: `\\textcolor{blue}{\\text{${name}}}`,
    sample: `\\textcolor{blue}{\\text{${name}}}`,
    description: "Unidad definida en el documento",
    category: "Propias"
  }));
}
