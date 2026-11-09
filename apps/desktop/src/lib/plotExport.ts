import { save } from "@tauri-apps/plugin-dialog";
import { saveBinary } from "./api";
import { isTauri } from "./environment";

export async function savePlotSvg(
  svgMarkup: string,
  baseName: string
): Promise<boolean> {
  const bytes = new TextEncoder().encode(svgMarkup);

  if (isTauri()) {
    const path = await save({
      defaultPath: `${sanitizeName(baseName)}.svg`,
      filters: [{ name: "SVG", extensions: ["svg"] }]
    });
    if (!path) {
      return false;
    }
    await saveBinary(
      path.toLowerCase().endsWith(".svg") ? path : `${path}.svg`,
      bytesToBase64(bytes)
    );
    return true;
  }

  downloadBlob(
    new Blob([bytes], { type: "image/svg+xml" }),
    `${sanitizeName(baseName)}.svg`
  );
  return true;
}

export async function savePlotPng(
  svgMarkup: string,
  width: number,
  height: number,
  baseName: string
): Promise<boolean> {
  const png = await svgToPng(svgMarkup, width, height);
  const base64 = png.split(",")[1] ?? "";

  if (isTauri()) {
    const path = await save({
      defaultPath: `${sanitizeName(baseName)}.png`,
      filters: [{ name: "PNG", extensions: ["png"] }]
    });
    if (!path) {
      return false;
    }
    await saveBinary(
      path.toLowerCase().endsWith(".png") ? path : `${path}.png`,
      base64
    );
    return true;
  }

  const link = document.createElement("a");
  link.href = png;
  link.download = `${sanitizeName(baseName)}.png`;
  link.click();
  return true;
}

function svgToPng(
  svgMarkup: string,
  width: number,
  height: number
): Promise<string> {
  return new Promise((resolve, reject) => {
    const blob = new Blob([svgMarkup], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    const image = new Image();

    image.onload = () => {
      const scale = 2;
      const canvas = document.createElement("canvas");
      canvas.width = Math.max(1, Math.round(width * scale));
      canvas.height = Math.max(1, Math.round(height * scale));
      const context = canvas.getContext("2d");
      if (!context) {
        URL.revokeObjectURL(url);
        reject(new Error("No se pudo crear el canvas"));
        return;
      }
      context.fillStyle = "#ffffff";
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.scale(scale, scale);
      context.drawImage(image, 0, 0, width, height);
      URL.revokeObjectURL(url);
      resolve(canvas.toDataURL("image/png"));
    };

    image.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error("No se pudo convertir el SVG a PNG"));
    };
    image.src = url;
  });
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let index = 0; index < bytes.length; index += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize));
  }
  return btoa(binary);
}

function downloadBlob(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  URL.revokeObjectURL(url);
}

function sanitizeName(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, "_").trim() || "grafico";
}
