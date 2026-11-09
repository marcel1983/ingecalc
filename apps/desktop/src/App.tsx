import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { PageSetupDialog } from "./components/PageSetupDialog";
import { PaletteDialog } from "./components/PaletteDialog";
import { UnitDialog } from "./components/UnitDialog";
import { Worksheet } from "./components/Worksheet";
import { FormulaSidebar } from "./components/FormulaSidebar";
import { ViewOptionsDialog } from "./components/ViewOptionsDialog";
import { UnitSettingsDialog } from "./components/UnitSettingsDialog";
import { NumberFormatDialog } from "./components/NumberFormatDialog";
import { ConstantsDialog } from "./components/ConstantsDialog";
import { HelpDialog } from "./components/HelpDialog";
import { getLastMathField, getSelectedLatex } from "./components/MathField";
import { latexToAscii } from "./components/InlineMath";
import { clearDraft, readDraft, writeDraft, type Draft } from "./lib/draft";
import { isTauri } from "./lib/environment";
import { evaluateDocument } from "./lib/api";
import { getPageSetup } from "./lib/pageSetup";
import { ASSIGNMENT_ITEM } from "./lib/formulaSidebar";
import { wrapSelectedMath } from "./lib/mathWrap";
import {
  readViewOptions,
  writeViewOptions,
  type ViewOptions
} from "./lib/viewOptions";
import type { PaletteItem, PaletteKind } from "./lib/palettes";
import { customUnitPalette, extractUserUnits } from "./lib/userUnits";
import { resultUnitSystemLabel } from "./lib/resultUnits";
import { normalizeNumberFormat } from "./lib/numberFormat";
import {
  buildMathCompletions,
  mathConstantNames
} from "./lib/mathCompletions";
import {
  detectLatexEngine,
  exportLatexPdf,
  storeLatexEngine
} from "./lib/latex";
import { exportPdf } from "./lib/pdf";
import { useDocumentStore } from "./stores/document";
import { useTranslation } from "react-i18next";
import i18n, { UI_LANGUAGES, normalizeUiLanguage } from "./lib/i18n";

type ShortcutActions = {
  save: () => void;
  open: () => void;
  newDoc: () => void;
  undo: () => void;
  redo: () => void;
  duplicate: () => void;
  recalc: () => void;
  print: () => void;
  copy: () => number;
  cut: () => number;
  paste: () => number;
};

const PAGE_PRESETS = [
  { id: "a4", label: "A4", width: 794, height: 1123 },
  { id: "letter", label: "Carta", width: 816, height: 1056 }
] as const;

const ZOOM_MIN = 0.4;
const ZOOM_MAX = 2;
const ZOOM_STEP = 0.1;

type RibbonTab =
  | "inicio"
  | "insertar"
  | "pagina"
  | "formulas"
  | "exportar"
  | "ayuda";
type Theme = "light" | "dark";

const RIBBON_TABS: Array<{ id: RibbonTab; key: string }> = [
  { id: "inicio", key: "tabs.home" },
  { id: "insertar", key: "tabs.insert" },
  { id: "pagina", key: "tabs.page" },
  { id: "formulas", key: "tabs.formulas" },
  { id: "exportar", key: "tabs.export" },
  { id: "ayuda", key: "tabs.help" }
];

