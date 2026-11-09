import { describe, expect, it } from "vitest";
import {
  DEFAULT_SURFACE_VIEW,
  surfaceSvgMarkup,
  type SurfaceGrid
} from "./plot3d";

const surface = (values: (number | null)[]): SurfaceGrid => ({
  expression: "sin(x)*cos(y)",
  color: "#2563eb",
  grid: [values.slice(0, 3), values.slice(3, 6), values.slice(6, 9)],
  xMin: -1,
  xMax: 1,
  yMin: -1,
  yMax: 1
});

describe("surface plot", () => {
  it("renders surface polygons and axes", () => {
    const svg = surfaceSvgMarkup(
      [surface([0, 1, 0, 1, 0, -1, 0, -1, 0])],
      640,
      480,
      DEFAULT_SURFACE_VIEW,
      "surface"
    );
    expect(svg).toContain("aria-label=\"Gráfico 3D\"");
    expect(svg).toContain("<polygon");
    expect(svg).toContain(">X</text>");
    expect(svg).toContain(">Y</text>");
    expect(svg).toContain("sin(x)*cos(y)");
  });

  it("supports a wireframe view", () => {
    const svg = surfaceSvgMarkup(
      [surface([0, 1, 0, 1, 0, -1, 0, -1, 0])],
      640,
      480,
      { ...DEFAULT_SURFACE_VIEW, wireframe: true },
      "wire"
    );
    expect(svg).toContain('fill="none"');
    expect(svg).not.toContain('fill="#2563eb" fill-opacity="0.82"');
  });

  it("skips cells with missing values", () => {
    const svg = surfaceSvgMarkup(
      [surface([0, null, 0, 1, 0, -1, 0, -1, 0])],
      640,
      480,
      DEFAULT_SURFACE_VIEW,
      "holes"
    );
    expect(svg).toContain("<polygon");
    expect(svg).toContain("z:");
  });
});
