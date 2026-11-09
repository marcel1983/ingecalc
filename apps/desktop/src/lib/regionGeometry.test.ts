import { describe, expect, it } from "vitest";
import {
  MIN_REGION_HEIGHT,
  MIN_REGION_WIDTH,
  contentBounds,
  moveRect,
  moveRectInBounds,
  resizeRect,
  resizeRectInBounds
} from "./regionGeometry";

const page = { width: 850, height: 1100 };
const rect = { x: 60, y: 60, w: 360, h: 48 };
const margins = { top: 48, right: 48, bottom: 48, left: 48 };

describe("region geometry", () => {
  it("moves a region by the pointer delta", () => {
    expect(moveRect(rect, 100, 80, page)).toEqual({
      x: 160,
      y: 140,
      w: 360,
      h: 48
    });
  });

  it("clamps movement to the page", () => {
    expect(moveRect(rect, 5000, 5000, page)).toEqual({
      x: page.width - rect.w,
      y: page.height - rect.h,
      w: rect.w,
      h: rect.h
    });

    expect(moveRect(rect, -5000, -5000, page)).toEqual({
      x: 0,
      y: 0,
      w: rect.w,
      h: rect.h
    });
  });

  it("resizes from the bottom-right corner", () => {
    expect(resizeRect(rect, 100, 50, page)).toEqual({
      x: 60,
      y: 60,
      w: 460,
      h: 98
    });
  });

  it("enforces the minimum size", () => {
    expect(resizeRect(rect, -5000, -5000, page)).toEqual({
      x: 60,
      y: 60,
      w: MIN_REGION_WIDTH,
      h: MIN_REGION_HEIGHT
    });
  });

  it("caps the size at the page edge", () => {
    expect(
      resizeRect({ x: 700, y: 1000, w: 120, h: 48 }, 500, 500, page)
    ).toEqual({
      x: 700,
      y: 1000,
      w: 150,
      h: 100
    });
  });

  it("snaps movement and resize to the grid", () => {
    expect(
      moveRect({ ...rect, x: 61, y: 59 }, 5, 3, page, 20)
    ).toMatchObject({ x: 60, y: 60 });
    expect(resizeRect(rect, 7, 7, page, 20)).toMatchObject({
      w: 360,
      h: 60
    });
  });

  it("computes the printable content area", () => {
    expect(contentBounds(page, margins)).toEqual({
      left: 48,
      top: 48,
      right: 802,
      bottom: 1052
    });
  });

  it("clamps movement to the content margins", () => {
    const bounds = contentBounds(page, margins);
    expect(moveRectInBounds(rect, -5000, -5000, page, bounds)).toEqual({
      x: 48,
      y: 48,
      w: 360,
      h: 48
    });
    expect(moveRectInBounds(rect, 5000, 5000, page, bounds)).toEqual({
      x: 442,
      y: 1004,
      w: 360,
      h: 48
    });
  });

  it("caps resizing at the content margins", () => {
    const bounds = contentBounds(page, margins);
    expect(resizeRectInBounds(rect, 5000, 5000, page, bounds)).toEqual({
      x: 60,
      y: 60,
      w: 742,
      h: 992
    });
  });
});
