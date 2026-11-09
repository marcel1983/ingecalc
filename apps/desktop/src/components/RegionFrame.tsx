import {
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent
} from "react";
import type { Region } from "../types/document";
import {
  moveRect,
  moveRectInBounds,
  resizeRect,
  resizeRectInBounds,
  type ContentBounds,
  type Rect
} from "../lib/regionGeometry";
import {
  MIN_CONTENT_WIDTH_CHARS,
  isRegionElementEmpty
} from "../lib/regionContent";
import type { MathCompletion } from "../lib/mathCompletions";
import { GRID_SIZE } from "../lib/viewOptions";
import { MathRegion } from "./MathRegion";
import { PlotRegion } from "./PlotRegion";
import { TextRegion } from "./TextRegion";

type Props = {
  region: Region;
  pageWidth: number;
  pageHeight: number;
  contentBounds: ContentBounds;
  selected: boolean;
  snapToGrid: boolean;
  groupSelected: boolean;
  positionLocked: boolean;
  documentLanguage: string;
  personalDictionary: string[];
  mathCompletions: MathCompletion[];
  constantNames: string[];
  onSelect: (additive?: boolean) => void;
  onChange: (patch: Partial<Region>) => void;
  onSyncSize: (w: number, h: number) => void;
  onTogglePin: () => void;
  onEnter: () => void;
  onLanguageChange: (language: string) => void;
  onAddToDictionary: (word: string) => void;
  onRemove: () => void;
  onTransactionStart: (key: string) => void;
  onTransactionEnd: () => void;
  onGroupMoveStart: () => void;
  onGroupMove: (dx: number, dy: number) => void;
  onGroupMoveEnd: () => void;
};

type Interaction = {
  mode: "move" | "resize";
  pointerId: number;
  startClientX: number;
  startClientY: number;
  startRect: Rect;
  scale: number;
  transactionStarted: boolean;
  groupSelected: boolean;
};

