import { describe, expect, it } from "vitest";
import { computePlotView, plotSvgMarkup } from "./plot";

describe("plot geometry", () => {
  it("computes a padded view around sampled values", () => {
    const view = computePlotView(
      [
        {
          expression: "x",
          color: "#2563eb",
          paths: [
            [
              [-1, -1],
              [1, 1]
            ]
          ]
        }
      ],
      -1,
      1
    );

    expect(view.xMin).toBe(-1);
    expect(view.xMax).toBe(1);
    expect(view.yMin).toBeLessThan(-1);
    expect(view.yMax).toBeGreaterThan(1);
  });

  it("renders an SVG path and escapes labels", () => {
    const series = [
      {
        expression: "x<y",
        color: "#ff0000",
        paths: [
          [
            [0, 0],
            [1, 1]
          ]
        ] as [number, number][][]
      }
    ];
    const view = computePlotView(series, 0, 1);
    const svg = plotSvgMarkup(series, view, 320, 200, "plot-1");

    expect(svg).toContain("<svg");
    expect(svg).toContain('stroke="#ff0000"');
    expect(svg).toContain("<path");
    expect(svg).toContain("x&lt;y");
  });

  it("falls back to blue for invalid colors", () => {
    const svg = plotSvgMarkup(
      [
        {
          expression: "x",
          color: "red",
          paths: [[[0, 0]]]
        }
      ],
      { xMin: -1, xMax: 1, yMin: -1, yMax: 1 },
      200,
      120,
      "plot-2"
    );
    expect(svg).toContain('stroke="#2563eb"');
  });

  it("snaps floating point noise around zero ticks", () => {
    const svg = plotSvgMarkup(
      [],
      { xMin: -1.543, xMax: 1.537, yMin: -1.543, yMax: 1.537 },
      320,
      200,
      "plot-3"
    );
    expect(svg).toContain(">0</text>");
    expect(svg).not.toContain("6.1e-6");
  });
});
