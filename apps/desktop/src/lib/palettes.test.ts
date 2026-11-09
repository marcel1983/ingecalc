import { describe, expect, it } from "vitest";
import { latexToAscii } from "../components/InlineMath";
import { PALETTES, PALETTE_TITLES, type PaletteKind } from "./palettes";

const kinds: PaletteKind[] = ["function", "unit"];

describe("palette catalogs", () => {
  it.each(kinds)("has unique and complete %s entries", (kind) => {
    const items = PALETTES[kind];
    expect(items.length).toBeGreaterThan(10);
    expect(PALETTE_TITLES[kind]).not.toBe("");

    const ids = new Set<string>();
    for (const item of items) {
      expect(item.id).not.toBe("");
      expect(item.label).not.toBe("");
      expect(item.latex).not.toBe("");
      expect(item.sample).not.toBe("");
      expect(item.description).not.toBe("");
      expect(item.category).not.toBe("");
      expect(ids.has(item.id)).toBe(false);
      ids.add(item.id);
    }
  });

  it("keeps function templates free of placeholders in their sample", () => {
    for (const item of PALETTES.function) {
      expect(item.latex).toContain("\\placeholder{}");
      expect(item.sample).not.toContain("\\placeholder{}");
    }
  });

  it("serializes multi-letter function samples as single names", () => {
    const names = [
      "trace",
      "row",
      "col",
      "augment",
      "stack",
      "minor",
      "cofactor",
      "sum",
      "mean",
      "asinh",
      "acosh",
      "atanh",
      "factorial",
      "comb",
      "round",
      "trunc",
      "frac",
      "sign",
      "mod",
      "gcd",
      "lcm"
    ];
    for (const item of PALETTES.function) {
      const ascii = latexToAscii(item.sample);
      for (const name of names) {
        if (item.sample.includes(`\\text{${name}}`)) {
          expect(ascii).toContain(name);
          expect(ascii).not.toMatch(new RegExp(name.split("").join("\\s+")));
        }
      }
    }
  });
});
