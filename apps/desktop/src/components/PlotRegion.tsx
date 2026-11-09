import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type WheelEvent as ReactWheelEvent
} from "react";
import { samplePlot, sampleSurface } from "../lib/api";
import { isTauri } from "../lib/environment";
import {
  computePlotView,
  plotSvgMarkup,
  type PlotSvgSeries
} from "../lib/plot";
import {
  DEFAULT_SURFACE_VIEW,
  surfaceSvgMarkup,
  type SurfaceGrid,
  type SurfaceView
} from "../lib/plot3d";
import { savePlotPng, savePlotSvg } from "../lib/plotExport";
import { useDocumentStore } from "../stores/document";
import type { PlotRegion as PlotRegionType } from "../types/document";

type Props = {
  region: PlotRegionType;
  onChange: (patch: Partial<PlotRegionType>) => void;
};

type ViewState = {
  xMin: number;
  xMax: number;
  yMin: number | null;
  yMax: number | null;
};

type DragState = {
  pointerId: number;
  startX: number;
  startY: number;
  view: ViewState;
  width: number;
  height: number;
  surface?: SurfaceView;
};

export function PlotRegion({ region, onChange }: Props) {
  const doc = useDocumentStore((state) => state.doc);
  const docRef = useRef(doc);
  docRef.current = doc;

  const [view, setView] = useState<ViewState>(() => ({
    xMin: region.xMin,
    xMax: region.xMax,
    yMin: region.yMin,
    yMax: region.yMax
  }));
  const [series, setSeries] = useState<PlotSvgSeries[]>([]);
  const [surfaceSeries, setSurfaceSeries] = useState<SurfaceGrid[]>([]);
  const [surfaceError, setSurfaceError] = useState("");
  const [surfaceView, setSurfaceView] = useState<SurfaceView>(
    DEFAULT_SURFACE_VIEW
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [size, setSize] = useState({ width: region.w, height: region.h });
  const canvasRef = useRef<HTMLDivElement>(null);
  const drag = useRef<DragState | null>(null);

  const mathSignature = useMemo(
    () =>
      JSON.stringify(
        doc.pages.map((page) =>
          page.regions
            .filter((item) => item.type === "math")
            .map((item) => [item.id, item.ascii])
        )
      ),
    [doc]
  );
  const seriesSignature = JSON.stringify(region.series);
  const hasSurface = region.series.some(
    (item) => item.kind === "surface" && item.visible && item.expression.trim()
  );

  useEffect(() => {
    setView({
      xMin: region.xMin,
      xMax: region.xMax,
      yMin: region.yMin,
      yMax: region.yMax
    });
  }, [
    region.id,
    region.xMin,
    region.xMax,
    region.yMin,
    region.yMax
  ]);

  useEffect(() => {
    const element = canvasRef.current;
    if (!element) {
      return;
    }

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }
      setSize({
        width: entry.contentRect.width,
        height: entry.contentRect.height
      });
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setSeries([]);
      setError("La gráfica requiere la app de escritorio");
      return;
    }

    const expressions = region.series
      .filter(
        (item) =>
          item.visible &&
          item.expression.trim() &&
          item.kind !== "surface"
      )
      .map((item) => ({
        kind: item.kind ?? "explicit",
        expression: item.expression,
        expression2: item.expression2 ?? "",
        parameterMin: item.parameterMin ?? -10,
        parameterMax: item.parameterMax ?? 10
      }));
    if (expressions.length === 0) {
      setSeries([]);
      setError("");
      return;
    }

    let cancelled = false;
    setLoading(true);
    const timer = window.setTimeout(async () => {
      try {
        const sampled = await samplePlot(
          docRef.current,
          expressions,
          view.xMin,
          view.xMax,
          view.yMin,
          view.yMax,
          320
        );
        if (cancelled) {
          return;
        }

        const visible = region.series.filter(
          (item) =>
            item.visible &&
            item.expression.trim() &&
            item.kind !== "surface"
        );
        const next = visible.map((item, index) => ({
          expression: item.expression,
          color: item.color,
          paths: sampled[index]?.paths ?? [],
          error: sampled[index]?.error ?? null,
          unit: sampled[index]?.unit ?? null,
          xUnit: sampled[index]?.xUnit ?? null,
          yUnit: sampled[index]?.yUnit ?? null
        }));
        setSeries(next);
        setError(
          sampled.find((item) => item.error && item.paths.length === 0)?.error ??
            ""
        );
      } catch (caught) {
        if (!cancelled) {
          setSeries([]);
          setError(String(caught));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }, 220);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [mathSignature, seriesSignature, view.xMin, view.xMax]);

  useEffect(() => {
    const surfaces = region.series.filter(
      (item) => item.kind === "surface" && item.visible && item.expression.trim()
    );
    if (surfaces.length === 0) {
      setSurfaceSeries([]);
      setSurfaceError("");
      return;
    }
    if (!isTauri()) {
      setSurfaceSeries([]);
      setSurfaceError("La gráfica 3D requiere la app de escritorio");
      return;
    }

    let cancelled = false;
    setLoading(true);
    const timer = window.setTimeout(async () => {
      try {
        const yMin = region.yMin ?? -5;
        const yMax = region.yMax ?? 5;
        const sampled = await Promise.all(
          surfaces.map((item) =>
            sampleSurface(
              docRef.current,
              item.expression,
              region.xMin,
              region.xMax,
              yMin,
              yMax,
              32
            )
          )
        );
        if (cancelled) {
          return;
        }
        setSurfaceSeries(
          surfaces.map((item, index) => ({
            expression: item.expression,
            color: item.color,
            grid: sampled[index]?.grid ?? [],
            xMin: region.xMin,
            xMax: region.xMax,
            yMin,
            yMax,
            error: sampled[index]?.error ?? null,
            zUnit: sampled[index]?.unit ?? null
          }))
        );
        setSurfaceError(
          sampled.find((item) => item.error && item.grid.length === 0)?.error ??
            ""
        );
      } catch (caught) {
        if (!cancelled) {
          setSurfaceSeries([]);
          setSurfaceError(String(caught));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }, 220);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [
    mathSignature,
    seriesSignature,
    region.xMin,
    region.xMax,
    region.yMin,
    region.yMax
  ]);

  const computedView = computePlotView(
    series,
    view.xMin,
    view.xMax,
    view.yMin,
    view.yMax
  );
  const svg = useMemo(
    () =>
      hasSurface
        ? surfaceSvgMarkup(
            surfaceSeries,
            size.width,
            size.height,
            surfaceView,
            region.id
          )
        : plotSvgMarkup(
            series,
            computedView,
            size.width,
            size.height,
            region.id
          ),
    [
      series,
      surfaceSeries,
      computedView,
      size.width,
      size.height,
      region.id,
      hasSurface,
      surfaceView
    ]
  );
  const displayError = hasSurface ? surfaceError : error;

  const updateSeries = (
    index: number,
    patch: Partial<PlotRegionType["series"][number]>
  ) => {
    onChange({
      series: region.series.map((item, itemIndex) =>
        itemIndex === index ? { ...item, ...patch } : item
      )
    });
  };

  const removeSeries = (index: number) => {
    const next = region.series.filter((_, itemIndex) => itemIndex !== index);
    onChange({ series: next });
  };

  const addSeries = () => {
    onChange({
      series: [
        ...region.series,
        {
          expression: "cos(x)",
          expression2: "",
          kind: "explicit",
          parameterMin: -10,
          parameterMax: 10,
          color: paletteColor(region.series.length),
          visible: true
        }
      ]
    });
  };

  const resetView = () => {
    if (hasSurface) {
      setSurfaceView(DEFAULT_SURFACE_VIEW);
      return;
    }
    setView({
      xMin: region.xMin,
      xMax: region.xMax,
      yMin: region.yMin,
      yMax: region.yMax
    });
  };

  const handleWheel = (event: ReactWheelEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    if (hasSurface) {
      const factor = event.deltaY > 0 ? 0.9 : 1.1;
      setSurfaceView((current) => ({
        ...current,
        zoom: Math.max(0.35, Math.min(3.2, current.zoom * factor))
      }));
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    if (rect.width <= 0) {
      return;
    }

    const fraction = Math.max(
      0,
      Math.min(1, (event.clientX - rect.left) / rect.width)
    );
    const factor = event.deltaY > 0 ? 1.2 : 1 / 1.2;
    const xCenter = view.xMin + fraction * (view.xMax - view.xMin);
    const next: ViewState = {
      ...view,
      xMin: xCenter - (xCenter - view.xMin) * factor,
      xMax: xCenter + (view.xMax - xCenter) * factor
    };

    const yMin = view.yMin ?? computedView.yMin;
    const yMax = view.yMax ?? computedView.yMax;
    const yCenter = (yMin + yMax) / 2;
    next.yMin = yCenter - (yCenter - yMin) * factor;
    next.yMax = yCenter + (yMax - yCenter) * factor;
    setView(next);
  };

  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    drag.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      view,
      width: rect.width,
      height: rect.height,
      surface: hasSurface ? surfaceView : undefined
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (!current || current.pointerId !== event.pointerId) {
      return;
    }

    if (current.surface) {
      const dx = (event.clientX - current.startX) / current.width;
      const dy = (event.clientY - current.startY) / current.height;
      setSurfaceView({
        ...current.surface,
        yaw: current.surface.yaw + dx * Math.PI * 1.6,
        pitch: Math.max(
          -1.35,
          Math.min(1.35, current.surface.pitch + dy * Math.PI)
        )
      });
      return;
    }

    const xSpan = current.view.xMax - current.view.xMin;
    const dx = ((event.clientX - current.startX) / current.width) * xSpan;
    const next: ViewState = {
      ...current.view,
      xMin: current.view.xMin - dx,
      xMax: current.view.xMax - dx
    };

    if (current.view.yMin !== null && current.view.yMax !== null) {
      const ySpan = current.view.yMax - current.view.yMin;
      const dy =
        ((event.clientY - current.startY) / current.height) * ySpan;
      next.yMin = current.view.yMin + dy;
      next.yMax = current.view.yMax + dy;
    } else {
      const ySpan = computedView.yMax - computedView.yMin;
      const dy =
        ((event.clientY - current.startY) / current.height) * ySpan;
      next.yMin = computedView.yMin + dy;
      next.yMax = computedView.yMax + dy;
    }
    setView(next);
  };

  const handlePointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (drag.current?.pointerId !== event.pointerId) {
      return;
    }
    drag.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  const exportPlot = async (format: "svg" | "png") => {
    const baseName = `${region.id}-grafico`;
    try {
      if (format === "svg") {
        await savePlotSvg(svg, baseName);
      } else {
        await savePlotPng(svg, size.width, size.height, baseName);
      }
    } catch (caught) {
      setError(String(caught));
    }
  };

  return (
    <div className="plot-region">
      <div className="plot-toolbar" contentEditable={false}>
        {region.series.map((item, index) => (
          <div className="plot-series-row" key={`${region.id}-${index}`}>
            <select
              className="plot-kind"
              value={item.kind ?? "explicit"}
              title="Tipo de gráfico"
              onChange={(event) =>
                updateSeries(index, {
                  kind: event.target.value as
                    | "explicit"
                    | "parametric"
                    | "implicit"
                    | "surface"
                })
              }
            >
              <option value="explicit">y=f(x)</option>
              <option value="parametric">x(t), y(t)</option>
              <option value="implicit">F(x,y)=0</option>
              <option value="surface">z=f(x,y)</option>
            </select>
            <input
              className="plot-expression"
              value={item.expression}
              placeholder={
                item.kind === "parametric"
                  ? "x(t)"
                  : item.kind === "implicit"
                    ? "x^2+y^2-4"
                    : item.kind === "surface"
                      ? "f(x,y)"
                      : "f(x)"
              }
              title="Expresión de la serie"
              onChange={(event) =>
                updateSeries(index, { expression: event.target.value })
              }
            />
            {(item.kind ?? "explicit") === "surface" && (
              <>
                <input
                  className="plot-range"
                  type="number"
                  value={region.xMin}
                  title="x mínimo"
                  onChange={(event) =>
                    onChange({ xMin: Number(event.target.value) })
                  }
                />
                <input
                  className="plot-range"
                  type="number"
                  value={region.xMax}
                  title="x máximo"
                  onChange={(event) =>
                    onChange({ xMax: Number(event.target.value) })
                  }
                />
                <input
                  className="plot-range"
                  type="number"
                  value={region.yMin ?? -5}
                  title="y mínimo"
                  onChange={(event) =>
                    onChange({ yMin: Number(event.target.value) })
                  }
                />
                <input
                  className="plot-range"
                  type="number"
                  value={region.yMax ?? 5}
                  title="y máximo"
                  onChange={(event) =>
                    onChange({ yMax: Number(event.target.value) })
                  }
                />
              </>
            )}
            {(item.kind ?? "explicit") === "parametric" && (
              <>
                <input
                  className="plot-expression"
                  value={item.expression2 ?? ""}
                  placeholder="y(t)"
                  title="Segunda expresión paramétrica"
                  onChange={(event) =>
                    updateSeries(index, { expression2: event.target.value })
                  }
                />
                <input
                  className="plot-range"
                  type="number"
                  value={item.parameterMin ?? -10}
                  title="t mínimo"
                  onChange={(event) =>
                    updateSeries(index, {
                      parameterMin: Number(event.target.value)
                    })
                  }
                />
                <input
                  className="plot-range"
                  type="number"
                  value={item.parameterMax ?? 10}
                  title="t máximo"
                  onChange={(event) =>
                    updateSeries(index, {
                      parameterMax: Number(event.target.value)
                    })
                  }
                />
              </>
            )}
            <input
              className="plot-color"
              type="color"
              value={item.color}
              title="Color de la serie"
              onChange={(event) =>
                updateSeries(index, { color: event.target.value })
              }
            />
            <button
              type="button"
              className={item.visible ? "active" : ""}
              title={item.visible ? "Ocultar serie" : "Mostrar serie"}
              onClick={() =>
                updateSeries(index, { visible: !item.visible })
              }
            >
              ◉
            </button>
            <button
              type="button"
              title="Eliminar serie"
              onClick={() => removeSeries(index)}
            >
              ×
            </button>
          </div>
        ))}
        <div className="plot-actions">
          <button type="button" onClick={addSeries} title="Añadir serie">
            + Serie
          </button>
          <button type="button" onClick={resetView} title="Restablecer vista">
            Ajustar
          </button>
          {hasSurface && (
            <button
              type="button"
              className={surfaceView.wireframe ? "active" : ""}
              title="Mostrar u ocultar la malla"
              onClick={() =>
                setSurfaceView((current) => ({
                  ...current,
                  wireframe: !current.wireframe
                }))
              }
            >
              Malla
            </button>
          )}
          <button type="button" onClick={() => void exportPlot("svg")}>
            SVG
          </button>
          <button type="button" onClick={() => void exportPlot("png")}>
            PNG
          </button>
          {loading && <span className="plot-status">…</span>}
        </div>
      </div>

      <div
        ref={canvasRef}
        className="plot-canvas"
        title={
          displayError ||
          (hasSurface
            ? "Arrastra para rotar, rueda para zoom"
            : "Rueda para zoom, arrastra para desplazar")
        }
        onWheel={handleWheel}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onLostPointerCapture={handlePointerUp}
        dangerouslySetInnerHTML={{ __html: svg }}
      />
      {displayError && <span className="plot-error">{displayError}</span>}
    </div>
  );
}

function paletteColor(index: number): string {
  const colors = ["#2563eb", "#dc2626", "#059669", "#d97706", "#7c3aed"];
  return colors[index % colors.length];
}
