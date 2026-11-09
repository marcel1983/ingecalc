import { create } from "zustand";
import { produce } from "immer";
import {
  MIN_REGION_HEIGHT,
  MIN_REGION_WIDTH
} from "../lib/regionGeometry";
import {
  findFreeY,
  rectsOverlap,
  resolveVerticalOverlaps
} from "../lib/layout";
import type {
  Document,
  ImageRegion,
  MathRegion,
  Page,
  PageSetup,
  PlotRegion,
  Region,
  TextRegion
} from "../types/document";

export type MathResult = {
  display: string;
  error: string;
};

const id = () =>
  globalThis.crypto?.randomUUID?.() ?? Math.random().toString(36).slice(2);

const HISTORY_LIMIT = 50;
const COALESCE_MS = 600;

let regionClipboard: Region[] = [];
let groupMoveSnapshot:
  | Array<{ id: string; x: number; y: number; w: number; h: number }>
  | null = null;
let groupMovePageId: string | null = null;

export function newDocument(): Document {
  const page: Page = {
    id: id(),
    width: 850,
    height: 1100,
    regions: [
      {
        id: id(),
        type: "math",
        x: 60,
        y: 60,
        w: 360,
        h: 48,
        source: "2+2",
        ascii: "2+2"
      },
      {
        id: id(),
        type: "text",
        x: 60,
        y: 140,
        w: 500,
        h: 100,
        html: "<p>Escribe aquí...</p>"
      }
    ]
  };

  return {
    format: "ingecalc-json",
    version: 1,
    metadata: {
      title: "Sin título",
      author: "",
      language: "es"
    },
    settings: {
      units: "metric",
      unitOverrides: {},
      customConstants: [],
      personalDictionary: [],
      precision: 4,
      fractions: "auto",
      notation: "auto",
      trimZeros: true,
      page: {
        marginTop: 48,
        marginRight: 48,
        marginBottom: 48,
        marginLeft: 48,
        header: "",
        footer: ""
      }
    },
    pages: [page]
  };
}

type Store = {
  doc: Document;
  selectedIds: string[];
  past: Document[];
  future: Document[];
  historyKey: string | null;
  historyAt: number;
  transactionKey: string | null;
  recalcToken: number;
  mathResults: Record<string, MathResult>;
  setDoc: (doc: Document) => void;
  newDoc: () => void;
  addMathRegion: () => void;
  addMathRegionWith: (source: string, ascii: string) => void;
  addMathRegionAt: (pageId: string, x: number, y: number) => string | null;
  addMathRegionBelow: (regionId: string) => string | null;
  addTextRegion: () => void;
  addHeaderRegion: () => void;
  addFooterRegion: () => void;
  addImageRegion: (src: string, width?: number, height?: number) => void;
  addPlotRegion: () => void;
  addPage: () => void;
  removePage: (pageId: string) => void;
  movePage: (pageId: string, direction: -1 | 1) => void;
  setPageSize: (width: number, height: number) => void;
  updatePageSetup: (setup: PageSetup) => void;
  updateDocumentLanguage: (language: string) => void;
  updateResultUnits: (
    system: string,
    overrides: Record<string, string>
  ) => void;
  updateNumberFormat: (format: {
    notation: string;
    precision: number;
    fractions: string;
    trimZeros: boolean;
  }) => void;
  updateCustomConstants: (
    constants: Array<{ name: string; expression: string; description?: string }>
  ) => void;
  addPersonalDictionaryWord: (word: string) => void;
  updateRegion: (id: string, patch: Partial<Region>) => void;
  syncRegionSize: (id: string, w: number, h: number) => void;
  removeRegion: (id: string) => void;
  removeSelected: () => void;
  duplicateSelected: () => void;
  copySelected: () => number;
  cutSelected: () => number;
  pasteClipboard: () => number;
  clearClipboard: () => void;
  select: (id: string | null, additive?: boolean) => void;
  selectMany: (ids: string[]) => void;
  beginGroupMove: (pageId: string) => void;
  moveGroup: (dx: number, dy: number, snap?: number) => void;
  endGroupMove: () => void;
  clearSelection: () => void;
  beginTransaction: (key: string) => void;
  endTransaction: () => void;
  undo: () => void;
  redo: () => void;
  recalculate: () => void;
  setMathResults: (results: Record<string, MathResult>) => void;
};

