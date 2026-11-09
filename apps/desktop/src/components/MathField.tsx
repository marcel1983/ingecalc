import {
  Fragment,
  useEffect,
  useRef,
  useState
} from "react";
import { createPortal } from "react-dom";
import type { MathfieldElement } from "mathlive";
import { normalizeAsciiMath } from "../lib/mathAscii";
import {
  UNIT_TOKEN_CLASS,
  isUnitRun,
  nextStartsFunctionCall
} from "../lib/unitText";
import {
  buildTokenCompletions,
  matchMathCompletions,
  type MathCompletion
} from "../lib/mathCompletions";

type Props = {
  value: string;
  readOnly?: boolean;
  onChange: (latex: string, ascii: string) => void;
  onEnter?: () => void;
  completions?: MathCompletion[];
  constantNames?: string[];
};

let lastMathField: MathfieldElement | null = null;

const UNIT_COLOR = "var(--ingecalc-unit-color, #2563eb)";
const CONSTANT_CLASS = "ingecalc-constant-token";
const CONSTANT_FONT =
  "var(--ingecalc-constant-font, 'Cambria Math', 'STIX Two Math', 'Times New Roman', serif)";
const CONSTANT_COLOR = "var(--ingecalc-constant-color, #7c3aed)";

type CompletionMenu = {
  items: MathCompletion[];
  index: number;
  token: string;
  navigated: boolean;
  x: number;
  y: number;
};

function clearUnitColors(root: ShadowRoot) {
  root
    .querySelectorAll<HTMLElement>(
      `.${UNIT_TOKEN_CLASS}, .${CONSTANT_CLASS}`
    )
    .forEach((span) => {
      span.style.color = "";
      span.style.fontFamily = "";
      span.style.fontWeight = "";
      span.classList.remove(UNIT_TOKEN_CLASS, CONSTANT_CLASS);
    });
}

function colorUnitRuns(
  element: MathfieldElement,
  constantNames: Set<string>
) {
  const root = element.shadowRoot;
  if (!root) {
    return;
  }

  clearUnitColors(root);

  const tokens = [
    ...root.querySelectorAll<HTMLElement>(".ML__mathit, .ML__text")
  ];
  let group: HTMLElement[] = [];

  const flush = () => {
    if (group.length === 0) {
      return;
    }

    const text = group.map((span) => span.textContent ?? "").join("");
    const next =
      group[group.length - 1].nextElementSibling?.textContent ?? "";
    if (isUnitRun(text) && !nextStartsFunctionCall(next)) {
      for (const span of group) {
        span.classList.add(UNIT_TOKEN_CLASS);
        span.style.color = UNIT_COLOR;
      }
    } else if (constantNames.has(text)) {
      for (const span of group) {
        span.classList.add(CONSTANT_CLASS);
        span.style.color = CONSTANT_COLOR;
        span.style.fontFamily = CONSTANT_FONT;
        span.style.fontWeight = "600";
      }
    }
    group = [];
  };

  for (const token of tokens) {
    const previous = group[group.length - 1];
    const sameClass =
      previous?.classList.contains("ML__mathit") ===
      token.classList.contains("ML__mathit");
    if (
      previous &&
      sameClass &&
      token.previousElementSibling === previous
    ) {
      group.push(token);
    } else {
      flush();
      group = [token];
    }
  }
  flush();

  root.querySelectorAll<HTMLElement>(".ML__cmr").forEach((span) => {
    const text = span.textContent?.trim() ?? "";
    if (text === "°" || text === "%") {
      span.classList.add(UNIT_TOKEN_CLASS);
      span.style.color = UNIT_COLOR;
    }
  });
}

function completionGroupLabel(kind: MathCompletion["kind"]): string {
  switch (kind) {
    case "operator":
      return "Operadores";
    case "variable":
      return "Variables";
    case "function":
      return "Funciones";
    case "constant":
      return "Constantes";
  }
}

function observeUnitColors(
  element: MathfieldElement,
  constantNames: Set<string>
) {
  const root = element.shadowRoot;
  if (!root) {
    return () => undefined;
  }

  let frame = 0;
  const schedule = () => {
    window.cancelAnimationFrame(frame);
    frame = window.requestAnimationFrame(() =>
      colorUnitRuns(element, constantNames)
    );
  };

  const observer = new MutationObserver(schedule);
  observer.observe(root, {
    childList: true,
    subtree: true,
    characterData: true
  });
  schedule();

  return () => {
    observer.disconnect();
    window.cancelAnimationFrame(frame);
  };
}

