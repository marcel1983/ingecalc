export type PlotPoint = [number, number];

export type PlotSvgSeries = {
  expression: string;
  color: string;
  paths: PlotPoint[][];
  error?: string | null;
  unit?: string | null;
  xUnit?: string | null;
  yUnit?: string | null;
};

export type PlotView = {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
};

export function computePlotView(
  series: PlotSvgSeries[],
  xMin: number,
  xMax: number,
  yMin?: number | null,
  yMax?: number | null
): PlotView {
  const values = series
    .flatMap((item) => item.paths)
    .flatMap((path) => path)
    .map((point) => point[1])
    .filter(Number.isFinite);

  let lower = yMin ?? (values.length ? Math.min(...values) : -1);
  let upper = yMax ?? (values.length ? Math.max(...values) : 1);

  if (!Number.isFinite(lower) || !Number.isFinite(upper)) {
    lower = -1;
    upper = 1;
  }
  if (lower === upper) {
    lower -= 1;
    upper += 1;
  }
  if (yMin == null || yMax == null) {
    const padding = Math.max(0.5, (upper - lower) * 0.08);
    lower -= padding;
    upper += padding;
  }

  return {
    xMin,
    xMax,
    yMin: lower,
    yMax: upper
  };
}

export function plotSvgMarkup(
  series: PlotSvgSeries[],
  view: PlotView,
  width: number,
  height: number,
  idPrefix: string
): string {
  const safeWidth = Math.max(120, width);
  const safeHeight = Math.max(90, height);
  const padding = { left: 38, right: 12, top: 16, bottom: 24 };
  const plotWidth = Math.max(1, safeWidth - padding.left - padding.right);
  const plotHeight = Math.max(1, safeHeight - padding.top - padding.bottom);
  const xSpan = Math.max(1e-12, view.xMax - view.xMin);
  const ySpan = Math.max(1e-12, view.yMax - view.yMin);
  const clipId = `plot-${idPrefix.replace(/[^a-zA-Z0-9_-]/g, "")}-clip`;

  const x = (value: number) =>
    padding.left + ((value - view.xMin) / xSpan) * plotWidth;
  const y = (value: number) =>
    padding.top + ((view.yMax - value) / ySpan) * plotHeight;

  const parts: string[] = [];
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 ${safeWidth} ${safeHeight}" role="img" aria-label="Gráfico 2D">`
  );
  parts.push(`<rect width="${safeWidth}" height="${safeHeight}" fill="#ffffff"/>`);
  parts.push(
    `<defs><clipPath id="${clipId}"><rect x="${padding.left}" y="${padding.top}" width="${plotWidth}" height="${plotHeight}"/></clipPath></defs>`
  );

  for (let index = 0; index <= 4; index += 1) {
    const fraction = index / 4;
    const gridX = padding.left + fraction * plotWidth;
    const gridY = padding.top + fraction * plotHeight;
    parts.push(
      `<line x1="${gridX.toFixed(1)}" y1="${padding.top}" x2="${gridX.toFixed(1)}" y2="${(padding.top + plotHeight).toFixed(1)}" stroke="#e2e8f0" stroke-width="1"/>`
    );
    parts.push(
      `<line x1="${padding.left}" y1="${gridY.toFixed(1)}" x2="${(padding.left + plotWidth).toFixed(1)}" y2="${gridY.toFixed(1)}" stroke="#e2e8f0" stroke-width="1"/>`
    );

    const xValue = snapZero(view.xMin + fraction * xSpan, xSpan);
    const yValue = snapZero(view.yMax - fraction * ySpan, ySpan);
    parts.push(
      `<text x="${gridX.toFixed(1)}" y="${(padding.top + plotHeight + 16).toFixed(1)}" font-size="10" fill="#64748b" text-anchor="middle">${escapeXml(formatTick(xValue))}</text>`
    );
    parts.push(
      `<text x="${(padding.left - 6).toFixed(1)}" y="${(gridY + 3).toFixed(1)}" font-size="10" fill="#64748b" text-anchor="end">${escapeXml(formatTick(yValue))}</text>`
    );
  }

  if (view.yMin <= 0 && view.yMax >= 0) {
    const zeroY = y(0);
    parts.push(
      `<line x1="${padding.left}" y1="${zeroY.toFixed(1)}" x2="${(padding.left + plotWidth).toFixed(1)}" y2="${zeroY.toFixed(1)}" stroke="#94a3b8" stroke-width="1.2"/>`
    );
  }
  if (view.xMin <= 0 && view.xMax >= 0) {
    const zeroX = x(0);
    parts.push(
      `<line x1="${zeroX.toFixed(1)}" y1="${padding.top}" x2="${zeroX.toFixed(1)}" y2="${(padding.top + plotHeight).toFixed(1)}" stroke="#94a3b8" stroke-width="1.2"/>`
    );
  }

  const xUnit = series.find((item) => item.xUnit)?.xUnit;
  const yUnit = series.find((item) => item.yUnit ?? item.unit)?.yUnit ??
    series.find((item) => item.unit)?.unit;
  if (xUnit) {
    parts.push(
      `<text x="${(padding.left + plotWidth).toFixed(1)}" y="${(safeHeight - 6).toFixed(1)}" font-size="10" fill="#475569" text-anchor="end">x (${escapeXml(xUnit)})</text>`
    );
  }
  if (yUnit) {
    parts.push(
      `<text x="${padding.left.toFixed(1)}" y="${(padding.top - 5).toFixed(1)}" font-size="10" fill="#475569">y (${escapeXml(yUnit)})</text>`
    );
  }

  parts.push(`<g clip-path="url(#${clipId})">`);
  let hasPoints = false;
  for (const item of series) {
    for (const pathPoints of item.paths) {
      if (pathPoints.length === 0) {
        continue;
      }
      hasPoints = true;
      const path = pathPoints
        .map((point, index) => {
          const command = index === 0 ? "M" : "L";
          return `${command}${x(point[0]).toFixed(2)},${y(point[1]).toFixed(2)}`;
        })
        .join(" ");
      parts.push(
        `<path d="${path}" fill="none" stroke="${safeColor(item.color)}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>`
      );
    }
  }
  parts.push("</g>");

  if (!hasPoints) {
    parts.push(
      `<text x="${(padding.left + plotWidth / 2).toFixed(1)}" y="${(padding.top + plotHeight / 2).toFixed(1)}" font-size="12" fill="#64748b" text-anchor="middle">Sin datos</text>`
    );
  }

  const legend = series.filter((item) => item.paths.length > 0).slice(0, 3);
  legend.forEach((item, index) => {
    const legendY = padding.top + 12 + index * 14;
    parts.push(
      `<line x1="${(padding.left + 4).toFixed(1)}" y1="${legendY.toFixed(1)}" x2="${(padding.left + 20).toFixed(1)}" y2="${legendY.toFixed(1)}" stroke="${safeColor(item.color)}" stroke-width="2"/>`
    );
    parts.push(
      `<text x="${(padding.left + 24).toFixed(1)}" y="${(legendY + 3).toFixed(1)}" font-size="9" fill="#475569">${escapeXml(trimLabel(item.expression))}</text>`
    );
  });

  parts.push("</svg>");
  return parts.join("");
}

function snapZero(value: number, span: number): number {
  return Math.abs(value) <= span * 2e-3 ? 0 : value;
}

function formatTick(value: number): string {
  if (!Number.isFinite(value)) {
    return "";
  }
  if (Math.abs(value) < 1e-9) {
    return "0";
  }
  if (Math.abs(value) >= 1000 || Math.abs(value) < 0.01) {
    return value.toExponential(1);
  }
  return value.toFixed(2).replace(/\.?0+$/, "");
}

function safeColor(color: string): string {
  return /^#[0-9a-fA-F]{6}$/.test(color) ? color : "#2563eb";
}

function trimLabel(label: string): string {
  return label.length > 22 ? `${label.slice(0, 21)}…` : label;
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
