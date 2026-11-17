const STORAGE_KEY = "ingecalc-recent-files";
export const MAX_RECENT_FILES = 8;

export type RecentFile = {
  path: string;
  title: string;
  openedAt: number;
};

export function readRecentFiles(): RecentFile[] {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter(
        (item): item is RecentFile =>
          Boolean(item) &&
          typeof item === "object" &&
          typeof (item as RecentFile).path === "string" &&
          (item as RecentFile).path.length > 0 &&
          typeof (item as RecentFile).title === "string" &&
          typeof (item as RecentFile).openedAt === "number"
      )
      .slice(0, MAX_RECENT_FILES);
  } catch {
    return [];
  }
}

export function rememberRecentFile(
  files: RecentFile[],
  path: string,
  title: string
): RecentFile[] {
  const next = [
    { path, title: title || path, openedAt: Date.now() },
    ...files.filter((item) => item.path !== path)
  ].slice(0, MAX_RECENT_FILES);
  writeRecentFiles(next);
  return next;
}

export function forgetRecentFile(files: RecentFile[], path: string): RecentFile[] {
  const next = files.filter((item) => item.path !== path);
  writeRecentFiles(next);
  return next;
}

export function writeRecentFiles(files: RecentFile[]): void {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(files));
  } catch {
    // A blocked storage should not break file opening.
  }
}

export function sortRecentFiles(files: RecentFile[]): RecentFile[] {
  return [...files]
    .filter((item) => item.path.length > 0)
    .sort((left, right) => right.openedAt - left.openedAt)
    .slice(0, MAX_RECENT_FILES);
}