export function getLastMathField(): MathfieldElement | null {
  return lastMathField;
}

export function clearLastMathField() {
  lastMathField = null;
}

export function getSelectedLatex(): string {
  const element = lastMathField;
  if (!element?.isConnected) {
    return "";
  }

  const selection = element.selection;
  if (
    !selection ||
    !selection.ranges.some(([start, end]) => start !== end)
  ) {
    return "";
  }

  try {
    return element.getValue(selection, "latex");
  } catch {
    return "";
  }
}

export function MathField({
  value,
  readOnly = false,
  onChange,
  onEnter,
  completions = [],
  constantNames = []
}: Props) {
  const ref = useRef<MathfieldElement | null>(null);
  const onChangeRef = useRef(onChange);
  const onEnterRef = useRef(onEnter);
  const completionsRef = useRef(completions);
  const completionMenuRef = useRef<CompletionMenu | null>(null);
  const constantSignature = constantNames.join("|");
  const constantNamesRef = useRef(new Set(constantNames));
  const insertCompletionRef = useRef<(item: MathCompletion) => void>(
    () => undefined
  );
  const [completionMenu, setCompletionMenu] =
    useState<CompletionMenu | null>(null);
  onChangeRef.current = onChange;
  onEnterRef.current = onEnter;
  completionsRef.current = completions;
  constantNamesRef.current = new Set(constantNames);
  completionMenuRef.current = completionMenu;

  const updateCompletionMenu = (element: MathfieldElement) => {
    const position = element.position;
    if (position <= 0) {
      setCompletionMenu(null);
      return;
    }

    const prefix = element.value.slice(0, position);
    const match = prefix.match(/([A-Za-z][A-Za-z0-9_]*)$/);
    if (!match || (match.index ?? 0) > 0 && prefix[(match.index ?? 0) - 1] === "\\") {
      setCompletionMenu(null);
      return;
    }

    const token = match[1];
    const items = buildTokenCompletions(completionsRef.current, token);
    if (items.length === 0) {
      setCompletionMenu(null);
      return;
    }

    const rect = element.getBoundingClientRect();
    setCompletionMenu({
      items,
      index: 0,
      token,
      navigated: false,
      x: rect.left,
      y: rect.bottom + 4
    });
  };

  const openAllCompletions = (element: MathfieldElement) => {
    if (readOnly) {
      return;
    }
    const position = element.position;
    const prefix = element.value.slice(0, position);
    const match = prefix.match(/([A-Za-z][A-Za-z0-9_]*)$/);
    const token = match?.[1] ?? "";
    const items = matchMathCompletions(
      completionsRef.current,
      "",
      completionsRef.current.length
    );
    if (items.length === 0) {
      setCompletionMenu(null);
      return;
    }
    const rect = element.getBoundingClientRect();
    setCompletionMenu({
      items,
      index: 0,
      token,
      navigated: false,
      x: rect.left,
      y: rect.bottom + 4
    });
  };

  const insertCompletion = (item: MathCompletion) => {
    const element = ref.current;
    const menu = completionMenuRef.current;
    if (!element || !menu) {
      return;
    }

    const end = element.position;
    const start = Math.max(0, end - menu.token.length);
    try {
      element.selection = {
        ranges: [[start, end]],
        direction: "forward"
      };
    } catch {
      // Ignore stale selections and insert after the caret instead.
    }
    element.insert(item.insert, {
      mode: "math",
      format: "latex",
      insertionMode: "replaceSelection",
      selectionMode: item.selectionMode ?? "after",
      focus: true,
      scrollIntoView: true
    });
    setCompletionMenu(null);
  };
  insertCompletionRef.current = insertCompletion;

  useEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }

    element.readOnly = readOnly;
    // Do not let MathLive rewrite what the user types. Suggestions insert
    // symbols explicitly; ordinary letters must remain literal identifiers.
    element.inlineShortcuts = {};
    element.smartMode = false;
    element.smartFence = false;
    element.removeExtraneousParentheses = false;
    element.title =
      "Escribe para ver sugerencias; Ctrl+Espacio abre todas las funciones y operadores";
    if (element.value !== value) {
      element.value = value;
      setCompletionMenu(null);
    }
    const colorFrame = window.requestAnimationFrame(() =>
      colorUnitRuns(element, constantNamesRef.current)
    );

    return () => {
      window.cancelAnimationFrame(colorFrame);
    };
  }, [value, readOnly, constantSignature]);

  useEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }

    const handleFocus = () => {
      lastMathField = element;
    };
    const handleBlur = () => {
      setCompletionMenu(null);
    };
    const handler = () => {
      onChangeRef.current(
        element.value,
        normalizeAsciiMath(element.getValue("ascii-math"), element.value)
      );
      updateCompletionMenu(element);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        !readOnly &&
        !event.isComposing &&
        (event.ctrlKey || event.metaKey) &&
        (event.code === "Space" || event.key === " ")
      ) {
        event.preventDefault();
        event.stopPropagation();
        openAllCompletions(element);
        return;
      }

      const menu = completionMenuRef.current;
      if (menu && !readOnly && !event.isComposing) {
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault();
          event.stopPropagation();
          const delta = event.key === "ArrowDown" ? 1 : -1;
          const index =
            (menu.index + delta + menu.items.length) % menu.items.length;
          setCompletionMenu({ ...menu, index, navigated: true });
          return;
        }
        if (event.key === "Enter") {
          if (menu.navigated) {
            event.preventDefault();
            event.stopPropagation();
            insertCompletionRef.current(menu.items[menu.index]);
            return;
          }
          event.preventDefault();
          event.stopPropagation();
          setCompletionMenu(null);
          return;
        }
        if (event.key === "Tab") {
          event.preventDefault();
          event.stopPropagation();
          insertCompletionRef.current(menu.items[menu.index]);
          return;
        }
        if (event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          setCompletionMenu(null);
          return;
        }
      }

      if (
        event.key !== "Enter" ||
        event.shiftKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.altKey ||
        event.isComposing ||
        readOnly
      ) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      onEnterRef.current?.();
    };

    const stopObserving = observeUnitColors(
      element,
      constantNamesRef.current
    );
    element.addEventListener("focus", handleFocus);
    element.addEventListener("blur", handleBlur);
    element.addEventListener("input", handler);
    element.addEventListener("keydown", handleKeyDown, true);
    return () => {
      stopObserving();
      element.removeEventListener("focus", handleFocus);
      element.removeEventListener("blur", handleBlur);
      element.removeEventListener("input", handler);
      element.removeEventListener("keydown", handleKeyDown, true);
      if (lastMathField === element) {
        lastMathField = null;
      }
    };
  }, [readOnly, constantSignature]);

  useEffect(() => {
    if (!completionMenu) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest(".math-completion-menu")) {
        setCompletionMenu(null);
      }
    };
    window.addEventListener("pointerdown", handlePointerDown);
    return () => window.removeEventListener("pointerdown", handlePointerDown);
  }, [completionMenu]);

  return (
    <>
      <math-field ref={ref} className="math-field" />
      {completionMenu &&
        createPortal(
          <div
            className="math-completion-menu"
            style={{ left: completionMenu.x, top: completionMenu.y }}
          >
            {completionMenu.items.map((item, index) => (
              <Fragment key={item.id}>
                {completionMenu.items[index - 1]?.kind !== item.kind && (
                  <div className="math-completion-group">
                    {completionGroupLabel(item.kind)}
                  </div>
                )}
                <button
                  type="button"
                  className={index === completionMenu.index ? "active" : ""}
                  onMouseDown={(event) => event.preventDefault()}
                  onClick={() => insertCompletion(item)}
                >
                  <span className="math-completion-name">
                    {item.label ?? item.name}
                  </span>
                  <span className="math-completion-kind">
                    {item.kind === "function"
                      ? "ƒ"
                      : item.kind === "constant"
                        ? "π"
                        : item.kind === "operator"
                          ? "±"
                          : "x"}
                  </span>
                  <span className="math-completion-description">
                    {item.description}
                  </span>
                </button>
              </Fragment>
            ))}
          </div>,
          document.body
        )}
    </>
  );
}
