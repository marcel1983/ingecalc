import { invoke } from "@tauri-apps/api/core";
import type { Document } from "../types/document";

const ENGINE_KEY = "ingecalc.latexEngine";

export function storedLatexEngine(): string | null {
  try {
    return globalThis.localStorage?.getItem(ENGINE_KEY) ?? null;
  } catch {
    return null;
  }
}

export function storeLatexEngine(path: string): void {
  try {
    globalThis.localStorage?.setItem(ENGINE_KEY, path);
  } catch {
    // A blocked storage should not prevent using the engine in this session.
  }
}

export async function detectLatexEngine(): Promise<string | null> {
  const stored = storedLatexEngine();
  if (stored) {
    return stored;
  }

  return invoke<string | null>("detect_latex_engine");
}

export async function exportLatexPdf(
  document: Document,
  path: string,
  engine: string
): Promise<void> {
  await invoke("export_pdf_latex", { document, path, engine });
}
