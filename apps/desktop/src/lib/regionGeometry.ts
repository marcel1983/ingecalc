export type Rect = {
  x: number;
  y: number;
  w: number;
  h: number;
};

export type PageSize = {
  width: number;
  height: number;
};

export type PageMargins = {
  top: number;
  right: number;
  bottom: number;
  left: number;
};

export type ContentBounds = {
  left: number;
  top: number;
  right: number;
  bottom: number;
};

export const MIN_REGION_WIDTH = 120;
export const MIN_REGION_HEIGHT = 48;

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function contentBounds(
  page: PageSize,
  margins: PageMargins
): ContentBounds {
  const left = clamp(margins.left, 0, Math.max(0, page.width - MIN_REGION_WIDTH));
  const top = clamp(margins.top, 0, Math.max(0, page.height - MIN_REGION_HEIGHT));
  return {
    left,
    top,
    right: Math.max(left + MIN_REGION_WIDTH, page.width - margins.right),
    bottom: Math.max(top + MIN_REGION_HEIGHT, page.height - margins.bottom)
  };
}

function snapValue(value: number, step: number): number {
  return step > 0 ? Math.round(value / step) * step : Math.round(value);
}

export function moveRect(
  rect: Rect,
  dx: number,
  dy: number,
  page: PageSize,
  snap = 0
): Rect {
  const x = snapValue(
    clamp(rect.x + dx, 0, Math.max(0, page.width - rect.w)),
    snap
  );
  const y = snapValue(
    clamp(rect.y + dy, 0, Math.max(0, page.height - rect.h)),
    snap
  );
  return {
    ...rect,
    x: clamp(x, 0, Math.max(0, page.width - rect.w)),
    y: clamp(y, 0, Math.max(0, page.height - rect.h))
  };
}

export function resizeRect(
  rect: Rect,
  dw: number,
  dh: number,
  page: PageSize,
  snap = 0
): Rect {
  const width = snapValue(
    clamp(
      rect.w + dw,
      MIN_REGION_WIDTH,
      Math.max(MIN_REGION_WIDTH, page.width - rect.x)
    ),
    snap
  );
  const height = snapValue(
    clamp(
      rect.h + dh,
      MIN_REGION_HEIGHT,
      Math.max(MIN_REGION_HEIGHT, page.height - rect.y)
    ),
    snap
  );
  return {
    ...rect,
    w: clamp(width, MIN_REGION_WIDTH, Math.max(MIN_REGION_WIDTH, page.width - rect.x)),
    h: clamp(
      height,
      MIN_REGION_HEIGHT,
      Math.max(MIN_REGION_HEIGHT, page.height - rect.y)
    )
  };
}

export function moveRectInBounds(
  rect: Rect,
  dx: number,
  dy: number,
  page: PageSize,
  bounds: ContentBounds,
  snap = 0
): Rect {
  const maxX = Math.max(bounds.left, bounds.right - rect.w);
  const maxY = Math.max(bounds.top, bounds.bottom - rect.h);
  const x = snapValue(clamp(rect.x + dx, bounds.left, maxX), snap);
  const y = snapValue(clamp(rect.y + dy, bounds.top, maxY), snap);
  return {
    ...rect,
    x: clamp(x, bounds.left, Math.min(maxX, page.width - rect.w)),
    y: clamp(y, bounds.top, Math.min(maxY, page.height - rect.h))
  };
}

export function resizeRectInBounds(
  rect: Rect,
  dw: number,
  dh: number,
  page: PageSize,
  bounds: ContentBounds,
  snap = 0
): Rect {
  const maxWidth = Math.max(
    MIN_REGION_WIDTH,
    Math.min(bounds.right, page.width) - rect.x
  );
  const maxHeight = Math.max(
    MIN_REGION_HEIGHT,
    Math.min(bounds.bottom, page.height) - rect.y
  );
  const width = snapValue(
    clamp(rect.w + dw, MIN_REGION_WIDTH, maxWidth),
    snap
  );
  const height = snapValue(
    clamp(rect.h + dh, MIN_REGION_HEIGHT, maxHeight),
    snap
  );
  return {
    ...rect,
    w: clamp(width, MIN_REGION_WIDTH, maxWidth),
    h: clamp(height, MIN_REGION_HEIGHT, maxHeight)
  };
}
