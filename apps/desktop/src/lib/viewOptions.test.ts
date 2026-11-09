import { describe, expect, it } from "vitest";
import { DEFAULT_VIEW_OPTIONS, readViewOptions } from "./viewOptions";

describe("view options", () => {
  it("defaults to grid and snap enabled when there is no stored value", () => {
    globalThis.localStorage?.removeItem("ingecalc-view-options");
    expect(readViewOptions()).toEqual(DEFAULT_VIEW_OPTIONS);
    expect(DEFAULT_VIEW_OPTIONS.lockPositions).toBe(false);
  });
});
