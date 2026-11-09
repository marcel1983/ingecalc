export type SurfaceGrid = {
  expression: string;
  color: string;
  grid: (number | null)[][];
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  error?: string | null;
  zUnit?: string | null;
};

export type SurfaceView = {
  yaw: number;
  pitch: number;
  zoom: number;
  wireframe: boolean;
};

export const DEFAULT_SURFACE_VIEW: SurfaceView = {
  yaw: -0.65,
  pitch: 0.5,
  zoom: 1,
  wireframe: false
};

type Point3 = [number, number, number];
type Point2 = [number, number];

type Quad = {
  points: Point2[];
  depth: number;
  color: string;
  stroke: string;
  wireframe: boolean;
};

export function surfaceSvgMarkup(
  surfaces: SurfaceGrid[],
  width: number,
  height: number,
  view: SurfaceView,
  idPrefix: string
): string {
  const safeWidth = Math.max(160, width);
  const safeHeight = Math.max(120, height);
  const values = surfaces
    .flatMap((surface) => surface.grid.flat())
    .filter((value): value is number => value !== null && Number.isFinite(value));
  let zMin = values.length ? Math.min(...values) : -1;
  let zMax = values.length ? Math.max(...values) : 1;
  if (zMin === zMax) {
    zMin -= 1;
    zMax += 1;
  }

  const centerX = safeWidth / 2;
  const centerY = safeHeight * 0.54;
  const scale = Math.min(safeWidth, safeHeight) * 0.34 * view.zoom;
  const cosYaw = Math.cos(view.yaw);
  const sinYaw = Math.sin(view.yaw);
  const cosPitch = Math.cos(view.pitch);
  const sinPitch = Math.sin(view.pitch);
  const depth = 4.6;

  const project = (point: Point3): Point2 => {
    const x1 = point[0] * cosYaw - point[1] * sinYaw;
    const y1 = point[0] * sinYaw + point[1] * cosYaw;
    const z1 = point[2];
    const y2 = y1 * cosPitch + z1 * sinPitch;
    const z2 = -y1 * sinPitch + z1 * cosPitch;
    const perspective = depth / (depth + z2 * 1.35);
    return [
      centerX + x1 * scale * perspective,
      centerY - y2 * scale * perspective
    ];
  };

  const normalized = (
    x: number,
    y: number,
    z: number,
    surface: SurfaceGrid
  ): Point3 => [
    (2 * (x - surface.xMin)) / Math.max(1e-12, surface.xMax - surface.xMin) - 1,
    (2 * (y - surface.yMin)) / Math.max(1e-12, surface.yMax - surface.yMin) - 1,
    (2 * (z - zMin)) / Math.max(1e-12, zMax - zMin) - 1
  ];

  const quads: Quad[] = [];
  for (const surface of surfaces) {
    const rows = surface.grid.length;
    const columns = surface.grid[0]?.length ?? 0;
    for (let row = 0; row + 1 < rows; row += 1) {
      for (let column = 0; column + 1 < columns; column += 1) {
        const z00 = surface.grid[row][column];
        const z01 = surface.grid[row][column + 1];
        const z11 = surface.grid[row + 1][column + 1];
        const z10 = surface.grid[row + 1][column];
        if (
          z00 === null ||
          z01 === null ||
          z11 === null ||
          z10 === null ||
          !Number.isFinite(z00) ||
          !Number.isFinite(z01) ||
          !Number.isFinite(z11) ||
          !Number.isFinite(z10)
        ) {
          continue;
        }
        const x0 = surface.xMin + ((surface.xMax - surface.xMin) * column) / (columns - 1);
        const x1 = surface.xMin + ((surface.xMax - surface.xMin) * (column + 1)) / (columns - 1);
        const y0 = surface.yMin + ((surface.yMax - surface.yMin) * row) / (rows - 1);
        const y1 = surface.yMin + ((surface.yMax - surface.yMin) * (row + 1)) / (rows - 1);
        const corners = [
          normalized(x0, y0, z00, surface),
          normalized(x1, y0, z01, surface),
          normalized(x1, y1, z11, surface),
          normalized(x0, y1, z10, surface)
        ];
        const projected = corners.map(project);
        const depthValue =
          corners.reduce((sum, corner) => sum - corner[1] * sinPitch + corner[2] * cosPitch, 0) /
          corners.length;
        quads.push({
          points: projected,
          depth: depthValue,
          color: surface.color,
          stroke: darken(surface.color, 0.45),
          wireframe: view.wireframe
        });
      }
    }
  }

  quads.sort((left, right) => right.depth - left.depth);

  const parts: string[] = [];
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 ${safeWidth} ${safeHeight}" role="img" aria-label="Gráfico 3D">`
  );
  parts.push(`<rect width="${safeWidth}" height="${safeHeight}" fill="#ffffff"/>`);

  for (const quad of quads) {
    const points = quad.points
      .map((point) => `${point[0].toFixed(1)},${point[1].toFixed(1)}`)
      .join(" ");
    if (quad.wireframe) {
      parts.push(
        `<polygon points="${points}" fill="none" stroke="${quad.stroke}" stroke-width="0.55" opacity="0.8"/>`
      );
    } else {
      parts.push(
        `<polygon points="${points}" fill="${quad.color}" fill-opacity="0.82" stroke="${quad.stroke}" stroke-width="0.35" stroke-opacity="0.6"/>`
      );
    }
  }

  const origin = project([0, 0, 0]);
  const axes: Array<{ end: Point2; label: string; color: string }> = [
    { end: project([1.35, 0, 0]), label: "X", color: "#dc2626" },
    { end: project([0, 1.35, 0]), label: "Y", color: "#059669" },
    { end: project([0, 0, 1.2]), label: "Z", color: "#2563eb" }
  ];
  for (const axis of axes) {
    parts.push(
      `<line x1="${origin[0].toFixed(1)}" y1="${origin[1].toFixed(1)}" x2="${axis.end[0].toFixed(1)}" y2="${axis.end[1].toFixed(1)}" stroke="${axis.color}" stroke-width="1.5" opacity="0.8"/>`
    );
    parts.push(
      `<text x="${(axis.end[0] + 4).toFixed(1)}" y="${(axis.end[1] - 4).toFixed(1)}" font-size="11" font-weight="600" fill="${axis.color}">${axis.label}</text>`
    );
  }
  const zUnit = surfaces.find((surface) => surface.zUnit)?.zUnit;
  if (zUnit) {
    const zEnd = project([0, 0, 1.2]);
    parts.push(
      `<text x="${(zEnd[0] + 4).toFixed(1)}" y="${(zEnd[1] + 12).toFixed(1)}" font-size="10" fill="#475569">z (${escapeXml(zUnit)})</text>`
    );
  }

  parts.push(
    `<text x="10" y="${safeHeight - 10}" font-size="10" fill="#64748b">z: ${formatTick(zMin)} … ${formatTick(zMax)}</text>`
  );

  let legend = 14;
  for (const surface of surfaces) {
    if (!surface.expression.trim()) {
      continue;
    }
    parts.push(
      `<rect x="10" y="${legend - 8}" width="9" height="9" rx="2" fill="${surface.color}"/>`
    );
    parts.push(
      `<text x="24" y="${legend}" font-size="10" fill="#334155">${escapeXml(surface.expression)}</text>`
    );
    legend += 15;
  }

  parts.push(`</svg>`);
  return parts.join("");
}

function darken(hex: string, amount: number): string {
  const match = /^#([0-9a-f]{6})$/i.exec(hex);
  if (!match) {
    return hex;
  }
  const value = Number.parseInt(match[1], 16);
  const red = Math.round(((value >> 16) & 255) * (1 - amount));
  const green = Math.round(((value >> 8) & 255) * (1 - amount));
  const blue = Math.round((value & 255) * (1 - amount));
  return `#${[red, green, blue]
    .map((channel) => channel.toString(16).padStart(2, "0"))
    .join("")}`;
}

function formatTick(value: number): string {
  if (Math.abs(value) < 1e-10) {
    return "0";
  }
  return Number(value.toPrecision(4)).toString();
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
