import { afterEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_VIEW_OPTIONS, readViewOptions } from "./viewOptions";

const values = new Map<string, string>();
vi.stubGlobal("localStorage", {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => {
    values.set(key, value);
  },
  removeItem: (key: string) => {
    values.delete(key);
  }
});

afterEach(() => {
  values.clear();
});

describe("view options", () => {
  it("defaults to grid and snap enabled when there is no stored value", () => {
    globalThis.localStorage?.removeItem("ingecalc-view-options");
    expect(readViewOptions()).toEqual(DEFAULT_VIEW_OPTIONS);
    expect(DEFAULT_VIEW_OPTIONS.lockPositions).toBe(false);
    expect(DEFAULT_VIEW_OPTIONS.showResults).toBe(true);
  });

  it("remembers when automatic results are hidden", () => {
    globalThis.localStorage?.setItem(
      "ingecalc-view-options",
      JSON.stringify({ showResults: false })
    );
    expect(readViewOptions().showResults).toBe(false);
    globalThis.localStorage?.removeItem("ingecalc-view-options");
  });
});
