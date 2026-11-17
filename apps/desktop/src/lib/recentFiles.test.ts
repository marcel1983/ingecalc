import { afterEach, describe, expect, it, vi } from "vitest";
import {
  MAX_RECENT_FILES,
  readRecentFiles,
  rememberRecentFile,
  forgetRecentFile,
  sortRecentFiles
} from "./recentFiles";

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

function file(path: string) {
  return { path, title: path, openedAt: 1 };
}

describe("recent files", () => {
  it("remembers the most recent file first", () => {
    const first = rememberRecentFile([], "a.smj", "A");
    const second = rememberRecentFile(first, "b.smj", "B");
    const again = rememberRecentFile(second, "a.smj", "A");

    expect(again.map((item) => item.path)).toEqual(["a.smj", "b.smj"]);
    expect(readRecentFiles()).toEqual(again);
  });

  it("caps the history", () => {
    let files = Array.from({ length: MAX_RECENT_FILES }, (_, i) => file(`f${i}.smj`));
    files = rememberRecentFile(files, "new.smj", "new");
    expect(files).toHaveLength(MAX_RECENT_FILES);
    expect(files[0].path).toBe("new.smj");
  });

  it("forgets a missing file", () => {
    const files = rememberRecentFile([], "a.smj", "A");
    expect(forgetRecentFile(files, "a.smj")).toEqual([]);
  });

  it("sorts and limits stored files", () => {
    const files = sortRecentFiles([
      { path: "a.smj", title: "A", openedAt: 1 },
      { path: "b.smj", title: "B", openedAt: 3 },
      { path: "c.smj", title: "C", openedAt: 2 }
    ]);
    expect(files.map((item) => item.path)).toEqual(["b.smj", "c.smj", "a.smj"]);
  });
});