export default function App() {
  const doc = useDocumentStore((state) => state.doc);
  const setDoc = useDocumentStore((state) => state.setDoc);
  const newDoc = useDocumentStore((state) => state.newDoc);
  const addMathRegion = useDocumentStore((state) => state.addMathRegion);
  const addMathRegionWith = useDocumentStore(
    (state) => state.addMathRegionWith
  );
  const addTextRegion = useDocumentStore((state) => state.addTextRegion);
  const addHeaderRegion = useDocumentStore((state) => state.addHeaderRegion);
  const addFooterRegion = useDocumentStore((state) => state.addFooterRegion);
  const addImageRegion = useDocumentStore((state) => state.addImageRegion);
  const addPlotRegion = useDocumentStore((state) => state.addPlotRegion);
  const addPage = useDocumentStore((state) => state.addPage);
  const setPageSize = useDocumentStore((state) => state.setPageSize);
  const updatePageSetup = useDocumentStore((state) => state.updatePageSetup);
  const updateDocumentLanguage = useDocumentStore(
    (state) => state.updateDocumentLanguage
  );
  const updateResultUnits = useDocumentStore(
    (state) => state.updateResultUnits
  );
  const updateNumberFormat = useDocumentStore(
    (state) => state.updateNumberFormat
  );
  const updateCustomConstants = useDocumentStore(
    (state) => state.updateCustomConstants
  );
  const selectedIds = useDocumentStore((state) => state.selectedIds);
  const duplicateSelected = useDocumentStore(
    (state) => state.duplicateSelected
  );
  const copySelected = useDocumentStore((state) => state.copySelected);
  const cutSelected = useDocumentStore((state) => state.cutSelected);
  const pasteClipboard = useDocumentStore((state) => state.pasteClipboard);
  const undo = useDocumentStore((state) => state.undo);
  const redo = useDocumentStore((state) => state.redo);
  const recalculate = useDocumentStore((state) => state.recalculate);
  const recalcToken = useDocumentStore((state) => state.recalcToken);
  const setMathResults = useDocumentStore((state) => state.setMathResults);
  const canUndo = useDocumentStore((state) => state.past.length > 0);
  const canRedo = useDocumentStore((state) => state.future.length > 0);

  const [status, setStatus] = useState("Listo");
  const [exportingPdf, setExportingPdf] = useState(false);
  const [pageSetupOpen, setPageSetupOpen] = useState(false);
  const [unitSettingsOpen, setUnitSettingsOpen] = useState(false);
  const [numberFormatOpen, setNumberFormatOpen] = useState(false);
  const [constantsOpen, setConstantsOpen] = useState(false);
  const [paletteKind, setPaletteKind] = useState<PaletteKind | null>(null);
  const [unitDialogOpen, setUnitDialogOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<RibbonTab>("inicio");
  const [helpOpen, setHelpOpen] = useState(false);
  const [helpSection, setHelpSection] = useState("inicio");
  const { t } = useTranslation();
  const [uiLanguage, setUiLanguage] = useState(() =>
    normalizeUiLanguage(
      globalThis.localStorage?.getItem("ingecalc-ui-language")
    )
  );
  const [theme, setTheme] = useState<Theme>(() => {
    const stored = window.localStorage.getItem("ingecalc-theme");
    if (stored === "light" || stored === "dark") {
      return stored;
    }
    return window.matchMedia?.("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  });
  const [zoom, setZoom] = useState(1);
  const [sidebarOpen, setSidebarOpen] = useState(
    () => window.localStorage.getItem("ingecalc-sidebar") !== "closed"
  );
  const [viewOptions, setViewOptions] = useState<ViewOptions>(readViewOptions);
  const [viewOptionsOpen, setViewOptionsOpen] = useState(false);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [draftResolved, setDraftResolved] = useState(false);
  const workspaceRef = useRef<HTMLElement | null>(null);
  const firstAutosave = useRef(true);
  const shortcuts = useRef<ShortcutActions | null>(null);

  useEffect(() => {
    let cancelled = false;

    readDraft()
      .then((stored) => {
        if (cancelled) {
          return;
        }
        if (stored) {
          setDraft(stored);
        } else {
          setDraftResolved(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDraftResolved(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("ingecalc-theme", theme);
  }, [theme]);

  useEffect(() => {
    window.localStorage.setItem(
      "ingecalc-sidebar",
      sidebarOpen ? "open" : "closed"
    );
  }, [sidebarOpen]);

  useEffect(() => {
    writeViewOptions(viewOptions);
  }, [viewOptions]);

  useEffect(() => {
    void i18n.changeLanguage(uiLanguage);
    globalThis.localStorage?.setItem("ingecalc-ui-language", uiLanguage);
    const language = UI_LANGUAGES.find((item) => item.id === uiLanguage);
    document.documentElement.lang = uiLanguage;
    document.documentElement.dir = language?.dir ?? "ltr";
  }, [uiLanguage]);

  useEffect(() => {
    if (!draftResolved) {
      return;
    }

    if (firstAutosave.current) {
      firstAutosave.current = false;
      return;
    }

    const timer = window.setTimeout(() => {
      writeDraft(doc).catch((error) => {
        setStatus(`No se pudo autoguardar: ${String(error)}`);
      });
    }, 1200);

    return () => window.clearTimeout(timer);
  }, [doc, draftResolved]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let cancelled = false;
    const timer = window.setTimeout(async () => {
      try {
        const results = await evaluateDocument(doc);
        if (cancelled) {
          return;
        }
        setMathResults(
          Object.fromEntries(
            results.map((result) => [
              result.id,
              {
                display: result.value ?? "",
                error: result.error ?? ""
              }
            ])
          )
        );
      } catch (error) {
        if (!cancelled) {
          setMathResults({});
          setStatus(`Error de evaluación: ${String(error)}`);
        }
      }
    }, 240);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [doc, recalcToken, setMathResults]);

  useEffect(() => {
    const handleBeforeUnload = () => {
      if (!draftResolved) {
        return;
      }

      void writeDraft(doc).catch(() => {
        // Closing should not be blocked by a failed draft write.
      });
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [doc, draftResolved]);

  const printPageWidth = doc.pages[0]?.width;
  const printPageHeight = doc.pages[0]?.height;

  useEffect(() => {
    if (!printPageWidth || !printPageHeight) {
      return;
    }

    const style = document.createElement("style");
    style.textContent = `@media print { @page { size: ${Math.round(
      printPageWidth
    )}px ${Math.round(printPageHeight)}px; margin: 0; } }`;
    document.head.appendChild(style);

    return () => style.remove();
  }, [printPageWidth, printPageHeight]);

  const resolveDraft = () => {
    void clearDraft();
    setDraft(null);
    setDraftResolved(true);
  };

  const handleNew = () => {
    newDoc();
    resolveDraft();
    setStatus("Documento nuevo");
  };

  const handleOpen = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "IngeCalc", extensions: ["smj", "json"] }]
      });

      if (!path || Array.isArray(path)) {
        return;
      }

      const loaded = await invoke<typeof doc>("load_document", { path });
      setDoc(loaded);
      resolveDraft();
      setStatus("Documento abierto");
    } catch (error) {
      setStatus(`Error al abrir: ${String(error)}`);
    }
  };

  const handleSave = async () => {
    try {
      const path = await save({
        filters: [{ name: "IngeCalc", extensions: ["smj"] }]
      });

      if (!path) {
        return;
      }

      await invoke("save_document", { path, document: doc });
      resolveDraft();
      setStatus("Documento guardado");
    } catch (error) {
      setStatus(`Error al guardar: ${String(error)}`);
    }
  };

  const handleExport = async (format: "latex" | "odt") => {
    try {
      const extension = format === "latex" ? "tex" : "odt";
      const baseName = (doc.metadata.title || "document").replace(
        /[\\/:*?"<>|]/g,
        "_"
      );
      const path = await save({
        defaultPath: `${baseName}.${extension}`,
        filters: [
          {
            name: format === "latex" ? "LaTeX" : "ODT",
            extensions: [extension]
          }
        ]
      });

      if (!path) {
        setStatus("Exportación cancelada");
        return;
      }

      const finalPath = path.toLowerCase().endsWith(`.${extension}`)
        ? path
        : `${path}.${extension}`;

      await invoke("export_document", {
        document: doc,
        format,
        path: finalPath
      });
      setStatus(`Exportado a ${format === "latex" ? "LaTeX" : "ODT"}`);
    } catch (error) {
      setStatus(`Error al exportar: ${String(error)}`);
    }
  };

  const handlePdfExport = async () => {
    setExportingPdf(true);
    setStatus("Generando PDF raster...");
    try {
      const exported = await exportPdf(doc);
      setStatus(exported ? "PDF raster exportado" : "Exportación cancelada");
    } catch (error) {
      setStatus(`Error al exportar PDF raster: ${String(error)}`);
    } finally {
      setExportingPdf(false);
    }
  };

  const handleLatexPdfExport = async () => {
    if (!isTauri()) {
      setStatus("La exportación PDF LaTeX requiere la app de escritorio");
      return;
    }

    setExportingPdf(true);
    setStatus("Preparando LaTeX...");

    try {
      let engine = await detectLatexEngine();

      if (!engine) {
        const picked = await open({
          multiple: false,
          title: "Selecciona xelatex, lualatex o pdflatex"
        });

        if (!picked || Array.isArray(picked)) {
          setStatus("No se configuró un motor LaTeX");
          return;
        }

        engine = picked;
        storeLatexEngine(engine);
      }

      const baseName = (doc.metadata.title || "document").replace(
        /[\\/:*?"<>|]/g,
        "_"
      );
      const path = await save({
        defaultPath: `${baseName}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }]
      });

      if (!path) {
        setStatus("Exportación cancelada");
        return;
      }

      setStatus("Generando PDF vectorial...");
      await exportLatexPdf(doc, path, engine);
      setStatus("PDF vectorial exportado");
    } catch (error) {
      setStatus(`Error al exportar PDF vectorial: ${String(error)}`);
    } finally {
      setExportingPdf(false);
    }
  };

  const configureLatexEngine = async () => {
    if (!isTauri()) {
      setStatus("La configuración de LaTeX requiere la app de escritorio");
      return;
    }

    try {
      const picked = await open({
        multiple: false,
        title: "Selecciona el ejecutable de XeLaTeX, LuaLaTeX o pdfLaTeX"
      });

      if (!picked || Array.isArray(picked)) {
        setStatus("Configuración de LaTeX cancelada");
        return;
      }

      storeLatexEngine(picked);
      setStatus(`Motor LaTeX configurado: ${picked}`);
    } catch (error) {
      setStatus(`Error al configurar LaTeX: ${String(error)}`);
    }
  };

  const handleAddImage = async () => {
    if (!isTauri()) {
      setStatus("La inserción de imágenes requiere la app de escritorio");
      return;
    }

    setStatus("Seleccionando imagen...");
    try {
      const path = await open({
        multiple: false,
        filters: [
          {
            name: "Imágenes",
            extensions: ["png", "jpg", "jpeg", "gif", "webp", "svg"]
          }
        ]
      });

      if (!path || Array.isArray(path)) {
        setStatus("Inserción cancelada");
        return;
      }

      const src = await invoke<string>("read_image", { path });
      const size = await measureImage(src);
      addImageRegion(src, size.width, size.height);
      setStatus("Imagen insertada");
    } catch (error) {
      setStatus(`Error al insertar imagen: ${String(error)}`);
    }
  };

  const handleAddPage = () => {
    addPage();
    setStatus("Página añadida");
  };

  const handleAddPlot = () => {
    addPlotRegion();
    setStatus("Gráfico 2D añadido");
  };

  const handleAddHeaderRegion = () => {
    addHeaderRegion();
    setStatus("Región de encabezado añadida");
  };

  const handleAddFooterRegion = () => {
    addFooterRegion();
    setStatus("Región de pie de página añadida");
  };

  const insertPaletteItem = (item: PaletteItem) => {
    const target = getLastMathField();

    if (target?.isConnected) {
      const selected = getSelectedLatex();
      const wrapped = selected
        ? wrapSelectedMath(item.latex, selected)
        : null;
      target.focus();

      if (wrapped) {
        target.insert(wrapped, {
          mode: "math",
          format: "latex",
          insertionMode: "replaceSelection",
          selectionMode: wrapped.includes("\\placeholder{}")
            ? "placeholder"
            : "after",
          focus: true,
          scrollIntoView: true
        });
        setStatus(`Aplicado a la selección: ${item.label}`);
        return;
      }

      target.insert(item.latex, {
        mode: "math",
        format: "latex",
        selectionMode: "placeholder",
        focus: true,
        scrollIntoView: true
      });
      setStatus(`Insertado: ${item.label}`);
      return;
    }

    addMathRegionWith(item.sample, latexToAscii(item.sample));
    setStatus(`Región matemática: ${item.label}`);
  };

  const handlePaletteInsert = (item: PaletteItem) => {
    insertPaletteItem(item);
    setPaletteKind(null);
  };

  const handleAssignmentInsert = () => {
    insertPaletteItem(ASSIGNMENT_ITEM);
  };

  const handleCreateUnit = (name: string, definition: string) => {
    const source = `unit(\\text{${name}}) := ${definition}`;
    const ascii = `unit("${name}") := ${definition}`;
    addMathRegionWith(source, ascii);
    setUnitDialogOpen(false);
    setStatus(`Unidad creada: ${name}`);
  };

  const customUnits = customUnitPalette(extractUserUnits(doc));

  const handlePrint = () => {
    setStatus("Preparando impresión...");
    window.setTimeout(() => {
      window.print();
      setStatus("Listo");
    }, 50);
  };

  const handleRecalculate = () => {
    recalculate();
    setStatus("Documento recalculado");
  };

  const handleDuplicate = () => {
    duplicateSelected();
    setStatus("Regiones duplicadas");
  };

  const handleUndo = () => {
    undo();
    setStatus("Deshecho");
  };

  const handleRedo = () => {
    redo();
    setStatus("Rehecho");
  };

  const handleCopy = () => {
    const count = copySelected();
    setStatus(
      count > 0 ? regionCount(count, "copiada") : "Nada que copiar"
    );
    return count;
  };

  const handleCut = () => {
    const count = cutSelected();
    setStatus(
      count > 0 ? regionCount(count, "cortada") : "Nada que cortar"
    );
    return count;
  };

  const handlePaste = () => {
    const count = pasteClipboard();
    setStatus(
      count > 0 ? regionCount(count, "pegada") : "Portapapeles vacío"
    );
    return count;
  };

  const fitZoom = () => {
    const available = workspaceRef.current?.clientWidth ?? window.innerWidth;
    const pageWidth = doc.pages[0]?.width ?? 850;
    const next = (available - 64) / pageWidth;
    setZoom(Math.max(ZOOM_MIN, Math.min(1.5, next)));
  };

  const recoverDraft = () => {
    if (!draft) {
      return;
    }

    // The draft is rewritten by autosave; do not clear it before the recovered
    // document has been persisted, or a quick close could lose it.
    firstAutosave.current = false;
    setDoc(draft.document);
    setDraft(null);
    setDraftResolved(true);
    setStatus("Borrador recuperado");
  };

  const discardDraft = () => {
    resolveDraft();
    setStatus("Borrador descartado");
  };

  shortcuts.current = {
    save: handleSave,
    open: handleOpen,
    newDoc: handleNew,
    undo: handleUndo,
    redo: handleRedo,
    duplicate: handleDuplicate,
    recalc: handleRecalculate,
    print: handlePrint,
    copy: handleCopy,
    cut: handleCut,
    paste: handlePaste
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const actions = shortcuts.current;
      if (!actions) {
        return;
      }

      const modifier = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();
      const target = event.target;
      const editable =
        target instanceof HTMLElement &&
        Boolean(
          target.closest(
            "input, textarea, [contenteditable='true'], math-field, .tiptap"
          )
        );

      if (event.key === "F1") {
        event.preventDefault();
        setHelpSection("inicio");
        setHelpOpen(true);
        return;
      }

      if (modifier && key === "s") {
        event.preventDefault();
        actions.save();
        return;
      }

      if (modifier && key === "o") {
        event.preventDefault();
        actions.open();
        return;
      }

      if (modifier && key === "n") {
        event.preventDefault();
        actions.newDoc();
        return;
      }

      if (modifier && (event.key === "+" || event.key === "=")) {
        event.preventDefault();
        setZoom((value) =>
          Math.min(ZOOM_MAX, roundZoom(value + ZOOM_STEP))
        );
        return;
      }

      if (modifier && event.key === "-") {
        event.preventDefault();
        setZoom((value) =>
          Math.max(ZOOM_MIN, roundZoom(value - ZOOM_STEP))
        );
        return;
      }

      if (modifier && event.key === "0") {
        event.preventDefault();
        setZoom(1);
        return;
      }

      if (event.key === "F9") {
        event.preventDefault();
        actions.recalc();
        return;
      }

      if (modifier && key === "p") {
        event.preventDefault();
        actions.print();
        return;
      }

      if (editable) {
        return;
      }

      if (modifier && key === "c") {
        event.preventDefault();
        const count = actions.copy();
        setStatus(
          count > 0 ? regionCount(count, "copiada") : "Nada que copiar"
        );
        return;
      }

      if (modifier && key === "x") {
        event.preventDefault();
        const count = actions.cut();
        setStatus(
          count > 0 ? regionCount(count, "cortada") : "Nada que cortar"
        );
        return;
      }

      if (modifier && key === "v") {
        event.preventDefault();
        const count = actions.paste();
        setStatus(
          count > 0 ? regionCount(count, "pegada") : "Portapapeles vacío"
        );
        return;
      }

      if (modifier && key === "z") {
        event.preventDefault();
        if (event.shiftKey) {
          actions.redo();
        } else {
          actions.undo();
        }
        return;
      }

      if (modifier && key === "y") {
        event.preventDefault();
        actions.redo();
        return;
      }

      if (modifier && key === "d") {
        event.preventDefault();
        actions.duplicate();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const firstPage = doc.pages[0];
  const allSameSize =
    firstPage !== undefined &&
    doc.pages.every(
      (page) =>
        page.width === firstPage.width && page.height === firstPage.height
    );
  const currentPreset = firstPage
    ? PAGE_PRESETS.find(
        (preset) =>
          preset.width === firstPage.width &&
          preset.height === firstPage.height
      )
    : undefined;
  const pageSizeValue =
    allSameSize && currentPreset ? currentPreset.id : "custom";
  const pageSizeLabel = firstPage
    ? `${Math.round(firstPage.width)}×${Math.round(firstPage.height)}`
    : "—";
  const numberFormat = normalizeNumberFormat(doc.settings);
  const openHelp = (section = "inicio") => {
    setHelpSection(section);
    setHelpOpen(true);
  };

  const mathCompletions = useMemo(() => buildMathCompletions(doc), [doc]);
  const constantNames = useMemo(() => mathConstantNames(doc), [doc]);

  return (
    <div className="app">
      <header className="ribbon">
        <div className="ribbon-titlebar">
          <div className="brand">IngeCalc</div>

          <div className="ribbon-quick">
            <button onClick={handleNew} title="Nuevo documento (Ctrl+N)">
              {t("quick.new")}
            </button>
            <button onClick={handleOpen} title="Abrir documento (Ctrl+O)">
              {t("quick.open")}
            </button>
            <button onClick={handleSave} title="Guardar documento (Ctrl+S)">
              {t("quick.save")}
            </button>
            <button
              onClick={handleUndo}
              disabled={!canUndo}
              title="Deshacer (Ctrl+Z)"
            >
              ↶
            </button>
            <button
              onClick={handleRedo}
              disabled={!canRedo}
              title="Rehacer (Ctrl+Y)"
            >
              ↷
            </button>
          </div>

          <div className="ribbon-spacer" />
          <div className="status" title={status}>
            {status}
          </div>
          <button
            type="button"
            className={`panel-toggle ${sidebarOpen ? "active" : ""}`}
            title={
              sidebarOpen
                ? "Ocultar panel de fórmulas"
                : "Mostrar panel de fórmulas"
            }
            aria-label={
              sidebarOpen
                ? "Ocultar panel de fórmulas"
                : "Mostrar panel de fórmulas"
            }
            aria-pressed={sidebarOpen}
            onClick={() => setSidebarOpen((current) => !current)}
          >
            ▤
          </button>
          <button
            type="button"
            className={`panel-toggle ${viewOptions.lockPositions ? "active" : ""}`}
            title={
              viewOptions.lockPositions
                ? "Liberar la posición de las cajas"
                : "Fijar la posición de las cajas"
            }
            aria-label={
              viewOptions.lockPositions
                ? "Liberar la posición de las cajas"
                : "Fijar la posición de las cajas"
            }
            aria-pressed={viewOptions.lockPositions}
            onClick={() => {
              const lockPositions = !viewOptions.lockPositions;
              setViewOptions({ ...viewOptions, lockPositions });
              setStatus(
                lockPositions ? "Posiciones fijadas" : "Posiciones liberadas"
              );
            }}
          >
            📌
          </button>
          <button
            type="button"
            className="theme-toggle help-toggle"
            title={t("help.open")}
            aria-label={t("help.open")}
            onClick={() => openHelp("inicio")}
          >
            ?
          </button>
          <button
            type="button"
            className="theme-toggle"
            title={
              theme === "dark" ? "Cambiar a modo claro" : "Cambiar a modo oscuro"
            }
            aria-label={
              theme === "dark" ? "Cambiar a modo claro" : "Cambiar a modo oscuro"
            }
            onClick={() =>
              setTheme((current) => (current === "dark" ? "light" : "dark"))
            }
          >
            {theme === "dark" ? "☀" : "☾"}
          </button>
          <select
            className="ui-language-select"
            value={uiLanguage}
            title={t("actions.language", "Idioma de la interfaz")}
            aria-label={t("actions.language", "Idioma de la interfaz")}
            onChange={(event) =>
              setUiLanguage(normalizeUiLanguage(event.target.value))
            }
          >
            {UI_LANGUAGES.map((language) => (
              <option key={language.id} value={language.id}>
                {language.native}
              </option>
            ))}
          </select>
        </div>

        <nav className="ribbon-tabs" role="tablist" aria-label="Ribbon">
          {RIBBON_TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={activeTab === tab.id}
              className={activeTab === tab.id ? "active" : ""}
              onClick={() => setActiveTab(tab.id)}
            >
              {t(tab.key)}
            </button>
          ))}
        </nav>

        <div className="ribbon-panel" role="tabpanel">
          {activeTab === "inicio" && (
            <>
              <RibbonGroup label={t("groups.clipboard")}>
                <button
                  onClick={handleCopy}
                  disabled={selectedIds.length === 0}
                  title="Copiar regiones (Ctrl+C)"
                >{t("actions.copy")}</button>
                <button onClick={handlePaste} title="Pegar regiones (Ctrl+V)">{t("actions.paste")}</button>
                <button
                  onClick={handleDuplicate}
                  disabled={selectedIds.length === 0}
                  title="Duplicar selección (Ctrl+D)"
                >{t("actions.duplicate")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.regions")}>
                <button onClick={addMathRegion} title="Añadir región matemática">{t("actions.addMath")}</button>
                <button onClick={addTextRegion} title="Añadir región de texto">{t("actions.addText")}</button>
                <button onClick={handleAddImage} title="Insertar imagen">{t("actions.addImage")}</button>
                <button onClick={handleAddPlot} title="Insertar gráfico 2D">{t("actions.addPlot")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.document")}>
                <button onClick={handleAddPage} title="Añadir página">{t("actions.addPage")}</button>
                <button
                  onClick={() => setPageSetupOpen(true)}
                  title="Márgenes, encabezado y pie de página"
                >{t("actions.pageSetup")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.calculation")}>
                <button onClick={handleRecalculate} title="Recalcular (F9)">{t("actions.recalculate")}</button>
              </RibbonGroup>
            </>
          )}

          {activeTab === "insertar" && (
            <>
              <RibbonGroup label={t("groups.regions")}>
                <button onClick={addMathRegion} title="Añadir región matemática">{t("actions.addMath")}</button>
                <button onClick={addTextRegion} title="Añadir región de texto">{t("actions.addText")}</button>
                <button onClick={handleAddImage} title="Insertar imagen">{t("actions.addImage")}</button>
                <button onClick={handleAddPlot} title="Insertar gráfico 2D">{t("actions.addPlot")}</button>
                <button onClick={handleAddPage} title="Añadir página">{t("actions.addPage")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.page")}>
                <button
                  onClick={handleAddHeaderRegion}
                  title="Añadir una región de encabezado"
                >{t("page.header")}</button>
                <button
                  onClick={handleAddFooterRegion}
                  title="Añadir una región de pie de página"
                >{t("page.footer")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.math")}>
                <button
                  onClick={() => setPaletteKind("function")}
                  title="Insertar función o plantilla matemática"
                >{t("actions.functions")}</button>
                <button
                  onClick={() => setPaletteKind("unit")}
                  title="Insertar unidad"
                >{t("actions.unitPalette")}</button>
                <button
                  onClick={() => setUnitDialogOpen(true)}
                  title="Definir una unidad propia"
                >{t("actions.customUnit")}</button>
                <button
                  onClick={() => setConstantsOpen(true)}
                  title="Constantes de ingeniería y constantes propias"
                >{t("actions.constants")}</button>
              </RibbonGroup>
            </>
          )}

          {activeTab === "pagina" && (
            <>
              <RibbonGroup label={t("groups.design")}>
                <select
                  className="page-size"
                  value={pageSizeValue}
                  onChange={(event) => {
                    const preset = PAGE_PRESETS.find(
                      (item) => item.id === event.target.value
                    );
                    if (!preset) {
                      return;
                    }
                    setPageSize(preset.width, preset.height);
                    setStatus(`Tamaño de página: ${preset.label}`);
                  }}
                  title={`Tamaño de página actual: ${pageSizeLabel}`}
                >
                  <option value="custom" disabled>
                    Personalizado ({pageSizeLabel})
                  </option>
                  {PAGE_PRESETS.map((preset) => (
                    <option key={preset.id} value={preset.id}>
                      {preset.label}
                    </option>
                  ))}
                </select>
                <button
                  onClick={() => setPageSetupOpen(true)}
                  title="Márgenes, encabezado y pie de página"
                >{t("actions.pageSetup")}</button>
                <button
                  onClick={() => setUnitSettingsOpen(true)}
                  title={`Unidades de las respuestas: ${resultUnitSystemLabel(
                    doc.settings.units
                  )}`}
                >{t("actions.units")}</button>
                <button
                  onClick={() => setNumberFormatOpen(true)}
                  title="Notación, decimales y fracciones de los resultados"
                >{t("actions.numbers")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.pages")}>
                <button onClick={handleAddPage} title="Añadir página">{t("actions.addPage")}</button>
                <button
                  onClick={handleAddHeaderRegion}
                  title="Añadir una región de encabezado"
                >{t("page.header")}</button>
                <button
                  onClick={handleAddFooterRegion}
                  title="Añadir una región de pie de página"
                >{t("page.footer")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.view")}>
                <button
                  onClick={() =>
                    setZoom((value) =>
                      Math.max(ZOOM_MIN, roundZoom(value - ZOOM_STEP))
                    )
                  }
                  title="Alejar"
                >
                  −
                </button>
                <button
                  className="zoom-value"
                  onClick={() => setZoom(1)}
                  title="Restablecer zoom al 100% (Ctrl+0)"
                >
                  {Math.round(zoom * 100)}%
                </button>
                <button
                  onClick={() =>
                    setZoom((value) =>
                      Math.min(ZOOM_MAX, roundZoom(value + ZOOM_STEP))
                    )
                  }
                  title="Acercar"
                >
                  +
                </button>
                <button onClick={fitZoom} title="Ajustar a la ventana">{t("actions.fit")}</button>
                <label
                  className="ribbon-number-field"
                  title="Número de decimales de los resultados"
                >
                  <span>Decimales</span>
                  <input
                    type="number"
                    min={0}
                    max={12}
                    value={numberFormat.precision}
                    onChange={(event) =>
                      updateNumberFormat({
                        ...numberFormat,
                        precision: Math.max(
                          0,
                          Math.min(12, Number(event.target.value) || 0)
                        )
                      })
                    }
                  />
                </label>
                <button
                  className={viewOptions.lockPositions ? "active" : ""}
                  onClick={() => {
                    const lockPositions = !viewOptions.lockPositions;
                    setViewOptions({ ...viewOptions, lockPositions });
                    setStatus(
                      lockPositions
                        ? "Posiciones fijadas"
                        : "Posiciones liberadas"
                    );
                  }}
                  title="Fijar o liberar la posición de las cajas"
                >
                  📌{" "}
                  {viewOptions.lockPositions
                    ? t("actions.pinned")
                    : t("actions.pin")}
                </button>
                <button
                  onClick={() => setViewOptionsOpen(true)}
                  title="Cuadrícula de página, alineación y posiciones"
                >{t("actions.options")}</button>
              </RibbonGroup>

              <RibbonGroup label={t("groups.output")}>
                <button onClick={handlePrint} title="Imprimir el documento">{t("actions.print")}</button>
              </RibbonGroup>
            </>
          )}

          {activeTab === "formulas" && (
            <>
              <RibbonGroup label={t("groups.assignment")}>
                <button
                  onClick={handleAssignmentInsert}
                  title="Insertar := en la región activa"
                >
                  :=
                </button>
                <button
                  className={sidebarOpen ? "active" : ""}
                  onClick={() => setSidebarOpen((current) => !current)}
                  title="Mostrar u ocultar el panel lateral de fórmulas"
                >
                  {sidebarOpen
                    ? t("actions.hideSidebar")
                    : t("actions.showSidebar")}
                </button>
              </RibbonGroup>
              <RibbonGroup label={t("groups.calculation")}>
                <button onClick={handleRecalculate} title="Recalcular (F9)">{t("actions.recalculate")}</button>
              </RibbonGroup>
              <RibbonGroup label={t("groups.math")}>
                <button
                  onClick={() => setPaletteKind("function")}
                  title="Insertar función o plantilla matemática"
                >{t("actions.functions")}</button>
                <button
                  onClick={() => setPaletteKind("unit")}
                  title="Insertar unidad"
                >{t("actions.unitPalette")}</button>
                <button
                  onClick={() => setUnitDialogOpen(true)}
                  title="Definir una unidad propia"
                >{t("actions.customUnit")}</button>
                <button
                  onClick={() => setConstantsOpen(true)}
                  title="Constantes de ingeniería y constantes propias"
                >{t("actions.constants")}</button>
              </RibbonGroup>
            </>
          )}

          {activeTab === "exportar" && (
            <>
              <RibbonGroup label={t("groups.document")}>
                <button onClick={() => handleExport("latex")}>
                  Exportar LaTeX
                </button>
                <button onClick={() => handleExport("odt")}>
                  Exportar ODT
                </button>
              </RibbonGroup>
              <RibbonGroup label={t("groups.pdf")}>
                <button
                  onClick={handlePdfExport}
                  disabled={exportingPdf}
                  title="PDF raster: texto no seleccionable"
                >
                  PDF raster
                </button>
                <button
                  onClick={handleLatexPdfExport}
                  disabled={exportingPdf}
                  title="PDF vectorial con XeLaTeX/MiKTeX"
                >
                  PDF LaTeX
                </button>
                <button
                  onClick={configureLatexEngine}
                  title="Configurar el motor LaTeX"
                >
                  ⚙
                </button>
              </RibbonGroup>
            </>
          )}

          {activeTab === "ayuda" && (
            <>
              <RibbonGroup label={t("groups.help")}>
                <button onClick={() => openHelp("inicio")}>
                  {t("actions.manual")}
                </button>
                <button onClick={() => openHelp("documento")}>
                  {t("actions.quickStart")}
                </button>
                <button onClick={() => openHelp("atajos")}>
                  {t("actions.shortcuts")}
                </button>
                <button onClick={() => openHelp("problemas")}>
                  {t("actions.troubleshooting")}
                </button>
                <button onClick={() => openHelp("acerca")}>
                  {t("actions.about")}
                </button>
              </RibbonGroup>
              <RibbonGroup label={t("groups.reference")}>
                <button onClick={() => openHelp("matematica")}>
                  {t("actions.mathReference")}
                </button>
                <button onClick={() => openHelp("unidades")}>
                  {t("actions.unitsReference")}
                </button>
                <button onClick={() => openHelp("exportar")}>
                  {t("actions.exportReference")}
                </button>
              </RibbonGroup>
            </>
          )}
        </div>
      </header>

      {draft && (
        <div className="draft-banner" role="status">
          <span>
            Hay un borrador autoguardado del{" "}
            {new Date(draft.savedAt).toLocaleString("es")}.
          </span>
          <button onClick={recoverDraft}>Recuperar</button>
          <button className="secondary" onClick={discardDraft}>
            Descartar
          </button>
        </div>
      )}

      <div className="workspace-row">
        <main className="workspace" ref={workspaceRef}>
          <Worksheet
            zoom={zoom}
            showGrid={viewOptions.showGrid}
            snapToGrid={viewOptions.snapToGrid}
            positionLocked={viewOptions.lockPositions}
            documentLanguage={doc.metadata.language}
            personalDictionary={doc.settings.personalDictionary ?? []}
            mathCompletions={mathCompletions}
            constantNames={constantNames}
          />
        </main>

        {sidebarOpen && (
          <FormulaSidebar
            customUnits={customUnits}
            onInsert={insertPaletteItem}
            onOpenPalette={setPaletteKind}
            onOpenUnitDialog={() => setUnitDialogOpen(true)}
            onAddPlot={handleAddPlot}
            onClose={() => setSidebarOpen(false)}
          />
        )}
      </div>

      {pageSetupOpen && (
        <PageSetupDialog
          setup={getPageSetup(doc.settings)}
          language={doc.metadata.language}
          onClose={() => setPageSetupOpen(false)}
          onApply={(setup, language) => {
            updatePageSetup(setup);
            if (language !== doc.metadata.language) {
              updateDocumentLanguage(language);
            }
            setPageSetupOpen(false);
            setStatus("Configuración de página aplicada");
          }}
        />
      )}

      {unitSettingsOpen && (
        <UnitSettingsDialog
          units={doc.settings.units}
          overrides={doc.settings.unitOverrides ?? {}}
          onClose={() => setUnitSettingsOpen(false)}
          onApply={(system, overrides) => {
            updateResultUnits(system, overrides);
            setUnitSettingsOpen(false);
            setStatus(`Unidades de resultados: ${resultUnitSystemLabel(system)}`);
          }}
        />
      )}

      {numberFormatOpen && (
        <NumberFormatDialog
          notation={numberFormat.notation}
          precision={numberFormat.precision}
          fractions={numberFormat.fractions}
          trimZeros={numberFormat.trimZeros}
          onClose={() => setNumberFormatOpen(false)}
          onApply={(format) => {
            updateNumberFormat(format);
            setNumberFormatOpen(false);
            setStatus(
              `Formato numérico: ${format.precision} decimales, ${format.notation}`
            );
          }}
        />
      )}

      {constantsOpen && (
        <ConstantsDialog
          constants={doc.settings.customConstants ?? []}
          onClose={() => setConstantsOpen(false)}
          onApply={(constants) => {
            updateCustomConstants(constants);
            setConstantsOpen(false);
            setStatus("Constantes actualizadas");
          }}
          onInsert={(name) => {
            insertPaletteItem({
              id: `constant-${name}`,
              label: name,
              latex: `\\text{${name}}`,
              sample: `\\text{${name}}`,
              description: "Constante de ingeniería",
              category: "Constantes"
            });
            setConstantsOpen(false);
            setStatus(`Constante insertada: ${name}`);
          }}
        />
      )}

      {viewOptionsOpen && (
        <ViewOptionsDialog
          options={viewOptions}
          onClose={() => setViewOptionsOpen(false)}
          onApply={(options) => {
            setViewOptions(options);
            setViewOptionsOpen(false);
            setStatus("Opciones de vista actualizadas");
          }}
        />
      )}

      {paletteKind && (
        <PaletteDialog
          kind={paletteKind}
          extraItems={customUnits}
          onInsert={handlePaletteInsert}
          onClose={() => setPaletteKind(null)}
        />
      )}

      {unitDialogOpen && (
        <UnitDialog
          onApply={handleCreateUnit}
          onClose={() => setUnitDialogOpen(false)}
        />
      )}

      {helpOpen && (
        <HelpDialog
          language={uiLanguage}
          initialSection={helpSection}
          onClose={() => setHelpOpen(false)}
        />
      )}
    </div>
  );
}

function RibbonGroup({
  label,
  children
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="ribbon-group">
      <div className="ribbon-items">{children}</div>
      <span className="ribbon-group-label">{label}</span>
    </div>
  );
}

function roundZoom(value: number): number {
  return Math.round(value * 10) / 10;
}

function regionCount(count: number, action: string): string {
  return `${count} ${count === 1 ? "región" : "regiones"} ${action}${
    count === 1 ? "" : "s"
  }`;
}

function measureImage(
  src: string
): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => {
      const width = image.naturalWidth || 320;
      const height = image.naturalHeight || 240;
      const scale = Math.min(1, 400 / width, 400 / height);
      resolve({
        width: Math.max(80, Math.round(width * scale)),
        height: Math.max(60, Math.round(height * scale))
      });
    };
    image.onerror = () => resolve({ width: 320, height: 240 });
    image.src = src;
  });
}
