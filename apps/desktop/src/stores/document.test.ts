import { beforeEach, describe, expect, it } from "vitest";
import { newDocument, useDocumentStore } from "./document";

function reset() {
  useDocumentStore.getState().setDoc(newDocument());
  useDocumentStore.getState().clearClipboard();
  useDocumentStore.getState().endGroupMove();
}

function mathRegionId(): string {
  const page = useDocumentStore.getState().doc.pages[0];
  const region = page.regions.find((item) => item.type === "math");
  if (!region) {
    throw new Error("expected a math region");
  }
  return region.id;
}

function textRegionId(): string {
  const page = useDocumentStore.getState().doc.pages[0];
  const region = page.regions.find((item) => item.type === "text");
  if (!region) {
    throw new Error("expected a text region");
  }
  return region.id;
}

describe("document store", () => {
  beforeEach(reset);

  it("adds a region and supports undo/redo", () => {
    const store = useDocumentStore.getState();
    expect(store.doc.pages[0].regions).toHaveLength(2);

    store.addMathRegion();
    expect(useDocumentStore.getState().doc.pages[0].regions).toHaveLength(3);
    expect(useDocumentStore.getState().past).toHaveLength(1);

    useDocumentStore.getState().undo();
    expect(useDocumentStore.getState().doc.pages[0].regions).toHaveLength(2);
    expect(useDocumentStore.getState().selectedIds).toEqual([]);

    useDocumentStore.getState().redo();
    expect(useDocumentStore.getState().doc.pages[0].regions).toHaveLength(3);
  });

  it("creates an empty math region at a page point", () => {
    const page = useDocumentStore.getState().doc.pages[0];
    const regionId = useDocumentStore
      .getState()
      .addMathRegionAt(page.id, 200, 150);

    expect(regionId).not.toBeNull();
    const region = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === regionId);
    expect(region).toMatchObject({
      type: "math",
      x: 200,
      y: 150,
      source: "",
      ascii: ""
    });
    expect(useDocumentStore.getState().selectedIds).toEqual([regionId]);

    useDocumentStore.getState().undo();
    expect(
      useDocumentStore
        .getState()
        .doc.pages[0].regions.find((item) => item.id === regionId)
    ).toBeUndefined();
  });

  it("creates an empty math region below another one", () => {
    const store = useDocumentStore.getState();
    const sourceId = mathRegionId();
    const textId = textRegionId();
    store.removeRegion(textId);

    const belowId = useDocumentStore.getState().addMathRegionBelow(sourceId);
    expect(belowId).not.toBeNull();

    const source = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === sourceId);
    const below = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === belowId);
    expect(below).toMatchObject({
      type: "math",
      x: 60,
      y: 124,
      source: "",
      ascii: ""
    });
    expect(source ? below!.y >= source.y + source.h : false).toBe(true);
    expect(useDocumentStore.getState().selectedIds).toEqual([belowId]);

    useDocumentStore.getState().undo();
    expect(
      useDocumentStore
        .getState()
        .doc.pages[0].regions.find((item) => item.id === belowId)
    ).toBeUndefined();
  });

  it("avoids overlapping existing regions when inserting below", () => {
    const sourceId = mathRegionId();
    const textId = textRegionId();
    const text = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === textId)!;

    const belowId = useDocumentStore.getState().addMathRegionBelow(sourceId);
    const below = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === belowId)!;

    expect(below.y).toBeGreaterThanOrEqual(text.y + text.h + 16);
  });

  it("adds a new page when there is no room below", () => {
    const doc = newDocument();
    doc.pages[0].height = 200;
    doc.pages[0].regions = [doc.pages[0].regions[0]];
    doc.pages[0].regions[0].y = 120;
    doc.pages[0].regions[0].h = 48;
    useDocumentStore.getState().setDoc(doc);
    const sourceId = doc.pages[0].regions[0].id;

    const belowId = useDocumentStore.getState().addMathRegionBelow(sourceId);
    expect(belowId).not.toBeNull();
    expect(useDocumentStore.getState().doc.pages).toHaveLength(2);

    const page = useDocumentStore.getState().doc.pages[1];
    const below = page.regions.find((region) => region.id === belowId);
    expect(below).toMatchObject({
      type: "math",
      y: 48,
      source: ""
    });
  });

  it("creates header and footer text regions in the page margins", () => {
    useDocumentStore.getState().addHeaderRegion();
    useDocumentStore.getState().addFooterRegion();

    const regions = useDocumentStore.getState().doc.pages[0].regions;
    const header = regions.find(
      (region) => region.type === "text" && region.role === "header"
    );
    const footer = regions.find(
      (region) => region.type === "text" && region.role === "footer"
    );

    expect(header).toMatchObject({
      type: "text",
      x: 48,
      y: 0,
      w: 754,
      h: 48,
      role: "header"
    });
    if (header?.type !== "text") {
      throw new Error("expected header text region");
    }
    expect(header.html).toContain("Encabezado");
    expect(footer).toMatchObject({
      type: "text",
      x: 48,
      y: 1052,
      w: 754,
      h: 48,
      role: "footer"
    });
    if (footer?.type !== "text") {
      throw new Error("expected footer text region");
    }
    expect(footer.html).toContain("Pie de página");

    useDocumentStore.getState().addHeaderRegion();
    expect(
      useDocumentStore
        .getState()
        .doc.pages[0].regions.filter(
          (region) => region.type === "text" && region.role === "header"
        )
    ).toHaveLength(1);
  });

  it("records a drag as a single history entry", () => {
    const regionId = mathRegionId();
    const store = useDocumentStore.getState();

    store.beginTransaction(`move:${regionId}`);
    store.updateRegion(regionId, { x: 120, y: 300 });
    store.updateRegion(regionId, { x: 160, y: 320 });
    store.endTransaction();

    expect(useDocumentStore.getState().past).toHaveLength(1);

    const moved = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((region) => region.id === regionId);
    expect(moved).toMatchObject({ x: 160, y: 320 });

    useDocumentStore.getState().undo();
    const original = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((region) => region.id === regionId);
    expect(original).toMatchObject({ x: 60, y: 60 });
  });

  it("selects with Ctrl and duplicates/removes the selection", () => {
    const store = useDocumentStore.getState();
    const mathId = mathRegionId();
    const textId = textRegionId();

    store.select(mathId);
    useDocumentStore.getState().select(textId, true);
    expect(useDocumentStore.getState().selectedIds).toEqual([mathId, textId]);

    useDocumentStore.getState().duplicateSelected();
    const duplicated = useDocumentStore.getState();
    expect(duplicated.doc.pages[0].regions).toHaveLength(4);
    expect(duplicated.selectedIds).toHaveLength(2);

    useDocumentStore.getState().removeSelected();
    expect(useDocumentStore.getState().doc.pages[0].regions).toHaveLength(2);
    expect(useDocumentStore.getState().selectedIds).toEqual([]);
  });

  it("adds and removes pages, and undo restores them", () => {
    useDocumentStore.getState().addPage();
    const added = useDocumentStore.getState();
    expect(added.doc.pages).toHaveLength(2);
    const pageId = added.doc.pages[1].id;

    useDocumentStore.getState().removePage(pageId);
    expect(useDocumentStore.getState().doc.pages).toHaveLength(1);

    useDocumentStore.getState().undo();
    expect(useDocumentStore.getState().doc.pages).toHaveLength(2);

    const onlyPage = useDocumentStore.getState().doc.pages[0].id;
    useDocumentStore.getState().removePage(onlyPage);
    expect(useDocumentStore.getState().doc.pages).toHaveLength(1);
  });

  it("stores document evaluation results and clears them on new documents", () => {
    useDocumentStore.getState().setMathResults({
      "math-1": { display: "2.3 m", error: "" }
    });
    expect(useDocumentStore.getState().mathResults["math-1"]).toEqual({
      display: "2.3 m",
      error: ""
    });

    useDocumentStore.getState().newDoc();
    expect(useDocumentStore.getState().mathResults).toEqual({});
  });

  it("adds a math region with a palette template", () => {
    useDocumentStore
      .getState()
      .addMathRegionWith("\\sin\\left(x\\right)", "sin(x)");

    const state = useDocumentStore.getState();
    const math = state.doc.pages[state.doc.pages.length - 1].regions.find(
      (region) => region.type === "math" && region.source.includes("sin")
    );

    expect(math).toMatchObject({
      type: "math",
      source: "\\sin\\left(x\\right)",
      ascii: "sin(x)"
    });
    expect(state.selectedIds).toContain(math?.id);
  });

  it("adds an image region with its data URL and dimensions", () => {
    const src = "data:image/png;base64,iVBORw0KGgo=";
    useDocumentStore.getState().addImageRegion(src, 96, 64);

    const state = useDocumentStore.getState();
    const image = state.doc.pages[state.doc.pages.length - 1].regions.find(
      (region) => region.type === "image"
    );

    expect(image).toMatchObject({
      type: "image",
      src,
      w: 96,
      h: 64
    });
    expect(state.selectedIds).toContain(image?.id);
  });

  it("moves pages and undo restores their order", () => {
    useDocumentStore.getState().addPage();
    const firstId = useDocumentStore.getState().doc.pages[0].id;
    useDocumentStore.getState().movePage(firstId, 1);
    expect(useDocumentStore.getState().doc.pages[1].id).toBe(firstId);

    useDocumentStore.getState().undo();
    expect(useDocumentStore.getState().doc.pages[0].id).toBe(firstId);
  });

  it("updates page setup and undo restores the previous setup", () => {
    const store = useDocumentStore.getState();
    const initial = store.doc.settings.page;

    store.updatePageSetup({
      ...initial,
      marginTop: 96,
      marginLeft: 72,
      header: "Informe {page}",
      footer: "Página {page}"
    });

    const updated = useDocumentStore.getState();
    expect(updated.doc.settings.page.marginTop).toBe(96);
    expect(updated.doc.settings.page.marginLeft).toBe(72);
    expect(updated.doc.settings.page.header).toBe("Informe {page}");
    expect(updated.past).toHaveLength(1);

    useDocumentStore.getState().undo();
    const restored = useDocumentStore.getState().doc.settings.page;
    expect(restored.marginTop).toBe(48);
    expect(restored.marginLeft).toBe(48);
    expect(restored.header).toBe("");
    expect(restored.footer).toBe("");
  });

  it("changes page size and keeps regions inside the page", () => {
    useDocumentStore.getState().setPageSize(400, 300);
    const page = useDocumentStore.getState().doc.pages[0];

    expect(page.width).toBe(400);
    expect(page.height).toBe(300);
    for (const region of page.regions) {
      expect(region.x).toBeGreaterThanOrEqual(0);
      expect(region.y).toBeGreaterThanOrEqual(0);
      expect(region.x + region.w).toBeLessThanOrEqual(400);
      expect(region.y + region.h).toBeLessThanOrEqual(300);
    }

    useDocumentStore.getState().undo();
    expect(useDocumentStore.getState().doc.pages[0].width).toBe(850);
  });

  it("copies, pastes and cuts regions", () => {
    const mathId = mathRegionId();
    useDocumentStore.getState().select(mathId);

    expect(useDocumentStore.getState().copySelected()).toBe(1);
    expect(useDocumentStore.getState().pasteClipboard()).toBe(1);

    let state = useDocumentStore.getState();
    expect(state.doc.pages[0].regions).toHaveLength(3);
    expect(state.selectedIds).toHaveLength(1);
    const pasted = state.doc.pages[0].regions.find(
      (region) => region.id === state.selectedIds[0]
    );
    expect(pasted).toMatchObject({ x: 84, y: 84 });

    expect(useDocumentStore.getState().cutSelected()).toBe(1);
    state = useDocumentStore.getState();
    expect(state.doc.pages[0].regions).toHaveLength(2);

    expect(useDocumentStore.getState().pasteClipboard()).toBe(1);
    expect(useDocumentStore.getState().doc.pages[0].regions).toHaveLength(3);
  });

  it("moves a multi-selection as a group and records one history entry", () => {
    const mathId = mathRegionId();
    const textId = textRegionId();
    const pageId = useDocumentStore.getState().doc.pages[0].id;

    useDocumentStore.getState().selectMany([mathId, textId]);
    useDocumentStore.getState().beginTransaction("group");
    useDocumentStore.getState().beginGroupMove(pageId);
    useDocumentStore.getState().moveGroup(100, 80);
    useDocumentStore.getState().endGroupMove();
    useDocumentStore.getState().endTransaction();

    const page = useDocumentStore.getState().doc.pages[0];
    expect(page.regions.find((region) => region.id === mathId)).toMatchObject({
      x: 160,
      y: 140
    });
    expect(page.regions.find((region) => region.id === textId)).toMatchObject({
      x: 160,
      y: 220
    });
    expect(useDocumentStore.getState().past).toHaveLength(1);

    useDocumentStore.getState().undo();
    const restored = useDocumentStore.getState().doc.pages[0];
    expect(restored.regions.find((region) => region.id === mathId)).toMatchObject({
      x: 60,
      y: 60
    });
    expect(restored.regions.find((region) => region.id === textId)).toMatchObject({
      x: 60,
      y: 140
    });
  });

  it("clamps a group move to the page while preserving offsets", () => {
    const mathId = mathRegionId();
    const textId = textRegionId();
    const pageId = useDocumentStore.getState().doc.pages[0].id;

    useDocumentStore.getState().selectMany([mathId, textId]);
    useDocumentStore.getState().beginTransaction("group");
    useDocumentStore.getState().beginGroupMove(pageId);
    useDocumentStore.getState().moveGroup(5000, 5000);
    useDocumentStore.getState().endGroupMove();
    useDocumentStore.getState().endTransaction();

    const page = useDocumentStore.getState().doc.pages[0];
    const math = page.regions.find((region) => region.id === mathId);
    const text = page.regions.find((region) => region.id === textId);
    expect(math).toMatchObject({ x: 302, y: 872 });
    expect(text).toMatchObject({ x: 302, y: 952 });
    expect(math!.x + math!.w).toBeLessThanOrEqual(page.width - 48);
    expect(text!.y + text!.h).toBeLessThanOrEqual(page.height - 48);
  });

  it("does not coalesce different actions", () => {
    const store = useDocumentStore.getState();
    const mathId = mathRegionId();

    store.updateRegion(mathId, { source: "3+3", ascii: "3+3" });
    useDocumentStore.getState().updateRegion(mathId, { x: 100 });
    expect(useDocumentStore.getState().past).toHaveLength(2);
  });

  it("syncs auto-fit sizes without adding history entries", () => {
    const mathId = mathRegionId();
    const historyBefore = useDocumentStore.getState().past.length;

    useDocumentStore.getState().syncRegionSize(mathId, 88, 44);

    const region = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((item) => item.id === mathId);
    expect(region).toMatchObject({ w: 88, h: 44 });
    expect(useDocumentStore.getState().past).toHaveLength(historyBefore);
  });

  it("keeps pinned regions fixed during a group move", () => {
    const mathId = mathRegionId();
    const textId = textRegionId();
    const pageId = useDocumentStore.getState().doc.pages[0].id;

    useDocumentStore.getState().updateRegion(mathId, { pinned: true });
    useDocumentStore.getState().selectMany([mathId, textId]);
    useDocumentStore.getState().beginGroupMove(pageId);
    useDocumentStore.getState().moveGroup(50, 40);
    useDocumentStore.getState().endGroupMove();

    const page = useDocumentStore.getState().doc.pages[0];
    expect(page.regions.find((region) => region.id === mathId)).toMatchObject({
      x: 60,
      y: 60
    });
    expect(page.regions.find((region) => region.id === textId)).toMatchObject({
      x: 110,
      y: 180
    });
  });

  it("pushes the boxes below when a math box grows", () => {
    const mathId = mathRegionId();
    const textId = textRegionId();

    useDocumentStore.getState().syncRegionSize(mathId, 360, 100);

    const page = useDocumentStore.getState().doc.pages[0];
    const math = page.regions.find((region) => region.id === mathId);
    const text = page.regions.find((region) => region.id === textId);
    expect(math).toMatchObject({ h: 100 });
    expect(text!.y).toBeGreaterThanOrEqual(math!.y + math!.h);
  });

  it("rejects a move that would overlap another box", () => {
    const mathId = mathRegionId();

    useDocumentStore.getState().updateRegion(mathId, { x: 60, y: 120 });

    const math = useDocumentStore
      .getState()
      .doc.pages[0].regions.find((region) => region.id === mathId);
    expect(math).toMatchObject({ x: 60, y: 60 });
  });

  it("moves an overflowing box to the next page", () => {
    const mathId = mathRegionId();

    useDocumentStore.getState().syncRegionSize(mathId, 360, 1000);

    const pages = useDocumentStore.getState().doc.pages;
    expect(pages).toHaveLength(2);
    expect(pages[0].regions.some((region) => region.id === mathId)).toBe(false);
    expect(pages[1].regions.some((region) => region.id === mathId)).toBe(true);
  });

  it("keeps boxes inside the page margins", () => {
    const mathId = mathRegionId();

    useDocumentStore.getState().updateRegion(mathId, { x: 0, y: 0 });
    const page = useDocumentStore.getState().doc.pages[0];
    const math = page.regions.find((region) => region.id === mathId);
    expect(math!.x).toBeGreaterThanOrEqual(48);
    expect(math!.y).toBeGreaterThanOrEqual(48);
  });

  it("clamps auto-fit width to the content area", () => {
    const mathId = mathRegionId();

    useDocumentStore.getState().syncRegionSize(mathId, 5000, 44);

    const page = useDocumentStore.getState().doc.pages[0];
    const math = page.regions.find((region) => region.id === mathId);
    expect(math!.x + math!.w).toBeLessThanOrEqual(
      page.width - 48
    );
  });

  it("reflows boxes when the page margins grow", () => {
    const store = useDocumentStore.getState();
    const setup = store.doc.settings.page;

    useDocumentStore.getState().updatePageSetup({
      ...setup,
      marginTop: 100,
      marginRight: 100,
      marginBottom: 100,
      marginLeft: 100
    });

    const page = useDocumentStore.getState().doc.pages[0];
    for (const region of page.regions) {
      expect(region.x).toBeGreaterThanOrEqual(100);
      expect(region.y).toBeGreaterThanOrEqual(100);
      expect(region.x + region.w).toBeLessThanOrEqual(
        page.width - 100
      );
    }
  });
});