function historyPatch(state: Store, key: string | null): Partial<Store> {
  if (state.transactionKey) {
    return {};
  }

  const now = Date.now();

  if (
    key !== null &&
    key === state.historyKey &&
    now - state.historyAt < COALESCE_MS
  ) {
    return { historyKey: key, historyAt: now };
  }

  return {
    past: [...state.past.slice(-(HISTORY_LIMIT - 1)), state.doc],
    future: [],
    historyKey: key,
    historyAt: now
  };
}

function nextY(
  page: { height: number; regions: Region[] },
  height: number,
  setup: PageSetup
): number {
  const top = Math.max(0, setup.marginTop);
  const contentBottom = Math.max(
    top + MIN_REGION_HEIGHT,
    page.height - setup.marginBottom
  );
  const bottom = page.regions.reduce(
    (max, region) => Math.max(max, region.y + region.h),
    top
  );
  const desired = Math.max(top, bottom + 24);
  return Math.max(
    top,
    Math.min(desired, Math.max(top, contentBottom - height))
  );
}

function clampRegionToContent(
  region: Region,
  page: { width: number; height: number },
  setup: PageSetup
): void {
  if ("role" in region && region.role) {
    region.w = Math.max(
      MIN_REGION_WIDTH,
      Math.min(region.w, Math.max(MIN_REGION_WIDTH, page.width))
    );
    region.h = Math.max(
      MIN_REGION_HEIGHT,
      Math.min(region.h, Math.max(MIN_REGION_HEIGHT, page.height))
    );
    region.x = Math.max(0, Math.min(region.x, page.width - region.w));
    region.y = Math.max(0, Math.min(region.y, page.height - region.h));
    return;
  }

  const left = Math.max(0, setup.marginLeft);
  const top = Math.max(0, setup.marginTop);
  const right = Math.max(
    left + MIN_REGION_WIDTH,
    page.width - setup.marginRight
  );
  const bottom = Math.max(
    top + MIN_REGION_HEIGHT,
    page.height - setup.marginBottom
  );

  region.w = Math.max(
    MIN_REGION_WIDTH,
    Math.min(region.w, right - left)
  );
  region.x = Math.max(left, Math.min(region.x, right - region.w));
  region.h = Math.max(MIN_REGION_HEIGHT, region.h);
  region.y = Math.max(top, Math.min(region.y, Math.max(top, bottom - region.h)));
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export const useDocumentStore = create<Store>((set, get) => {
  const mutate = (
    key: string | null,
    recipe: (draft: Store) => void
  ) => {
    set((state) => {
      const history = historyPatch(state, key);
      const next = produce(state, (draft) => {
        recipe(draft);
      });
      return { ...next, ...history };
    });
  };

  const addPageFurniture = (role: "header" | "footer") =>
    mutate(null, (draft) => {
      const page = draft.doc.pages[draft.doc.pages.length - 1];
      if (!page) {
        return;
      }

      const existing = page.regions.find(
        (region) => region.type === "text" && region.role === role
      );
      if (existing) {
        draft.selectedIds = [existing.id];
        return;
      }

      const setup = draft.doc.settings.page;
      const height = 48;
      const width = Math.max(
        160,
        page.width - setup.marginLeft - setup.marginRight
      );
      const x = Math.max(0, setup.marginLeft);
      const y =
        role === "header"
          ? Math.max(0, setup.marginTop - height)
          : Math.max(0, page.height - setup.marginBottom);
      const legacy = role === "header" ? setup.header : setup.footer;
      const label = role === "header" ? "Encabezado" : "Pie de página";
      const region: TextRegion = {
        id: id(),
        type: "text",
        x,
        y,
        w: width,
        h: height,
        html: `<p>${escapeHtml(legacy.trim() || label)}</p>`,
        role
      };

      page.regions.push(region);
      if (role === "header") {
        draft.doc.settings.page.header = "";
      } else {
        draft.doc.settings.page.footer = "";
      }
      draft.selectedIds = [region.id];
    });

  const pageBottom = (doc: Document, page: Page) =>
    page.height - doc.settings.page.marginBottom;

  const hasRole = (region: Region) =>
    "role" in region && Boolean(region.role);

  const placeOnNextPage = (
    draft: Document,
    pageIndex: number,
    regionId: string
  ) => {
    const page = draft.pages[pageIndex];
    if (!page) {
      return;
    }
    const index = page.regions.findIndex((region) => region.id === regionId);
    if (index < 0) {
      return;
    }
    const [region] = page.regions.splice(index, 1);
    if (!region) {
      return;
    }

    const setup = draft.settings.page;
    const startY = Math.max(0, setup.marginTop);
    let targetIndex = pageIndex + 1;
    if (targetIndex >= draft.pages.length) {
      draft.pages.push({
        id: id(),
        width: page.width,
        height: page.height,
        regions: []
      });
    }

    let targetPage = draft.pages[targetIndex];
    region.x = Math.max(0, Math.min(region.x, targetPage.width - region.w));
    let y = findFreeY(targetPage.regions, { ...region, y: startY }, startY);

    if (y + region.h > pageBottom(draft, targetPage)) {
      if (targetPage.regions.length > 0) {
        targetIndex += 1;
        draft.pages.splice(targetIndex, 0, {
          id: id(),
          width: page.width,
          height: page.height,
          regions: []
        });
        targetPage = draft.pages[targetIndex];
        y = startY;
      } else {
        y = startY;
      }
    }

    region.y = Math.round(y);
    targetPage.regions.push(region);
  };

  const reflowAfterResize = (
    draft: Document,
    pageIndex: number,
    targetId: string
  ) => {
    const page = draft.pages[pageIndex];
    const target = page?.regions.find((region) => region.id === targetId);
    if (!page || !target || hasRole(target)) {
      return;
    }

    if (target.y + target.h > pageBottom(draft, page)) {
      placeOnNextPage(draft, pageIndex, targetId);
      return;
    }

    const { positions, blockedByPinned } = resolveVerticalOverlaps(
      page.regions,
      targetId
    );
    if (blockedByPinned) {
      placeOnNextPage(draft, pageIndex, targetId);
      return;
    }

    for (const [regionId, y] of positions) {
      const region = page.regions.find((item) => item.id === regionId);
      if (region) {
        region.y = y;
      }
    }

    const bottom = pageBottom(draft, page);
    const overflowing = page.regions.filter(
      (region) =>
        region.id !== targetId &&
        !hasRole(region) &&
        region.y + region.h > bottom
    );
    for (const region of overflowing) {
      placeOnNextPage(draft, pageIndex, region.id);
    }
  };

  return {
    doc: newDocument(),
    selectedIds: [],
    past: [],
    future: [],
    historyKey: null,
    historyAt: 0,
    transactionKey: null,
    recalcToken: 0,
    mathResults: {},

    setDoc: (doc) =>
      set({
        doc: produce(doc, (draft) => {
          const setup = draft.settings.page;
          for (const page of draft.pages) {
            for (const region of page.regions) {
              clampRegionToContent(region, page, setup);
            }
          }
        }),
        selectedIds: [],
        past: [],
        future: [],
        historyKey: null,
        historyAt: 0,
        transactionKey: null,
        mathResults: {}
      }),

    newDoc: () =>
      set({
        doc: newDocument(),
        selectedIds: [],
        past: [],
        future: [],
        historyKey: null,
        historyAt: 0,
        transactionKey: null,
        mathResults: {}
      }),

    addMathRegion: () =>
      mutate(null, (draft) => {
        const page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        const height = 48;
        const region: MathRegion = {
          id: id(),
          type: "math",
          x: draft.doc.settings.page.marginLeft,
          y: nextY(page, height, draft.doc.settings.page),
          w: Math.min(360, page.width - draft.doc.settings.page.marginLeft - draft.doc.settings.page.marginRight),
          h: height,
          source: "x^2",
          ascii: "x^2"
        };
        page.regions.push(region);
        draft.selectedIds = [region.id];
      }),

    addMathRegionWith: (source, ascii) =>
      mutate(null, (draft) => {
        const page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        const height = 48;
        const region: MathRegion = {
          id: id(),
          type: "math",
          x: draft.doc.settings.page.marginLeft,
          y: nextY(page, height, draft.doc.settings.page),
          w: Math.min(360, page.width - draft.doc.settings.page.marginLeft - draft.doc.settings.page.marginRight),
          h: height,
          source,
          ascii
        };
        page.regions.push(region);
        draft.selectedIds = [region.id];
      }),

    addMathRegionAt: (pageId, x, y) => {
      const regionId = id();
      let created = false;

      mutate(null, (draft) => {
        const page = draft.doc.pages.find(
          (candidate) => candidate.id === pageId
        );
        if (!page) {
          return;
        }

        const setup = draft.doc.settings.page;
        const contentWidth = Math.max(
          MIN_REGION_WIDTH,
          page.width - setup.marginLeft - setup.marginRight
        );
        const contentBottom = Math.max(
          setup.marginTop + MIN_REGION_HEIGHT,
          page.height - setup.marginBottom
        );
        const height = 48;
        const width = Math.min(360, contentWidth);
        const left = Math.round(
          Math.max(
            setup.marginLeft,
            Math.min(page.width - setup.marginRight - width, x)
          )
        );
        const top = Math.round(
          Math.max(
            setup.marginTop,
            Math.min(contentBottom - height, y)
          )
        );
        const region: MathRegion = {
          id: regionId,
          type: "math",
          x: left,
          y: top,
          w: width,
          h: height,
          source: "",
          ascii: ""
        };
        page.regions.push(region);
        draft.selectedIds = [regionId];
        created = true;
      });

      return created ? regionId : null;
    },

    addMathRegionBelow: (regionId) => {
      const newId = id();
      let created = false;

      mutate(null, (draft) => {
        const pageIndex = draft.doc.pages.findIndex((page) =>
          page.regions.some((region) => region.id === regionId)
        );
        if (pageIndex < 0) {
          return;
        }

        const page = draft.doc.pages[pageIndex];
        const source = page.regions.find((region) => region.id === regionId);
        if (!source) {
          return;
        }

        const gap = 16;
        const height = 48;
        const setup = draft.doc.settings.page;
        const contentRight = Math.max(
          setup.marginLeft + MIN_REGION_WIDTH,
          page.width - setup.marginRight
        );
        const contentBottom = Math.max(
          setup.marginTop + MIN_REGION_HEIGHT,
          page.height - setup.marginBottom
        );
        const width = Math.min(
          360,
          Math.max(120, contentRight - source.x)
        );
        const collisionWidth = 64;
        const newRegion = (): MathRegion => ({
          id: newId,
          type: "math",
          x: Math.max(
            setup.marginLeft,
            Math.min(source.x, contentRight - width)
          ),
          y: 0,
          w: width,
          h: height,
          source: "",
          ascii: ""
        });
        const region = newRegion();
        let y = source.y + source.h + gap;

        const overlaps = (candidate: Region) =>
          candidate.id !== source.id &&
          region.x < candidate.x + candidate.w &&
          region.x + collisionWidth > candidate.x &&
          y < candidate.y + candidate.h &&
          y + height > candidate.y;

        let guard = 0;
        while (page.regions.some(overlaps) && guard < 100) {
          const bottom = page.regions
            .filter(overlaps)
            .reduce(
              (max, candidate) => Math.max(max, candidate.y + candidate.h),
              y + height
            );
          y = bottom + gap;
          guard += 1;
        }

        if (y + height > contentBottom) {
          const nextPage: Page = {
            id: id(),
            width: page.width,
            height: page.height,
            regions: []
          };
          region.x = Math.max(
            setup.marginLeft,
            Math.min(
              source.x,
              Math.max(
                setup.marginLeft,
                nextPage.width - setup.marginRight - region.w
              )
            )
          );
          region.y = Math.max(setup.marginTop, 0);
          nextPage.regions.push(region);
          draft.doc.pages.splice(pageIndex + 1, 0, nextPage);
        } else {
          region.y = Math.round(y);
          page.regions.push(region);
        }

        draft.selectedIds = [newId];
        created = true;
      });

      return created ? newId : null;
    },

    addTextRegion: () =>
      mutate(null, (draft) => {
        const page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        const height = 100;
        const region: TextRegion = {
          id: id(),
          type: "text",
          x: draft.doc.settings.page.marginLeft,
          y: nextY(page, height, draft.doc.settings.page),
          w: Math.min(500, page.width - draft.doc.settings.page.marginLeft - draft.doc.settings.page.marginRight),
          h: height,
          html: "<p>Texto</p>"
        };
        page.regions.push(region);
        draft.selectedIds = [region.id];
      }),

    addHeaderRegion: () => addPageFurniture("header"),

    addFooterRegion: () => addPageFurniture("footer"),

    addImageRegion: (src, width = 320, height = 240) =>
      mutate(null, (draft) => {
        const page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        const setup = draft.doc.settings.page;
        const w = Math.min(
          width,
          page.width - setup.marginLeft - setup.marginRight
        );
        const h = Math.min(
          height,
          page.height - setup.marginTop - setup.marginBottom
        );
        const region: ImageRegion = {
          id: id(),
          type: "image",
          x: draft.doc.settings.page.marginLeft,
          y: nextY(page, h, draft.doc.settings.page),
          w,
          h,
          src
        };
        page.regions.push(region);
        draft.selectedIds = [region.id];
      }),

    addPlotRegion: () =>
      mutate(null, (draft) => {
        const page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        const setup = draft.doc.settings.page;
        const w = Math.min(
          480,
          page.width - setup.marginLeft - setup.marginRight
        );
        const h = Math.min(
          300,
          page.height - setup.marginTop - setup.marginBottom
        );
        const region: PlotRegion = {
          id: id(),
          type: "plot",
          x: draft.doc.settings.page.marginLeft,
          y: nextY(page, h, draft.doc.settings.page),
          w,
          h,
          series: [
            {
              expression: "sin(x)",
              expression2: "",
              kind: "explicit",
              parameterMin: -10,
              parameterMax: 10,
              color: "#2563eb",
              visible: true
            }
          ],
          xMin: -10,
          xMax: 10,
          yMin: null,
          yMax: null
        };
        page.regions.push(region);
        draft.selectedIds = [region.id];
      }),

    addPage: () =>
      mutate(null, (draft) => {
        const last = draft.doc.pages[draft.doc.pages.length - 1];
        draft.doc.pages.push({
          id: id(),
          width: last?.width ?? 850,
          height: last?.height ?? 1100,
          regions: []
        });
        draft.selectedIds = [];
      }),

    removePage: (pageId) => {
      if (get().doc.pages.length <= 1) {
        return;
      }

      mutate(null, (draft) => {
        draft.doc.pages = draft.doc.pages.filter((page) => page.id !== pageId);
        const remaining = new Set(
          draft.doc.pages.flatMap((page) =>
            page.regions.map((region) => region.id)
          )
        );
        draft.selectedIds = draft.selectedIds.filter((regionId) =>
          remaining.has(regionId)
        );
      });
    },

    movePage: (pageId, direction) => {
      const pages = get().doc.pages;
      const index = pages.findIndex((page) => page.id === pageId);
      const target = index + direction;

      if (index < 0 || target < 0 || target >= pages.length) {
        return;
      }

      mutate(null, (draft) => {
        const [page] = draft.doc.pages.splice(index, 1);
        draft.doc.pages.splice(target, 0, page);
      });
    },

    setPageSize: (width, height) =>
      mutate(null, (draft) => {
        for (const page of draft.doc.pages) {
          page.width = width;
          page.height = height;

          for (const region of page.regions) {
            clampRegionToContent(region, page, draft.doc.settings.page);
          }
        }
      }),

    updatePageSetup: (setup) =>
      mutate(null, (draft) => {
        draft.doc.settings.page = setup;
        for (const page of draft.doc.pages) {
          for (const region of page.regions) {
            clampRegionToContent(region, page, setup);
          }
        }
      }),

    updateDocumentLanguage: (language) =>
      mutate(null, (draft) => {
        draft.doc.metadata.language = language;
      }),

    updateResultUnits: (system, overrides) =>
      mutate(null, (draft) => {
        draft.doc.settings.units = system;
        draft.doc.settings.unitOverrides = { ...overrides };
      }),

    updateNumberFormat: (format) =>
      mutate(null, (draft) => {
        draft.doc.settings.notation = format.notation;
        draft.doc.settings.precision = Math.max(
          0,
          Math.min(12, Math.round(format.precision))
        );
        draft.doc.settings.fractions = format.fractions;
        draft.doc.settings.trimZeros = format.trimZeros;
      }),

    updateCustomConstants: (constants) =>
      mutate(null, (draft) => {
        draft.doc.settings.customConstants = constants.map((constant) => ({
          name: constant.name,
          expression: constant.expression,
          description: constant.description ?? ""
        }));
      }),

    addPersonalDictionaryWord: (word) => {
      const clean = word.trim();
      if (!clean) {
        return;
      }
      mutate(null, (draft) => {
        const dictionary = draft.doc.settings.personalDictionary ?? [];
        if (
          !dictionary.some(
            (entry) => entry.toLocaleLowerCase() === clean.toLocaleLowerCase()
          )
        ) {
          dictionary.push(clean);
          draft.doc.settings.personalDictionary = dictionary;
        }
      });
    },

    updateRegion: (regionId, patch) => {
      let key: string | null = null;

      if ("source" in patch || "ascii" in patch) {
        key = `math:${regionId}`;
      } else if ("html" in patch) {
        key = `text:${regionId}`;
      } else if ("x" in patch || "y" in patch) {
        key = `move:${regionId}`;
      } else if ("w" in patch || "h" in patch) {
        key = `resize:${regionId}`;
      } else {
        key = `update:${regionId}`;
      }

      mutate(key, (draft) => {
        for (const page of draft.doc.pages) {
          const index = page.regions.findIndex(
            (region) => region.id === regionId
          );
          if (index >= 0) {
            const current = page.regions[index];
            const effective = { ...patch };
            let candidate = {
              ...current,
              ...effective
            } as Region;

            if (!hasRole(current)) {
              clampRegionToContent(candidate, page, draft.doc.settings.page);
            }

            if (("x" in patch || "y" in patch) && !hasRole(current)) {
              const collides = page.regions.some(
                (other, otherIndex) =>
                  otherIndex !== index && rectsOverlap(candidate, other)
              );
              if (collides) {
                delete effective.x;
                delete effective.y;
                candidate = {
                  ...current,
                  ...effective
                } as Region;
                clampRegionToContent(
                  candidate,
                  page,
                  draft.doc.settings.page
                );
              }
            }

            page.regions[index] = candidate;
            break;
          }
        }
      });
    },

    syncRegionSize: (regionId, w, h) => {
      let width = Math.max(1, Math.round(w));
      let height = Math.max(1, Math.round(h));

      set((state) => {
        const pageIndex = state.doc.pages.findIndex((candidate) =>
          candidate.regions.some((region) => region.id === regionId)
        );
        if (pageIndex < 0) {
          return {};
        }
        const page = state.doc.pages[pageIndex];
        const region = page.regions.find(
          (candidate) => candidate.id === regionId
        );
        if (!region) {
          return {};
        }

        const setup = state.doc.settings.page;
        let targetX = region.x;
        if (!hasRole(region)) {
          const left = Math.max(0, setup.marginLeft);
          const right = Math.max(
            left + MIN_REGION_WIDTH,
            page.width - setup.marginRight
          );
          targetX = Math.max(
            left,
            Math.min(region.x, Math.max(left, right - MIN_REGION_WIDTH))
          );
          width = Math.min(width, Math.max(1, right - targetX));
        }

        if (
          region.x === targetX &&
          region.w === width &&
          region.h === height
        ) {
          return {};
        }

        const doc = produce(state.doc, (draft) => {
          const target = draft.pages[pageIndex]?.regions.find(
            (item) => item.id === regionId
          );
          if (!target) {
            return;
          }
          target.x = targetX;
          target.w = width;
          target.h = height;
          reflowAfterResize(draft, pageIndex, regionId);
        });
        return { doc };
      });
    },

    removeRegion: (regionId) =>
      mutate(null, (draft) => {
        for (const page of draft.doc.pages) {
          const index = page.regions.findIndex(
            (region) => region.id === regionId
          );
          if (index >= 0) {
            page.regions.splice(index, 1);
            break;
          }
        }

        draft.selectedIds = draft.selectedIds.filter(
          (selectedId) => selectedId !== regionId
        );
      }),

    removeSelected: () => {
      if (get().selectedIds.length === 0) {
        return;
      }

      mutate(null, (draft) => {
        const selected = new Set(draft.selectedIds);
        for (const page of draft.doc.pages) {
          page.regions = page.regions.filter(
            (region) => !selected.has(region.id)
          );
        }
        draft.selectedIds = [];
      });
    },

    duplicateSelected: () => {
      if (get().selectedIds.length === 0) {
        return;
      }

      mutate(null, (draft) => {
        const selected = new Set(draft.selectedIds);
        const clones: string[] = [];

        for (const page of draft.doc.pages) {
          const copied = page.regions
            .filter((region) => selected.has(region.id))
            .map((region) => {
              const clone = {
                ...region,
                id: id(),
                x: Math.min(region.x + 24, page.width - region.w),
                y: Math.min(region.y + 24, page.height - region.h)
              } as Region;
              clones.push(clone.id);
              return clone;
            });
          page.regions.push(...copied);
        }

        draft.selectedIds = clones;
      });
    },

    copySelected: () => {
      const state = get();
      const selected = new Set(state.selectedIds);
      regionClipboard = state.doc.pages
        .flatMap((page) =>
          page.regions.filter((region) => selected.has(region.id))
        )
        .map((region) => ({ ...region } as Region));
      return regionClipboard.length;
    },

    cutSelected: () => {
      const count = get().copySelected();
      if (count > 0) {
        get().removeSelected();
      }
      return count;
    },

    pasteClipboard: () => {
      if (regionClipboard.length === 0) {
        return 0;
      }

      mutate(null, (draft) => {
        let page = draft.doc.pages[draft.doc.pages.length - 1];
        if (!page) {
          return;
        }

        if (draft.selectedIds.length > 0) {
          const selected = new Set(draft.selectedIds);
          const selectedPage = draft.doc.pages.find((candidate) =>
            candidate.regions.some((region) => selected.has(region.id))
          );
          if (selectedPage) {
            page = selectedPage;
          }
        }

        const pasted = regionClipboard.map((region) => {
          const clone = { ...region, id: id() } as Region;
          clone.x = Math.max(
            0,
            Math.min(region.x + 24, page.width - clone.w)
          );
          clone.y = Math.max(
            0,
            Math.min(region.y + 24, page.height - clone.h)
          );
          return clone;
        });

        page.regions.push(...pasted);
        draft.selectedIds = pasted.map((region) => region.id);
      });

      return regionClipboard.length;
    },

    clearClipboard: () => {
      regionClipboard = [];
    },

    select: (selectedId, additive = false) =>
      set((state) => {
        if (!selectedId) {
          return { selectedIds: [] };
        }

        if (!additive) {
          return { selectedIds: [selectedId] };
        }

        const exists = state.selectedIds.includes(selectedId);
        return {
          selectedIds: exists
            ? state.selectedIds.filter((id) => id !== selectedId)
            : [...state.selectedIds, selectedId]
        };
      }),

    selectMany: (ids) => set({ selectedIds: [...new Set(ids)] }),

    beginGroupMove: (pageId) => {
      const state = get();
      const page = state.doc.pages.find((candidate) => candidate.id === pageId);
      if (!page) {
        return;
      }

      const selected = new Set(state.selectedIds);
      groupMoveSnapshot = page.regions
        .filter((region) => selected.has(region.id) && !region.pinned)
        .map((region) => ({
          id: region.id,
          x: region.x,
          y: region.y,
          w: region.w,
          h: region.h
        }));
      groupMovePageId = pageId;
    },

    moveGroup: (dx, dy, snap = 0) => {
      const snapshot = groupMoveSnapshot;
      const pageId = groupMovePageId;
      if (!snapshot || snapshot.length === 0 || !pageId) {
        return;
      }

      const page = get().doc.pages.find((candidate) => candidate.id === pageId);
      if (!page) {
        return;
      }

      let minX = Infinity;
      let minY = Infinity;
      let maxX = -Infinity;
      let maxY = -Infinity;
      for (const region of snapshot) {
        minX = Math.min(minX, region.x);
        minY = Math.min(minY, region.y);
        maxX = Math.max(maxX, region.x + region.w);
        maxY = Math.max(maxY, region.y + region.h);
      }

      const selected = new Set(snapshot.map((region) => region.id));
      const hasFurniture = page.regions.some(
        (region) =>
          selected.has(region.id) &&
          "role" in region &&
          Boolean(region.role)
      );
      const setup = get().doc.settings.page;
      const contentLeft = hasFurniture ? 0 : Math.max(0, setup.marginLeft);
      const contentTop = hasFurniture ? 0 : Math.max(0, setup.marginTop);
      const contentRight = hasFurniture
        ? page.width
        : Math.max(
            contentLeft + MIN_REGION_WIDTH,
            page.width - setup.marginRight
          );
      const contentBottom = hasFurniture
        ? page.height
        : Math.max(
            contentTop + MIN_REGION_HEIGHT,
            page.height - setup.marginBottom
          );

      const clampedX = Math.max(
        contentLeft - minX,
        Math.min(dx, contentRight - maxX)
      );
      const clampedY = Math.max(
        contentTop - minY,
        Math.min(dy, contentBottom - maxY)
      );
      const snapValue = (value: number) =>
        snap > 0 ? Math.round(value / snap) * snap : Math.round(value);
      const collides = page.regions.some((other) => {
        if (selected.has(other.id)) {
          return false;
        }
        return snapshot.some((region) => {
          const candidate = {
            ...region,
            x: Math.max(
              0,
              Math.min(page.width - region.w, snapValue(region.x + clampedX))
            ),
            y: Math.max(
              0,
              Math.min(page.height - region.h, snapValue(region.y + clampedY))
            )
          };
          return rectsOverlap(candidate, other);
        });
      });
      if (collides) {
        return;
      }

      mutate(null, (draft) => {
        const target = draft.doc.pages.find(
          (candidate) => candidate.id === pageId
        );
        if (!target) {
          return;
        }

        for (const region of snapshot) {
          const current = target.regions.find(
            (candidate) => candidate.id === region.id
          );
          if (current) {
            current.x = Math.max(
              0,
              Math.min(
                target.width - current.w,
                snapValue(region.x + clampedX)
              )
            );
            current.y = Math.max(
              0,
              Math.min(
                target.height - current.h,
                snapValue(region.y + clampedY)
              )
            );
          }
        }
      });
    },

    endGroupMove: () => {
      groupMoveSnapshot = null;
      groupMovePageId = null;
    },

    clearSelection: () => set({ selectedIds: [] }),

    beginTransaction: (key) =>
      set((state) => {
        if (state.transactionKey) {
          return state;
        }

        return {
          ...state,
          past: [...state.past.slice(-(HISTORY_LIMIT - 1)), state.doc],
          future: [],
          historyKey: key,
          historyAt: Date.now(),
          transactionKey: key
        };
      }),

    endTransaction: () =>
      set((state) =>
        state.transactionKey ? { ...state, transactionKey: null } : state
      ),

    undo: () =>
      set((state) => {
        const previous = state.past[state.past.length - 1];
        if (!previous) {
          return state;
        }

        return {
          ...state,
          doc: previous,
          past: state.past.slice(0, -1),
          future: [state.doc, ...state.future].slice(0, HISTORY_LIMIT),
          selectedIds: [],
          historyKey: null,
          historyAt: 0,
          transactionKey: null
        };
      }),

    redo: () =>
      set((state) => {
        const next = state.future[0];
        if (!next) {
          return state;
        }

        return {
          ...state,
          doc: next,
          past: [...state.past, state.doc].slice(-HISTORY_LIMIT),
          future: state.future.slice(1),
          selectedIds: [],
          historyKey: null,
          historyAt: 0,
          transactionKey: null
        };
      }),

    recalculate: () =>
      set((state) => ({ recalcToken: state.recalcToken + 1 })),

    setMathResults: (results) => set({ mathResults: results })
  };
});

export type { ImageRegion, MathRegion, TextRegion };
