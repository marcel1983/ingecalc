import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { clearDraft, readDraft, writeDraft } from "./draft";
import { newDocument } from "../stores/document";

const storage = new Map<string, string>();

const localStorageMock = {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => {
    storage.set(key, value);
  },
  removeItem: (key: string) => {
    storage.delete(key);
  }
};

describe("draft storage", () => {
  beforeEach(() => {
    storage.clear();
    vi.stubGlobal("localStorage", localStorageMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("writes and reads a draft", async () => {
    const document = newDocument();
    const written = await writeDraft(document);
    const read = await readDraft();

    expect(read?.savedAt).toBe(written.savedAt);
    expect(read?.document.metadata.title).toBe("Sin título");
    expect(read?.document.pages).toHaveLength(1);
  });

  it("clears a draft", async () => {
    await writeDraft(newDocument());
    await clearDraft();
    expect(await readDraft()).toBeNull();
  });

  it("ignores malformed storage", async () => {
    storage.set("ingecalc.draft.v1", "{");
    expect(await readDraft()).toBeNull();
  });
});
