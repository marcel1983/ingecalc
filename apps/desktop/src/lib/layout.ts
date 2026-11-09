export type LayoutBox = {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  pinned?: boolean;
  role?: string;
};

export const REGION_GAP = 12;

export function horizontalOverlap(left: LayoutBox, right: LayoutBox): boolean {
  return left.x < right.x + right.w && right.x < left.x + left.w;
}

export function rectsOverlap(left: LayoutBox, right: LayoutBox): boolean {
  return (
    horizontalOverlap(left, right) &&
    left.y < right.y + right.h &&
    right.y < left.y + left.h
  );
}

/**
 * Push the regions below `targetId` down until none overlap it.
 *
 * The function only moves boxes in the same horizontal band, so two columns
 * can grow independently. Pinned boxes are never moved; when one blocks the
 * reflow the caller should move the growing region to another page instead.
 */
export function resolveVerticalOverlaps(
  regions: LayoutBox[],
  targetId: string,
  gap = REGION_GAP
): { positions: Map<string, number>; blockedByPinned: boolean } {
  const target = regions.find((region) => region.id === targetId);
  const positions = new Map<string, number>();
  if (!target) {
    return { positions, blockedByPinned: false };
  }

  const blockers: LayoutBox[] = [{ ...target }];
  const candidates = regions
    .filter(
      (region) =>
        region.id !== targetId && !region.role && region.y + region.h > target.y
    )
    .sort((left, right) => left.y - right.y || left.x - right.x);

  let blockedByPinned = false;

  for (const candidate of candidates) {
    const blockersInBand = blockers.filter((blocker) =>
      horizontalOverlap(candidate, blocker)
    );
    if (blockersInBand.length === 0) {
      continue;
    }

    const lowest = Math.max(
      ...blockersInBand.map((blocker) => blocker.y + blocker.h)
    );
    let next = candidate;
    if (candidate.y < lowest + gap) {
      if (candidate.pinned) {
        blockedByPinned = true;
        continue;
      }
      const y = Math.ceil(lowest + gap);
      positions.set(candidate.id, y);
      next = { ...candidate, y };
    }

    blockers.push(next);
  }

  return { positions, blockedByPinned };
}

/**
 * Find the first vertical position at or below `startY` where `box` does not
 * overlap any region. Used when a region is moved to the next page.
 */
export function findFreeY(
  regions: LayoutBox[],
  box: LayoutBox,
  startY: number,
  gap = REGION_GAP
): number {
  let y = startY;
  let changed = true;
  let guard = 0;

  while (changed && guard < 500) {
    changed = false;
    for (const region of regions) {
      if (region.id === box.id || region.role) {
        continue;
      }
      if (!horizontalOverlap(box, region)) {
        continue;
      }
      if (y < region.y + region.h + gap && y + box.h > region.y) {
        y = region.y + region.h + gap;
        changed = true;
      }
    }
    guard += 1;
  }

  return y;
}
