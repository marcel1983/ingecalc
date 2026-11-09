import {
  useEffect,
  useState,
  type PointerEvent as ReactPointerEvent
} from "react";
import { useDocumentStore } from "../stores/document";
import { useTranslation } from "react-i18next";
import type { MathCompletion } from "../lib/mathCompletions";
import { formatPageText, getPageSetup } from "../lib/pageSetup";
import { GRID_SIZE } from "../lib/viewOptions";
import { contentBounds as contentBoundsForPage } from "../lib/regionGeometry";
import { RegionFrame } from "./RegionFrame";

type Props = {
  zoom: number;
  showGrid: boolean;
  snapToGrid: boolean;
  positionLocked: boolean;
  documentLanguage: string;
  personalDictionary: string[];
  mathCompletions: MathCompletion[];
  constantNames: string[];
};

type Marquee = {
  pageId: string;
  pointerId: number;
  startX: number;
  startY: number;
  x: number;
  y: number;
  additive: boolean;
};

export function Worksheet({
  zoom,
  showGrid,
  snapToGrid,
  positionLocked,
  documentLanguage,
  personalDictionary,
  mathCompletions,
  constantNames
}: Props) {
  const { t } = useTranslation();
  const doc = useDocumentStore((state) => state.doc);
  const selectedIds = useDocumentStore((state) => state.selectedIds);
  const select = useDocumentStore((state) => state.select);
  const selectMany = useDocumentStore((state) => state.selectMany);
  const clearSelection = useDocumentStore((state) => state.clearSelection);
  const updateRegion = useDocumentStore((state) => state.updateRegion);
  const syncRegionSize = useDocumentStore((state) => state.syncRegionSize);
  const updateDocumentLanguage = useDocumentStore(
    (state) => state.updateDocumentLanguage
  );
  const addPersonalDictionaryWord = useDocumentStore(
    (state) => state.addPersonalDictionaryWord
  );
  const removeRegion = useDocumentStore((state) => state.removeRegion);
  const removeSelected = useDocumentStore((state) => state.removeSelected);
  const addPage = useDocumentStore((state) => state.addPage);
  const addMathRegionAt = useDocumentStore((state) => state.addMathRegionAt);
  const addMathRegionBelow = useDocumentStore(
    (state) => state.addMathRegionBelow
  );
  const removePage = useDocumentStore((state) => state.removePage);
  const movePage = useDocumentStore((state) => state.movePage);
  const beginGroupMove = useDocumentStore((state) => state.beginGroupMove);
  const moveGroup = useDocumentStore((state) => state.moveGroup);
  const endGroupMove = useDocumentStore((state) => state.endGroupMove);
  const beginTransaction = useDocumentStore(
    (state) => state.beginTransaction
  );
  const endTransaction = useDocumentStore((state) => state.endTransaction);
  const [marquee, setMarquee] = useState<Marquee | null>(null);

  const pagePoint = (
    event: ReactPointerEvent<HTMLDivElement>,
    width: number,
    height: number
  ) => {
    const rect = event.currentTarget.getBoundingClientRect();
    const scale = rect.width > 0 ? rect.width / width : 1;
    return {
      x: Math.max(0, Math.min(width, (event.clientX - rect.left) / scale)),
      y: Math.max(0, Math.min(height, (event.clientY - rect.top) / scale))
    };
  };

  const focusMathRegion = (regionId: string) => {
    window.requestAnimationFrame(() => {
      const field = document.querySelector<HTMLElement>(
        `[data-region-id="${regionId}"] math-field`
      );
      field?.focus();
      field?.scrollIntoView({ block: "nearest" });
    });
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target;
      const editable =
        target instanceof HTMLElement &&
        Boolean(
          target.closest(
            "input, textarea, [contenteditable='true'], math-field, .tiptap"
          )
        );

      if (event.key === "Tab") {
        const insideChrome =
          target instanceof HTMLElement &&
          Boolean(
            target.closest(
              ".toolbar, .draft-banner, .page-meta, input, textarea, [contenteditable='true'], math-field, .tiptap"
            )
          );

        if (insideChrome) {
          return;
        }

        const ids = doc.pages.flatMap((page) =>
          page.regions.map((region) => region.id)
        );

        if (ids.length === 0) {
          return;
        }

        event.preventDefault();
        const current = selectedIds.length
          ? ids.indexOf(selectedIds[selectedIds.length - 1])
          : -1;
        const direction = event.shiftKey ? -1 : 1;
        const nextIndex =
          current < 0
            ? direction > 0
              ? 0
              : ids.length - 1
            : (current + direction + ids.length) % ids.length;
        const nextId = ids[nextIndex];
        select(nextId);

        window.requestAnimationFrame(() => {
          document
            .querySelector(`[data-region-id="${nextId}"]`)
            ?.scrollIntoView({ block: "nearest" });
        });
        return;
      }

      if (selectedIds.length === 0) {
        return;
      }

      if (event.key !== "Delete" && event.key !== "Backspace") {
        return;
      }

      if (editable) {
        return;
      }

      event.preventDefault();
      removeSelected();
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedIds, removeSelected, doc.pages, select]);

  return (
    <div
      className="worksheet"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) {
          if (
            document.activeElement instanceof HTMLElement &&
            document.activeElement !== document.body
          ) {
            document.activeElement.blur();
          }
          clearSelection();
        }
      }}
    >
      {doc.pages.map((page, index) => {
        const setup = getPageSetup(doc.settings);
        const contentWidth = Math.max(
          0,
          page.width - setup.marginLeft - setup.marginRight
        );
        const contentHeight = Math.max(
          0,
          page.height - setup.marginTop - setup.marginBottom
        );
        const hasHeaderRegion = page.regions.some(
          (region) => region.type === "text" && region.role === "header"
        );
        const hasFooterRegion = page.regions.some(
          (region) => region.type === "text" && region.role === "footer"
        );

        return (
          <section
            key={page.id}
            className="page-shell"
            style={{ width: page.width * zoom }}
          >
          <div className="page-meta">
            <span>{t("page.page")} {index + 1}</span>
            <div className="page-actions">
              <button
                type="button"
                className="page-move"
                disabled={index === 0}
                title="Mover página arriba"
                onClick={() => movePage(page.id, -1)}
              >
                ↑
              </button>
              <button
                type="button"
                className="page-move"
                disabled={index === doc.pages.length - 1}
                title="Mover página abajo"
                onClick={() => movePage(page.id, 1)}
              >
                ↓
              </button>
              {doc.pages.length > 1 && (
                <button
                  type="button"
                  className="page-delete"
                  title="Eliminar página"
                  onClick={() => removePage(page.id)}
                >
                  {t("page.delete")}
                </button>
              )}
            </div>
          </div>

          <div
            className="page-scaler"
            style={{
              width: page.width * zoom,
              height: page.height * zoom
            }}
          >
            <div
              className={`page ${showGrid ? "page-grid" : ""}`}
              style={{
                width: page.width,
                height: page.height,
                transform: `scale(${zoom})`,
                transformOrigin: "top left"
              }}
              onPointerDown={(event) => {
                if (event.target !== event.currentTarget) {
                  return;
                }

                if (
                  document.activeElement instanceof HTMLElement &&
                  document.activeElement !== document.body
                ) {
                  document.activeElement.blur();
                }

                event.preventDefault();
                const point = pagePoint(event, page.width, page.height);
                const additive =
                  event.ctrlKey || event.metaKey || event.shiftKey;

                if (!additive) {
                  clearSelection();
                }

                setMarquee({
                  pageId: page.id,
                  pointerId: event.pointerId,
                  startX: point.x,
                  startY: point.y,
                  x: point.x,
                  y: point.y,
                  additive
                });
                event.currentTarget.setPointerCapture(event.pointerId);
              }}
              onPointerMove={(event) => {
                if (
                  !marquee ||
                  marquee.pageId !== page.id ||
                  marquee.pointerId !== event.pointerId
                ) {
                  return;
                }

                const point = pagePoint(event, page.width, page.height);
                setMarquee({ ...marquee, x: point.x, y: point.y });
              }}
              onPointerUp={(event) => {
                if (
                  !marquee ||
                  marquee.pageId !== page.id ||
                  marquee.pointerId !== event.pointerId
                ) {
                  return;
                }

                const left = Math.min(marquee.startX, marquee.x);
                const top = Math.min(marquee.startY, marquee.y);
                const right = Math.max(marquee.startX, marquee.x);
                const bottom = Math.max(marquee.startY, marquee.y);
                const moved = right - left >= 4 || bottom - top >= 4;

                if (moved) {
                  const ids = page.regions
                    .filter(
                      (region) =>
                        region.x < right &&
                        region.x + region.w > left &&
                        region.y < bottom &&
                        region.y + region.h > top
                    )
                    .map((region) => region.id);
                  selectMany(
                    marquee.additive
                      ? [...new Set([...selectedIds, ...ids])]
                      : ids
                  );
                } else if (!marquee.additive) {
                  const x = snapToGrid
                    ? Math.round(marquee.startX / GRID_SIZE) * GRID_SIZE
                    : marquee.startX;
                  const y = snapToGrid
                    ? Math.round(marquee.startY / GRID_SIZE) * GRID_SIZE
                    : marquee.startY;
                  const regionId = addMathRegionAt(page.id, x, y);
                  if (regionId) {
                    focusMathRegion(regionId);
                  } else {
                    clearSelection();
                  }
                }

                setMarquee(null);
                if (event.currentTarget.hasPointerCapture(event.pointerId)) {
                  event.currentTarget.releasePointerCapture(event.pointerId);
                }
              }}
              onPointerCancel={() => setMarquee(null)}
              onLostPointerCapture={() => setMarquee(null)}
            >
              <div
                className="page-margin-guide"
                style={{
                  left: setup.marginLeft,
                  top: setup.marginTop,
                  width: contentWidth,
                  height: contentHeight
                }}
              />

              {setup.header && !hasHeaderRegion && (
                <div
                  className="page-header"
                  style={{
                    left: setup.marginLeft,
                    right: setup.marginRight,
                    top: Math.max(4, setup.marginTop / 2)
                  }}
                >
                  {formatPageText(setup.header, index + 1)}
                </div>
              )}

              {setup.footer && !hasFooterRegion && (
                <div
                  className="page-footer"
                  style={{
                    left: setup.marginLeft,
                    right: setup.marginRight,
                    bottom: Math.max(4, setup.marginBottom / 2)
                  }}
                >
                  {formatPageText(setup.footer, index + 1)}
                </div>
              )}

              {page.regions.map((region) => (
                <RegionFrame
                  key={region.id}
                  region={region}
                  pageWidth={page.width}
                  pageHeight={page.height}
                  contentBounds={contentBoundsForPage(
                    { width: page.width, height: page.height },
                    {
                      top: setup.marginTop,
                      right: setup.marginRight,
                      bottom: setup.marginBottom,
                      left: setup.marginLeft
                    }
                  )}
                  selected={selectedIds.includes(region.id)}
                  snapToGrid={snapToGrid}
                  positionLocked={positionLocked}
                  documentLanguage={documentLanguage}
                  personalDictionary={personalDictionary}
                  mathCompletions={mathCompletions}
                  constantNames={constantNames}
                  groupSelected={
                    selectedIds.length > 1 &&
                    selectedIds.includes(region.id)
                  }
                  onSelect={(additive) => select(region.id, additive)}
                  onChange={(patch) => updateRegion(region.id, patch)}
                  onSyncSize={(w, h) => syncRegionSize(region.id, w, h)}
                  onTogglePin={() =>
                    updateRegion(region.id, { pinned: !region.pinned })
                  }
                  onEnter={() => {
                    const newId = addMathRegionBelow(region.id);
                    if (newId) {
                      focusMathRegion(newId);
                    }
                  }}
                  onLanguageChange={updateDocumentLanguage}
                  onAddToDictionary={addPersonalDictionaryWord}
                  onRemove={() => removeRegion(region.id)}
                  onTransactionStart={beginTransaction}
                  onTransactionEnd={endTransaction}
                  onGroupMoveStart={() => beginGroupMove(page.id)}
                  onGroupMove={(dx, dy) =>
                    moveGroup(dx, dy, snapToGrid ? GRID_SIZE : 0)
                  }
                  onGroupMoveEnd={endGroupMove}
                />
              ))}

              {marquee && marquee.pageId === page.id && (
                <div
                  className="region-marquee"
                  style={{
                    left: Math.min(marquee.startX, marquee.x),
                    top: Math.min(marquee.startY, marquee.y),
                    width: Math.abs(marquee.x - marquee.startX),
                    height: Math.abs(marquee.y - marquee.startY)
                  }}
                />
              )}
            </div>
          </div>
          </section>
        );
      })}

      <button type="button" className="add-page" onClick={addPage}>
        {t("page.add")}
      </button>
    </div>
  );
}
