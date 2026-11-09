import { invoke } from "@tauri-apps/api/core";
import type { Document } from "../types/document";
import { isTauri } from "./environment";

const STORAGE_KEY = "ingecalc.draft.v1";
const MAX_LOCAL_BYTES = 4_000_000;

export type Draft = {
  document: Document;
  savedAt: number;
};

type StoredDraft = {
  savedAt: number;
  document: Document;
};

export async function readDraft(): Promise<Draft | null> {
  if (isTauri()) {
    try {
      const stored = await invoke<StoredDraft | null>("load_draft");
      if (stored?.document) {
        return stored;
      }
    } catch {
      // Fall through to the local draft for restricted environments.
    }
  }

  return readLocalDraft();
}

export async function writeDraft(document: Document): Promise<Draft> {
  if (isTauri()) {
    try {
      const savedAt = await invoke<number>("save_draft", { document });
      return { document, savedAt };
    } catch (error) {
      try {
        return writeLocalDraft(document);
      } catch {
        throw error;
      }
    }
  }

  return writeLocalDraft(document);
}

export async function clearDraft(): Promise<void> {
  if (isTauri()) {
    try {
      await invoke("clear_draft");
    } catch {
      // The local copy is still cleared below.
    }
  }

  clearLocalDraft();
}

function readLocalDraft(): Draft | null {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as Partial<Draft>;
    if (!parsed.document || typeof parsed.savedAt !== "number") {
      return null;
    }

    return parsed as Draft;
  } catch {
    return null;
  }
}

function writeLocalDraft(document: Document): Draft {
  const draft: Draft = { document, savedAt: Date.now() };
  const serialized = JSON.stringify(draft);

  if (serialized.length > MAX_LOCAL_BYTES) {
    throw new Error("el borrador local supera el límite de 4 MB");
  }

  globalThis.localStorage?.setItem(STORAGE_KEY, serialized);
  return draft;
}

function clearLocalDraft(): void {
  try {
    globalThis.localStorage?.removeItem(STORAGE_KEY);
  } catch {
    // A blocked storage should not break document editing.
  }
}
