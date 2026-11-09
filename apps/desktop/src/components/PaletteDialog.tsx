import { useEffect, useMemo, useState } from "react";
import {
  PALETTES,
  PALETTE_TITLES,
  type PaletteItem,
  type PaletteKind
} from "../lib/palettes";

type Props = {
  kind: PaletteKind;
  extraItems?: PaletteItem[];
  onInsert: (item: PaletteItem) => void;
  onClose: () => void;
};

export function PaletteDialog({
  kind,
  extraItems = [],
  onInsert,
  onClose
}: Props) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("Todas");
  const items = useMemo(
    () => [...PALETTES[kind], ...(kind === "unit" ? extraItems : [])],
    [extraItems, kind]
  );

  const categories = useMemo(
    () => ["Todas", ...new Set(items.map((item) => item.category))],
    [items]
  );

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return items.filter((item) => {
      if (category !== "Todas" && item.category !== category) {
        return false;
      }
      if (!needle) {
        return true;
      }
      return [item.label, item.description, item.category, item.latex]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });
  }, [category, items, query]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div
      className="dialog-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        className="dialog palette-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="palette-title"
      >
        <h2 id="palette-title">{PALETTE_TITLES[kind]}</h2>
        <p className="dialog-hint">
          {kind === "function"
            ? "Elige una plantilla para insertarla en la región matemática activa. Si no hay ninguna activa, se crea una región nueva."
            : "Elige una unidad para insertarla en la región matemática activa. Las unidades se insertan como texto matemático."}
        </p>

        <input
          className="palette-search"
          type="search"
          autoFocus
          value={query}
          placeholder={
            kind === "function" ? "Buscar función…" : "Buscar unidad…"
          }
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && filtered[0]) {
              event.preventDefault();
              onInsert(filtered[0]);
            }
          }}
        />

        <div className="palette-categories" role="tablist">
          {categories.map((item) => (
            <button
              key={item}
              type="button"
              role="tab"
              aria-selected={category === item}
              className={category === item ? "active" : ""}
              onClick={() => setCategory(item)}
            >
              {item}
            </button>
          ))}
        </div>

        <div className="palette-grid">
          {filtered.map((item) => (
            <button
              key={item.id}
              type="button"
              className="palette-item"
              title={`${item.description}\n${item.sample}`}
              onClick={() => onInsert(item)}
            >
              <span className="palette-label">{item.label}</span>
              <span className="palette-description">{item.description}</span>
            </button>
          ))}
          {filtered.length === 0 && (
            <p className="palette-empty">No hay resultados.</p>
          )}
        </div>

        <div className="dialog-actions">
          <button type="button" className="secondary" onClick={onClose}>
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
}
