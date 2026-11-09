import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  Document,
  DocumentEvalResult,
  EvalResponse
} from "../types/document";

export async function evaluateMath(text: string): Promise<EvalResponse> {
  return invoke<EvalResponse>("evaluate_math", { text });
}

export async function evaluateDocument(
  document: Document
): Promise<DocumentEvalResult[]> {
  return invoke<DocumentEvalResult[]>("evaluate_document", { document });
}

export type PlotSeriesRequest = {
  kind: "explicit" | "parametric" | "implicit" | "surface";
  expression: string;
  expression2: string;
  parameterMin: number;
  parameterMax: number;
};

export type PlotSampleResult = {
  paths: [number, number][][];
  error: string | null;
  unit?: string | null;
  xUnit?: string | null;
  yUnit?: string | null;
};

export async function samplePlot(
  document: Document,
  series: PlotSeriesRequest[],
  xMin: number,
  xMax: number,
  yMin: number | null,
  yMax: number | null,
  samples = 320
): Promise<PlotSampleResult[]> {
  return invoke<PlotSampleResult[]>("sample_plot", {
    request: {
      document,
      series,
      xMin,
      xMax,
      yMin,
      yMax,
      samples
    }
  });
}

export type SurfaceSampleResult = {
  grid: (number | null)[][];
  error: string | null;
  unit?: string | null;
};

export async function sampleSurface(
  document: Document,
  expression: string,
  xMin: number,
  xMax: number,
  yMin: number,
  yMax: number,
  samples = 32
): Promise<SurfaceSampleResult> {
  return invoke<SurfaceSampleResult>("sample_surface", {
    request: {
      document,
      expression,
      xMin,
      xMax,
      yMin,
      yMax,
      samples
    }
  });
}

export async function saveBinary(
  path: string,
  base64: string
): Promise<void> {
  await invoke("save_binary", { path, data: base64 });
}

export async function loadDocument(): Promise<Document | null> {
  const selection = await open({
    multiple: false,
    filters: [
      { name: "IngeCalc", extensions: ["smj", "json"] }
    ]
  });

  if (!selection || Array.isArray(selection)) {
    return null;
  }

  return invoke<Document>("load_document", { path: selection });
}

export async function saveDocument(document: Document): Promise<void> {
  const path = await save({
    filters: [
      { name: "IngeCalc", extensions: ["smj"] }
    ]
  });

  if (!path) {
    return;
  }

  await invoke("save_document", { path, document });
}

export type ExportFormat = "latex" | "odt";

export async function exportDocument(
  document: Document,
  format: ExportFormat
): Promise<boolean> {
  const extension = format === "latex" ? "tex" : "odt";
  const baseName = (document.metadata.title || "document").replace(
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
    return false;
  }

  const finalPath = path.toLowerCase().endsWith(`.${extension}`)
    ? path
    : `${path}.${extension}`;

  await invoke("export_document", { document, format, path: finalPath });
  return true;
}
