import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  MAX_DRAFT_HISTORY,
  clearDraftHistory,
  readDraftHistory,
  rememberDraftSnapshot,
  writeDraftHistory
} from "./draftHistory";
import { newDocument } from "../stores/document";

const storage = new Map<string, string>();
vi.stubGlobal("localStorage", {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => {
    storage.set(key, value);
  },
  removeItem: (key: string) => {
    storage.delete(key);
  }
});

beforeEach(() => {
  storage.clear();
});

afterEach(() => {
  storage.clear();
});

describe("draft history", () => {
  it("keeps the newest snapshots and caps the history", () => {
    const document = newDocument();
    const first = rememberDraftSnapshot(document, 1000);
    const second = rememberDraftSnapshot(document, 2000);
    const third = rememberDraftSnapshot(document, 3000);

    expect(first).toHaveLength(1);
    expect(second).toHaveLength(2);
    expect(third.map((item) => item.savedAt)).toEqual([3000, 2000, 1000]);
    expect(readDraftHistory()).toEqual(third);

    let history = third;
    for (let i = 0; i < MAX_DRAFT_HISTORY; i++) {
      history = rememberDraftSnapshot(document, 4000 + i);
    }
    expect(history).toHaveLength(MAX_DRAFT_HISTORY);
  });

  it("clears the history", () => {
    rememberDraftSnapshot(newDocument(), 1);
    clearDraftHistory();
    expect(readDraftHistory()).toEqual([]);
  });

  it("ignores malformed storage", () => {
    storage.set("ingecalc-draft-history-v1", "{");
    expect(readDraftHistory()).toEqual([]);
    writeDraftHistory([]);
    expect(readDraftHistory()).toEqual([]);
  });
});