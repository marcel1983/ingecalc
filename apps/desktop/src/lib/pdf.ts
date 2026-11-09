import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import DOMPurify from "dompurify";
import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";
import * as katex from "katex";
import { samplePlot } from "./api";
import { formatPageText, getPageSetup } from "./pageSetup";
import { computePlotView, plotSvgMarkup, type PlotSvgSeries } from "./plot";
import type { Document, Page, PageSetup } from "../types/document";

/**
 * Export the document as a raster PDF.
 *
 * The MVP uses the rendered DOM (KaTeX for math, TipTap HTML for text) and
 * html2canvas + jsPDF. It is deliberately platform-independent and works
 * without a native PDF layout engine.
 */
export async function exportPdf(document: Document): Promise<boolean> {
  const pdf = await buildPdf(document);
  const dataUri = pdf.output("datauristring");
  const base64 = dataUri.split(",")[1] ?? "";
  const baseName = sanitizeFileName(document.metadata.title || "document");

  if (isTauri()) {
    const path = await save({
      defaultPath: `${baseName}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }]
    });

    if (!path) {
      return false;
    }

    const finalPath = path.toLowerCase().endsWith(".pdf")
      ? path
      : `${path}.pdf`;

    await invoke("save_pdf", { path: finalPath, data: base64 });
    return true;
  }

  // Browser fallback for development and UI testing.
  pdf.save(`${baseName}.pdf`);
  return true;
}

async function buildPdf(document: Document): Promise<jsPDF> {
  await globalThis.document.fonts.ready;

  const container = globalThis.document.createElement("div");
  container.className = "pdf-export-root";
  globalThis.document.body.appendChild(container);

  let pdf: jsPDF | null = null;

  try {
    const setup = getPageSetup(document.settings);
    let pageNumber = 0;

    for (const page of document.pages) {
      pageNumber += 1;
      const pageElement = await renderPage(page, pageNumber, setup, document);
      container.appendChild(pageElement);
      await waitForImages(pageElement);

      const canvas = await html2canvas(pageElement, {
        scale: 2,
        backgroundColor: "#ffffff",
        logging: false,
        useCORS: true
      });

      const image = canvas.toDataURL("image/jpeg", 0.92);
      const orientation = page.width >= page.height ? "landscape" : "portrait";

      if (!pdf) {
        pdf = new jsPDF({
          orientation,
          unit: "px",
          format: [page.width, page.height],
          hotfixes: ["px_scaling"]
        });
      } else {
        pdf.addPage([page.width, page.height], orientation);
      }

      pdf.addImage(
        image,
        "JPEG",
        0,
        0,
        page.width,
        page.height,
        undefined,
        "FAST"
      );

      container.removeChild(pageElement);
    }
  } finally {
    container.remove();
  }

  if (!pdf) {
    throw new Error("El documento no tiene páginas");
  }

  return pdf;
}

function renderTextHtml(html: string): string {
  const sanitized = DOMPurify.sanitize(html);
  const template = document.createElement("template");
  template.innerHTML = sanitized;

  template.content.querySelectorAll("[data-inline-math]").forEach((element) => {
    const latex = element.getAttribute("data-latex") ?? "";
    const rendered = document.createElement("span");
    rendered.className = "inline-math";

    try {
      rendered.innerHTML = katex.renderToString(latex, {
        throwOnError: false,
        displayMode: false,
        output: "html"
      });
    } catch {
      rendered.textContent = latex;
    }

    element.replaceWith(rendered);
  });

  return template.innerHTML;
}

function renderPage(
  page: Page,
  pageNumber: number,
  setup: PageSetup,
  document: Document
): Promise<HTMLElement> {
  return renderPageAsync(page, pageNumber, setup, document);
}

async function renderPageAsync(
  page: Page,
  pageNumber: number,
  setup: PageSetup,
  doc: Document
): Promise<HTMLElement> {
  const pageElement = document.createElement("div");
  pageElement.className = "pdf-page";
  pageElement.style.width = `${page.width}px`;
  pageElement.style.height = `${page.height}px`;

  if (setup.header) {
    const header = document.createElement("div");
    header.className = "pdf-header";
    header.style.left = `${setup.marginLeft}px`;
    header.style.right = `${setup.marginRight}px`;
    header.style.top = `${Math.max(4, setup.marginTop / 2)}px`;
    header.textContent = formatPageText(setup.header, pageNumber);
    pageElement.appendChild(header);
  }

  if (setup.footer) {
    const footer = document.createElement("div");
    footer.className = "pdf-footer";
    footer.style.left = `${setup.marginLeft}px`;
    footer.style.right = `${setup.marginRight}px`;
    footer.style.bottom = `${Math.max(4, setup.marginBottom / 2)}px`;
    footer.textContent = formatPageText(setup.footer, pageNumber);
    pageElement.appendChild(footer);
  }

  for (const region of page.regions) {
    const regionElement = document.createElement("div");
    regionElement.className = "pdf-region";
    regionElement.style.left = `${region.x}px`;
    regionElement.style.top = `${region.y}px`;
    regionElement.style.width = `${region.w}px`;
    regionElement.style.height = `${region.h}px`;

    if (region.type === "math") {
      const mathElement = document.createElement("div");
      mathElement.className = "pdf-math";
      try {
        mathElement.innerHTML = katex.renderToString(
          region.source || region.ascii,
          {
            throwOnError: false,
            displayMode: true,
            output: "html"
          }
        );
      } catch {
        mathElement.textContent = region.source || region.ascii;
      }
      regionElement.appendChild(mathElement);
    } else if (region.type === "text") {
      const textElement = document.createElement("div");
      textElement.className = "pdf-text";
      textElement.innerHTML = renderTextHtml(region.html);
      regionElement.appendChild(textElement);
    } else if (region.type === "image") {
      const image = document.createElement("img");
      image.className = "pdf-image";
      image.src = region.src;
      image.alt = "";
      regionElement.appendChild(image);
    } else {
      const visible = region.series.filter(
        (item) => item.visible && item.expression.trim()
      );
      try {
        const sampled = await samplePlot(
          doc,
          visible.map((item) => ({
            kind: item.kind ?? "explicit",
            expression: item.expression,
            expression2: item.expression2 ?? "",
            parameterMin: item.parameterMin ?? -10,
            parameterMax: item.parameterMax ?? 10
          })),
          region.xMin,
          region.xMax,
          region.yMin,
          region.yMax,
          320
        );
        const plotSeries: PlotSvgSeries[] = visible.map((item, index) => ({
          expression: item.expression,
          color: item.color,
          paths: sampled[index]?.paths ?? [],
          error: sampled[index]?.error ?? null
        }));
        const view = computePlotView(
          plotSeries,
          region.xMin,
          region.xMax,
          region.yMin,
          region.yMax
        );
        regionElement.innerHTML = plotSvgMarkup(
          plotSeries,
          view,
          region.w,
          region.h,
          region.id
        );
      } catch {
        regionElement.textContent = "Gráfico 2D";
      }
    }

    pageElement.appendChild(regionElement);
  }

  return pageElement;
}

async function waitForImages(element: HTMLElement): Promise<void> {
  const images = Array.from(element.querySelectorAll("img"));
  await Promise.all(
    images.map((image) => {
      if (image.complete) {
        return Promise.resolve();
      }
      return new Promise<void>((resolve) => {
        image.addEventListener("load", () => resolve(), { once: true });
        image.addEventListener("error", () => resolve(), { once: true });
      });
    })
  );
}

function sanitizeFileName(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, "_").trim() || "document";
}

function isTauri(): boolean {
  return (
    typeof globalThis.window !== "undefined" &&
    "__TAURI_INTERNALS__" in globalThis.window
  );
}
