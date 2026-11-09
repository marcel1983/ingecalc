import {
  useMemo,
  useState,
  type MouseEvent as ReactMouseEvent
} from "react";
import {
  buildFormulaSections,
  type FormulaSection
} from "../lib/formulaSidebar";
import type { PaletteItem, PaletteKind } from "../lib/palettes";
import { useTranslation } from "react-i18next";

const ORDER_KEY = "ingecalc-formula-order";

const PLOT_SECTION: FormulaSection = {
  id: "plots",
  title: "Gráficos",
  items: []
};

function readOrder(): string[] {
  try {
    const raw = window.localStorage.getItem(ORDER_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed)
      ? parsed.filter((value): value is string => typeof value === "string")
      : [];
  } catch {
    return [];
  }
}

type Props = {
  customUnits: PaletteItem[];
  onInsert: (item: PaletteItem) => void;
  onOpenPalette: (kind: PaletteKind) => void;
  onOpenUnitDialog: () => void;
  onAddPlot: () => void;
  onClose: () => void;
};

export function FormulaSidebar({
  customUnits,
  onInsert,
  onOpenPalette,
  onOpenUnitDialog,
  onAddPlot,
  onClose
}: Props) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [order, setOrder] = useState<string[]>(readOrder);
  const sections = useMemo(
    () => [...buildFormulaSections(customUnits), PLOT_SECTION],
    [customUnits]
  );
  const orderedSections = useMemo(() => {
    if (order.length === 0) {
      return sections;
    }
    const rank = new Map(order.map((id, index) => [id, index]));
    return [...sections].sort(
      (left, right) =>
        (rank.get(left.id) ?? Number.MAX_SAFE_INTEGER) -
        (rank.get(right.id) ?? Number.MAX_SAFE_INTEGER)
    );
  }, [order, sections]);
  const needle = query.trim().toLowerCase();
  const filtered = orderedSections
    .map((section) => ({
      ...section,
      items:
        section.id === "plots"
          ? []
          : section.items.filter((item) => {
              if (!needle) {
                return true;
              }
              return [item.label, item.description, item.category, item.latex]
                .join(" ")
                .toLowerCase()
                .includes(needle);
            })
    }))
    .filter((section) =>
      section.id === "plots" ? !needle : section.items.length > 0
    );

  const moveSection = (id: string, direction: -1 | 1) => {
    const ids = orderedSections.map((section) => section.id);
    const index = ids.indexOf(id);
    const target = index + direction;
    if (index < 0 || target < 0 || target >= ids.length) {
      return;
    }
    [ids[index], ids[target]] = [ids[target], ids[index]];
    setOrder(ids);
    window.localStorage.setItem(ORDER_KEY, JSON.stringify(ids));
  };

  const keepMathFocus = (event: ReactMouseEvent) => {
    event.preventDefault();
  };

  const sectionTitle = (section: FormulaSection) => {
    const key =
      section.id === "greek-lower"
        ? "greekLower"
        : section.id === "greek-upper"
          ? "greekUpper"
          : section.id;
    return t(`sections.${key}`, section.title);
  };

  return (
    <aside className="formula-sidebar" aria-label={t("sidebar.title")}>
      <div className="formula-sidebar-header">
        <div>
          <strong>{t("sidebar.title")}</strong>
          <span>{t("sidebar.hint")}</span>
        </div>
        <button
          type="button"
          className="formula-sidebar-close"
          title={t("sidebar.close")}
          aria-label={t("sidebar.close")}
          onClick={onClose}
        >
          ×
        </button>
      </div>

      <input
        className="formula-sidebar-search"
        type="search"
        value={query}
        placeholder="Buscar símbolo, función o unidad…"
        aria-label="Buscar en el panel de fórmulas"
        onChange={(event) => setQuery(event.target.value)}
      />

      <div className="formula-sidebar-scroll">
        {filtered.map((section, index) => (
          <section className="formula-section" key={section.id}>
            <div className="formula-section-title">
              <span>{sectionTitle(section)}</span>
              <div className="formula-section-controls">
                {section.id !== "plots" && (
                  <span className="formula-section-count">
                    {section.items.length}
                  </span>
                )}
                {!needle && (
                  <>
                    <button
                      type="button"
                      className="formula-move"
                      disabled={index === 0}
                      title="Subir panel"
                      aria-label={`Subir ${sectionTitle(section)}`}
                      onClick={() => moveSection(section.id, -1)}
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      className="formula-move"
                      disabled={index === filtered.length - 1}
                      title="Bajar panel"
                      aria-label={`Bajar ${sectionTitle(section)}`}
                      onClick={() => moveSection(section.id, 1)}
                    >
                      ↓
                    </button>
                  </>
                )}
              </div>
            </div>

            {section.id === "plots" ? (
              <button
                type="button"
                className="formula-action"
                onClick={onAddPlot}
              >
                + Gráfico 2D
              </button>
            ) : section.id === "assignment" ? (
              <div className="formula-assignment-grid">
                {section.items.map((item) => (
                  <button
                    type="button"
                    key={item.id}
                    className={`formula-assignment ${
                      item.id === "assign" ? "primary" : ""
                    }`}
                    title={`${item.description}\n${item.sample}`}
                    onMouseDown={keepMathFocus}
                    onClick={() => onInsert(item)}
                  >
                    <span className="formula-assignment-label">
                      {item.label}
                    </span>
                    <span className="formula-assignment-description">
                      {item.description}
                    </span>
                  </button>
                ))}
              </div>
            ) : (
              <div
                className={`formula-grid ${
                  section.id.startsWith("greek") ? "formula-grid-greek" : ""
                }`}
              >
                {section.items.map((item) => (
                  <button
                    type="button"
                    key={item.id}
                    className="formula-key"
                    title={`${item.description}\n${item.sample}`}
                    onMouseDown={keepMathFocus}
                    onClick={() => onInsert(item)}
                  >
                    <span className="formula-key-label">{item.label}</span>
                  </button>
                ))}
              </div>
            )}

            {section.morePalette && (
              <button
                type="button"
                className="formula-more"
                onClick={() => onOpenPalette(section.morePalette!)}
              >
                {section.morePalette === "function"
                  ? "Catálogo completo de funciones…"
                  : "Todas las unidades…"}
              </button>
            )}

            {section.id === "units" && (
              <button
                type="button"
                className="formula-more"
                onClick={onOpenUnitDialog}
              >
                Definir unidad propia…
              </button>
            )}
          </section>
        ))}
      </div>

      <p className="formula-sidebar-hint">
        Escribe <code>:=</code> o usa el botón. Nombres largos:{" "}
        <code>f_g</code> o <code>"fg"</code>.
      </p>
    </aside>
  );
}
