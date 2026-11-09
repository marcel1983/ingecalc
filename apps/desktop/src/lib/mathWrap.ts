const PLACEHOLDER = "\\placeholder{}";

/**
 * Apply a palette template to the selected LaTeX instead of replacing it.
 * Only templates with a placeholder can wrap a selection.
 */
export function wrapSelectedMath(
  template: string,
  selected: string
): string | null {
  const selection = selected.trim();
  if (!selection) {
    return null;
  }

  if (template === "\\placeholder{}^{\\placeholder{}}") {
    return `\\left(${selection}\\right)^{\\placeholder{}}`;
  }
  if (template === "\\placeholder{}^{2}") {
    return `\\left(${selection}\\right)^{2}`;
  }
  if (template === "+") {
    return `\\left(${selection}\\right)+`;
  }
  if (template === "-") {
    return `\\left(${selection}\\right)-`;
  }
  if (template === "\\times") {
    return `\\left(${selection}\\right)\\times`;
  }
  if (template === "\\div") {
    return `\\left(${selection}\\right)\\div`;
  }

  const index = template.indexOf(PLACEHOLDER);
  if (index < 0) {
    return null;
  }

  return (
    template.slice(0, index) +
    selection +
    template.slice(index + PLACEHOLDER.length)
  );
}
