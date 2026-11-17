import type { Document } from "../types/document";

const HISTORY_KEY = "ingecalc-draft-history-v1";
export const HISTORY_INTERVAL_MS = 5 * 60 * 1000;
export const MAX_DRAFT_HISTORY = 10;

export type DraftSnapshot = {
  id: string;
  savedAt: number;
  title: string;
  document: Document;
};

export function readDraftHistory(): DraftSnapshot[] {
  try {
    const raw = globalThis.localStorage?.getItem(HISTORY_KEY);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter(
        (item): item is DraftSnapshot =>
          Boolean(item) &&
          typeof item === "object" &&
          typeof (item as DraftSnapshot).id === "string" &&
          typeof (item as DraftSnapshot).savedAt === "number" &&
          typeof (item as DraftSnapshot).title === "string" &&
          Boolean((item as DraftSnapshot).document)
      )
      .sort((left, right) => right.savedAt - left.savedAt)
      .slice(0, MAX_DRAFT_HISTORY);
  } catch {
    return [];
  }
}

export function rememberDraftSnapshot(
  document: Document,
  savedAt = Date.now()
): DraftSnapshot[] {
  const history = readDraftHistory();
  const snapshot: DraftSnapshot = {
    id: `${savedAt}-${Math.random().toString(36).slice(2, 8)}`,
    savedAt,
    title: document.metadata.title || "Sin título",
    document
  };
  const next = [snapshot, ...history]
    .sort((left, right) => right.savedAt - left.savedAt)
    .slice(0, MAX_DRAFT_HISTORY);
  writeDraftHistory(next);
  return next;
}

export function writeDraftHistory(history: DraftSnapshot[]): void {
  try {
    globalThis.localStorage?.setItem(HISTORY_KEY, JSON.stringify(history));
  } catch {
    // A blocked storage should not break document editing.
  }
}

export function clearDraftHistory(): void {
  try {
    globalThis.localStorage?.removeItem(HISTORY_KEY);
  } catch {
    // Ignore storage restrictions.
  }
}