export function RegionFrame({
  region,
  pageWidth,
  pageHeight,
  contentBounds,
  selected,
  snapToGrid,
  groupSelected,
  positionLocked,
  documentLanguage,
  personalDictionary,
  mathCompletions,
  constantNames,
  onSelect,
  onChange,
  onSyncSize,
  onTogglePin,
  onEnter,
  onLanguageChange,
  onAddToDictionary,
  onRemove,
  onTransactionStart,
  onTransactionEnd,
  onGroupMoveStart,
  onGroupMove,
  onGroupMoveEnd
}: Props) {
  const regionRef = useRef<HTMLDivElement>(null);
  const interaction = useRef<Interaction | null>(null);
  const removeTimer = useRef<number | null>(null);
  const onRemoveRef = useRef(onRemove);
  const onSyncSizeRef = useRef(onSyncSize);
  onRemoveRef.current = onRemove;
  onSyncSizeRef.current = onSyncSize;
  const [activeMode, setActiveMode] = useState<Interaction["mode"] | null>(
    null
  );
  const autoFitType =
    region.type === "math" || region.type === "text" ? region.type : null;
  const pinned = Boolean(region.pinned);
  const positionIsLocked = positionLocked || pinned;
  const isPageFurniture = "role" in region && Boolean(region.role);

  useEffect(
    () => () => {
      document.body.classList.remove("region-dragging", "region-resizing");
    },
    []
  );

  useEffect(() => {
    if (!autoFitType) {
      return;
    }

    const element = regionRef.current;
    if (!element) {
      return;
    }

    let frame = 0;
    const syncSize = () => {
      const width = Math.round(element.offsetWidth);
      const height = Math.round(element.offsetHeight);
      if (width > 0 && height > 0) {
        onSyncSizeRef.current(width, height);
      }
    };
    const observer = new ResizeObserver(() => {
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(syncSize);
    });
    observer.observe(element);
    syncSize();

    return () => {
      observer.disconnect();
      window.cancelAnimationFrame(frame);
    };
  }, [autoFitType, region.id]);

  useEffect(() => {
    if (!autoFitType) {
      return;
    }

    const element = regionRef.current;
    if (!element) {
      return;
    }

    const handleFocusOut = (event: FocusEvent) => {
      const next = event.relatedTarget as Node | null;
      if (next && element.contains(next)) {
        return;
      }

      if (removeTimer.current !== null) {
        window.clearTimeout(removeTimer.current);
      }
      removeTimer.current = window.setTimeout(() => {
        removeTimer.current = null;
        if (
          element.isConnected &&
          isRegionElementEmpty(element, autoFitType)
        ) {
          onRemoveRef.current();
        }
      }, 0);
    };

    element.addEventListener("focusout", handleFocusOut);
    return () => {
      element.removeEventListener("focusout", handleFocusOut);
      if (removeTimer.current !== null) {
        window.clearTimeout(removeTimer.current);
        removeTimer.current = null;
      }
    };
  }, [autoFitType]);

  const begin =
    (mode: Interaction["mode"]) =>
    (event: ReactPointerEvent<HTMLDivElement>) => {
      if (event.button !== 0) {
        return;
      }

      if (mode === "move" && positionIsLocked) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      if (mode === "resize" || !selected) {
        onSelect();
      }

      if (mode === "move" && groupSelected) {
        onGroupMoveStart();
      }

      const pageRect = regionRef.current?.parentElement?.getBoundingClientRect();
      const scale =
        pageRect && pageRect.width > 0 ? pageRect.width / pageWidth : 1;

      interaction.current = {
        mode,
        pointerId: event.pointerId,
        startClientX: event.clientX,
        startClientY: event.clientY,
        startRect: {
          x: region.x,
          y: region.y,
          w: region.w,
          h: region.h
        },
        scale: scale > 0 ? scale : 1,
        transactionStarted: false,
        groupSelected: mode === "move" && groupSelected
      };

      event.currentTarget.setPointerCapture(event.pointerId);
      document.body.classList.remove("region-dragging", "region-resizing");
      document.body.classList.add(
        mode === "move" ? "region-dragging" : "region-resizing"
      );
      setActiveMode(mode);
    };

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const current = interaction.current;
    if (!current || current.pointerId !== event.pointerId) {
      return;
    }

    const dx = (event.clientX - current.startClientX) / current.scale;
    const dy = (event.clientY - current.startClientY) / current.scale;
    const page = { width: pageWidth, height: pageHeight };

    if (!current.transactionStarted) {
      onTransactionStart(`${current.mode}:${region.id}`);
      current.transactionStarted = true;
    }

    if (current.mode === "move" && current.groupSelected) {
      onGroupMove(dx, dy);
      return;
    }

    if (current.mode === "move") {
      const next = isPageFurniture
        ? moveRect(
            current.startRect,
            dx,
            dy,
            page,
            snapToGrid ? GRID_SIZE : 0
          )
        : moveRectInBounds(
            current.startRect,
            dx,
            dy,
            page,
            contentBounds,
            snapToGrid ? GRID_SIZE : 0
          );
      onChange({ x: next.x, y: next.y });
    } else {
      const next = isPageFurniture
        ? resizeRect(
            current.startRect,
            dx,
            dy,
            page,
            snapToGrid ? GRID_SIZE : 0
          )
        : resizeRectInBounds(
            current.startRect,
            dx,
            dy,
            page,
            contentBounds,
            snapToGrid ? GRID_SIZE : 0
          );
      onChange({ w: next.w, h: next.h });
    }
  };

  const endInteraction = (event: ReactPointerEvent<HTMLDivElement>) => {
    const current = interaction.current;
    if (!current || current.pointerId !== event.pointerId) {
      return;
    }

    interaction.current = null;
    setActiveMode(null);
    document.body.classList.remove("region-dragging", "region-resizing");

    if (current.transactionStarted) {
      onTransactionEnd();
    }

    if (current.groupSelected) {
      onGroupMoveEnd();
    }

    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  const className = [
    "region",
    selected ? "selected" : "",
    activeMode ? `is-${activeMode}` : "",
    autoFitType ? "region-autofit" : "",
    positionIsLocked ? "region-position-locked" : ""
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      ref={regionRef}
      className={className}
      role="group"
      aria-label={regionLabel(region.type)}
      data-region-id={region.id}
      data-region-type={region.type}
      style={{
        left: region.x,
        top: region.y,
        ...(autoFitType
          ? {
              width: "max-content",
              height: "auto",
              minWidth: `${MIN_CONTENT_WIDTH_CHARS}ch`,
              maxWidth: Math.max(80, contentBounds.right - region.x)
            }
          : {
              width: region.w,
              height: region.h
            })
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
        onSelect(event.ctrlKey || event.metaKey);
      }}
    >
      {!positionIsLocked && (
        <div
          className="region-drag-handle"
          role="button"
          aria-label="Mover región"
          title="Arrastrar para mover"
          onPointerDown={begin("move")}
          onPointerMove={handlePointerMove}
          onPointerUp={endInteraction}
          onPointerCancel={endInteraction}
          onLostPointerCapture={endInteraction}
        />
      )}

      {pinned && (
        <div className="region-pin-badge" title="Posición fijada">
          📌
        </div>
      )}

      {selected && (
        <>
          {!autoFitType && (
            <div
              className="region-resize-handle"
              role="button"
              aria-label="Redimensionar región"
              title="Arrastrar para redimensionar"
              onPointerDown={begin("resize")}
              onPointerMove={handlePointerMove}
              onPointerUp={endInteraction}
              onPointerCancel={endInteraction}
              onLostPointerCapture={endInteraction}
            />
          )}
          <button
            type="button"
            className={`region-pin ${pinned ? "active" : ""}`}
            aria-label={pinned ? "Liberar posición" : "Fijar posición"}
            title={
              pinned
                ? "Liberar la posición de esta caja"
                : "Fijar la posición de esta caja"
            }
            onPointerDown={(event) => event.stopPropagation()}
            onClick={onTogglePin}
          >
            📌
          </button>
          <button
            type="button"
            className="region-delete"
            aria-label="Eliminar región"
            title="Eliminar región"
            onPointerDown={(event) => event.stopPropagation()}
            onClick={onRemove}
          >
            ×
          </button>
        </>
      )}

      {region.type === "math" && (
        <MathRegion
          region={region}
          onChange={(patch) => onChange(patch)}
          onEnter={onEnter}
          completions={mathCompletions}
          constantNames={constantNames}
        />
      )}
      {region.type === "text" && (
        <TextRegion
          region={region}
          selected={selected}
          documentLanguage={documentLanguage}
          personalDictionary={personalDictionary}
          onChange={(html) => onChange({ html })}
          onLanguageChange={onLanguageChange}
          onAddToDictionary={onAddToDictionary}
        />
      )}
      {region.type === "image" && (
        <img
          className="region-image"
          src={region.src}
          alt=""
          draggable={false}
        />
      )}
      {region.type === "plot" && (
        <PlotRegion region={region} onChange={(patch) => onChange(patch)} />
      )}
    </div>
  );
}

function regionLabel(type: Region["type"]): string {
  switch (type) {
    case "math":
      return "Región matemática";
    case "text":
      return "Región de texto";
    case "image":
      return "Región de imagen";
    case "plot":
      return "Región de gráfico 2D";
  }
}
