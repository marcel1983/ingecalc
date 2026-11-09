export const MIN_CONTENT_WIDTH_CHARS = 5;

export function isMathSourceEmpty(source: string): boolean {
  const trimmed = source
    .replace(/[\u200B-\u200D\uFEFF]/g, "")
    .trim();
  return trimmed === "" || trimmed === "\\placeholder{}";
}

export function isTextHtmlEmpty(html: string): boolean {
  if (!html.trim()) {
    return true;
  }

  const text = html
    .replace(/<br\s*\/?>/gi, "")
    .replace(/<[^>]+>/g, "")
    .replace(/&nbsp;|&#160;|&#xa0;/gi, " ")
    .replace(/[\u200B-\u200D\uFEFF]/g, "")
    .trim();

  if (text) {
    return false;
  }

  return !/<(img|table|hr|span[^>]*data-inline-math|math-inline)\b/i.test(html);
}

export function isRegionElementEmpty(
  element: HTMLElement,
  type: "math" | "text"
): boolean {
  if (type === "math") {
    const field = element.querySelector("math-field") as
      | (HTMLElement & { value?: string })
      | null;
    return isMathSourceEmpty(field?.value ?? "");
  }

  const editor = element.querySelector(".tiptap");
  if (!editor) {
    return true;
  }

  if (editor.querySelector("img, table, hr, [data-inline-math]")) {
    return false;
  }

  return (editor.textContent ?? "")
    .replace(/[\u200B-\u200D\uFEFF]/g, "")
    .trim() === "";
}
