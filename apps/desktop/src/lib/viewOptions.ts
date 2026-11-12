export const GRID_SIZE = 20;

export type ViewOptions = {
  showGrid: boolean;
  snapToGrid: boolean;
  lockPositions: boolean;
  showResults: boolean;
};

export const DEFAULT_VIEW_OPTIONS: ViewOptions = {
  showGrid: true,
  snapToGrid: true,
  lockPositions: false,
  showResults: true
};

const STORAGE_KEY = "ingecalc-view-options";

export function readViewOptions(): ViewOptions {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (!raw) {
      return { ...DEFAULT_VIEW_OPTIONS };
    }
    const parsed = JSON.parse(raw) as Partial<ViewOptions>;
    return {
      showGrid:
        typeof parsed.showGrid === "boolean"
          ? parsed.showGrid
          : DEFAULT_VIEW_OPTIONS.showGrid,
      snapToGrid:
        typeof parsed.snapToGrid === "boolean"
          ? parsed.snapToGrid
          : DEFAULT_VIEW_OPTIONS.snapToGrid,
      lockPositions:
        typeof parsed.lockPositions === "boolean"
          ? parsed.lockPositions
          : DEFAULT_VIEW_OPTIONS.lockPositions,
      showResults:
        typeof parsed.showResults === "boolean"
          ? parsed.showResults
          : DEFAULT_VIEW_OPTIONS.showResults
    };
  } catch {
    return { ...DEFAULT_VIEW_OPTIONS };
  }
}

export function writeViewOptions(options: ViewOptions): void {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(options));
  } catch {
    // A blocked storage should not break the editor.
  }
}
