import { describe, expect, it } from "vitest";
import {
  findFreeY,
  rectsOverlap,
  resolveVerticalOverlaps,
  type LayoutBox
} from "./layout";

const box = (
  id: string,
  x: number,
  y: number,
  w: number,
  h: number,
  extra: Partial<LayoutBox> = {}
): LayoutBox => ({ id, x, y, w, h, ...extra });

describe("region layout", () => {
  it("detects overlapping boxes", () => {
    expect(rectsOverlap(box("a", 0, 0, 100, 50), box("b", 50, 25, 100, 50))).toBe(true);
    expect(rectsOverlap(box("a", 0, 0, 100, 50), box("b", 100, 0, 100, 50))).toBe(false);
    expect(rectsOverlap(box("a", 0, 0, 100, 50), box("b", 0, 50, 100, 50))).toBe(false);
  });

  it("pushes the boxes below a growing region", () => {
    const regions = [
      box("target", 40, 40, 300, 80),
      box("below", 40, 120, 300, 60),
      box("far", 40, 260, 300, 60)
    ];

    const { positions, blockedByPinned } = resolveVerticalOverlaps(regions, "target");
    expect(blockedByPinned).toBe(false);
    expect(positions.get("below")).toBe(132);
    expect(positions.has("far")).toBe(false);
  });

  it("cascades the push through several boxes", () => {
    const regions = [
      box("target", 40, 40, 300, 140),
      box("first", 40, 100, 300, 60),
      box("second", 40, 172, 300, 60)
    ];

    const { positions } = resolveVerticalOverlaps(regions, "target");
    expect(positions.get("first")).toBe(192);
    expect(positions.get("second")).toBe(264);
  });

  it("does not move boxes in another column", () => {
    const regions = [
      box("target", 40, 40, 300, 100),
      box("right", 400, 40, 200, 60)
    ];

    const { positions } = resolveVerticalOverlaps(regions, "target");
    expect(positions.size).toBe(0);
  });

  it("reports a pinned box that cannot be pushed", () => {
    const regions = [
      box("target", 40, 40, 300, 120),
      box("pinned", 40, 100, 300, 60, { pinned: true })
    ];

    const { blockedByPinned } = resolveVerticalOverlaps(regions, "target");
    expect(blockedByPinned).toBe(true);
  });

  it("finds the next free vertical position", () => {
    const regions = [
      box("first", 40, 40, 300, 60),
      box("second", 40, 160, 300, 60)
    ];
    const y = findFreeY(regions, box("new", 40, 0, 300, 80), 40);
    expect(y).toBe(232);
  });
});
