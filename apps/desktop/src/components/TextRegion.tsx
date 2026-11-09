import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { Color } from "@tiptap/extension-color";
import { FontFamily, FontSize, TextStyle } from "@tiptap/extension-text-style";
import {
  Table,
  TableCell,
  TableHeader,
  TableRow
} from "@tiptap/extension-table";
import type { TextRegion as TextRegionType } from "../types/document";
import { InlineMath, latexToAscii } from "./InlineMath";
import { clearLastMathField } from "./MathField";
import { SpellcheckExtension, applySpellchecker } from "../lib/tiptapSpellcheck";
import {
  SPELLCHECK_LANGUAGES,
  getSpellchecker,
  isSpellcheckLanguage,
  suggestWords
} from "../lib/spellcheck";
import type NSpell from "nspell";
import { useTranslation } from "react-i18next";

type Props = {
  region: TextRegionType;
  selected: boolean;
  documentLanguage: string;
  personalDictionary: string[];
  onChange: (html: string) => void;
  onLanguageChange: (language: string) => void;
  onAddToDictionary: (word: string) => void;
};

type SuggestionMenu = {
  x: number;
  y: number;
  word: string;
  from: number;
  to: number;
  items: string[];
};

export function TextRegion({
  region,
  selected,
  documentLanguage,
  personalDictionary,
  onChange,
  onLanguageChange,
  onAddToDictionary
}: Props) {
  const { t } = useTranslation();
  const lastHtml = useRef(region.html);
  const [checker, setChecker] = useState<NSpell | null>(null);
  const [suggestions, setSuggestions] = useState<SuggestionMenu | null>(null);
  const [ignoredWords, setIgnoredWords] = useState<Set<string>>(
    () => new Set()
  );
  const spellLanguage = isSpellcheckLanguage(documentLanguage || "es")
    ? documentLanguage || "es"
    : "none";
  const acceptedWords = new Set(
    [...personalDictionary, ...ignoredWords].map((word) =>
      word.toLocaleLowerCase()
    )
  );
  const acceptedSignature = [...acceptedWords].sort().join("|");

  const editor = useEditor({
    extensions: [
      StarterKit,
      TextStyle,
      Color,
      FontSize,
      FontFamily,
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      TableCell,
      Placeholder.configure({
        placeholder: "Escribe texto..."
      }),
      InlineMath,
      SpellcheckExtension
    ],
    content: region.html,
    editorProps: {
      attributes: {
        spellcheck: "false",
        autocorrect: "off",
        autocapitalize: "off"
      }
    },
    onUpdate: ({ editor: currentEditor }) => {
      const html = currentEditor.getHTML();
      lastHtml.current = html;
      onChange(html);
    },
    onFocus: () => {
      clearLastMathField();
    }
  });

  useEffect(() => {
    if (!editor || region.html === lastHtml.current) {
      return;
    }

    lastHtml.current = region.html;
    editor.commands.setContent(region.html, { emitUpdate: false });
  }, [editor, region.html]);

  useEffect(() => {
    if (!editor) {
      return;
    }

    editor.view.dom.lang = documentLanguage || "es";
    editor.view.dom.dir =
      (documentLanguage || "es") === "ar" ? "rtl" : "auto";
  }, [editor, documentLanguage]);

  useEffect(() => {
    if (!editor) {
      return;
    }

    let cancelled = false;
    setChecker(null);
    setSuggestions(null);
    setIgnoredWords(new Set());

    if (spellLanguage === "none") {
      return;
    }

    void getSpellchecker(spellLanguage).then((loaded) => {
      if (cancelled || !editor) {
        return;
      }
      setChecker(loaded);
    });

    return () => {
      cancelled = true;
    };
  }, [editor, spellLanguage]);

  useEffect(() => {
    if (!editor) {
      return;
    }

    applySpellchecker(
      editor.view,
      checker,
      spellLanguage,
      acceptedWords
    );
  }, [editor, checker, spellLanguage, acceptedSignature]);

  useEffect(() => {
    if (!editor || !checker) {
      return;
    }

    const dom = editor.view.dom;
    const handleContextMenu = (event: MouseEvent) => {
      const target = (event.target as HTMLElement | null)?.closest<HTMLElement>(
        ".spell-error"
      );
      if (!target) {
        return;
      }

      const word = target.dataset.word ?? target.textContent ?? "";
      if (!word) {
        return;
      }

      let position: number;
      try {
        position = editor.view.posAtDOM(target, 0);
      } catch {
        return;
      }

      event.preventDefault();
      setSuggestions({
        x: event.clientX,
        y: event.clientY,
        word,
        from: position,
        to: position + word.length,
        items: suggestWords(checker, word)
      });
    };

    dom.addEventListener("contextmenu", handleContextMenu);
    return () => dom.removeEventListener("contextmenu", handleContextMenu);
  }, [editor, checker]);

  useEffect(() => {
    if (!suggestions) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest(".spellcheck-menu")) {
        setSuggestions(null);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setSuggestions(null);
      }
    };

    window.addEventListener("pointerdown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("pointerdown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [suggestions]);

  const replaceSuggestion = (word: string) => {
    if (!editor || !suggestions) {
      return;
    }

    editor
      .chain()
      .focus()
      .insertContentAt(
        { from: suggestions.from, to: suggestions.to },
        word
      )
      .run();
    setSuggestions(null);
  };

  const ignoreWordOnce = (word: string) => {
    setIgnoredWords((current) => {
      const next = new Set(current);
      next.add(word.toLocaleLowerCase());
      return next;
    });
    setSuggestions(null);
  };

  const addWordToDictionary = (word: string) => {
    onAddToDictionary(word);
    setSuggestions(null);
  };

  const insertInlineMath = () => {
    if (!editor) {
      return;
    }

    const latex = globalThis.window.prompt("Fórmula LaTeX (inline)", "x^2");
    if (!latex?.trim()) {
      return;
    }

    editor
      .chain()
      .focus()
      .insertContent({
        type: "inlineMath",
        attrs: {
          latex: latex.trim(),
          ascii: latexToAscii(latex.trim())
        }
      })
      .run();
  };

  const editLink = () => {
    if (!editor) {
      return;
    }

    const previous = editor.getAttributes("link").href;
    const href = globalThis.window.prompt("URL del enlace", previous ?? "https://");
    if (href === null) {
      return;
    }

    if (!href.trim()) {
      editor.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }

    editor
      .chain()
      .focus()
      .extendMarkRange("link")
      .setLink({ href: href.trim() })
      .run();
  };

  const insertTable = () => {
    editor
      ?.chain()
      .focus()
      .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
      .run();
  };

  const currentColor = toHexColor(
    editor?.getAttributes("textStyle").color
  );
  const activeFontFamily = String(
    editor?.getAttributes("textStyle").fontFamily ?? ""
  );
  const fontFamilies = [
    { value: "", label: t("text.fontDefault") },
    { value: "Arial, Helvetica, sans-serif", label: t("text.fontSans") },
    {
      value: "Georgia, 'Times New Roman', serif",
      label: t("text.fontSerif")
    },
    {
      value: "'Cascadia Mono', Consolas, monospace",
      label: t("text.fontMono")
    },
    { value: "Arial, sans-serif", label: "Arial" },
    { value: "'Times New Roman', serif", label: "Times New Roman" },
    { value: "'Courier New', monospace", label: "Courier New" },
    { value: "Verdana, sans-serif", label: "Verdana" },
    { value: "'Trebuchet MS', sans-serif", label: "Trebuchet MS" }
  ];
  if (
    activeFontFamily &&
    !fontFamilies.some((font) => font.value === activeFontFamily)
  ) {
    fontFamilies.push({ value: activeFontFamily, label: activeFontFamily });
  }
  const applyFontFamily = (value: string) => {
    if (!editor) {
      return;
    }

    if (value === "") {
      editor.chain().focus().unsetFontFamily().run();
    } else {
      editor.chain().focus().setFontFamily(value).run();
    }
  };
  const activeFontSize =
    parseFontSize(editor?.getAttributes("textStyle").fontSize) ?? "default";
  const fontSizes = ["12", "14", "16", "18", "20", "24", "28", "32"];
  if (activeFontSize !== "default" && !fontSizes.includes(activeFontSize)) {
    fontSizes.push(activeFontSize);
    fontSizes.sort((left, right) => Number(left) - Number(right));
  }

  const applyFontSize = (value: string) => {
    if (!editor) {
      return;
    }

    if (value === "default") {
      editor.chain().focus().unsetFontSize().run();
    } else {
      editor.chain().focus().setFontSize(`${value}px`).run();
    }
  };

  const button = (
    label: string,
    title: string,
    active: boolean,
    action: () => void
  ) => (
    <button
      type="button"
      title={title}
      className={active ? "active" : ""}
      onMouseDown={(event) => event.preventDefault()}
      onClick={action}
    >
      {label}
    </button>
  );

  return (
    <div className="text-region">
      {selected && (
        <div className="text-toolbar" contentEditable={false}>
          {button("B", t("text.bold"), Boolean(editor?.isActive("bold")), () =>
            editor?.chain().focus().toggleBold().run()
          )}
        {button("I", t("text.italic"), Boolean(editor?.isActive("italic")), () =>
          editor?.chain().focus().toggleItalic().run()
        )}
        {button(
          "U",
          t("text.underline"),
          Boolean(editor?.isActive("underline")),
          () => editor?.chain().focus().toggleUnderline().run()
        )}
        {button(
          "H2",
          t("text.heading"),
          Boolean(editor?.isActive("heading", { level: 2 })),
          () => editor?.chain().focus().toggleHeading({ level: 2 }).run()
        )}
        {button(
          "•",
          t("text.bulletList"),
          Boolean(editor?.isActive("bulletList")),
          () => editor?.chain().focus().toggleBulletList().run()
        )}
        {button(
          "1.",
          t("text.numberedList"),
          Boolean(editor?.isActive("orderedList")),
          () => editor?.chain().focus().toggleOrderedList().run()
        )}
        {button("🔗", t("text.link"), Boolean(editor?.isActive("link")), editLink)}
        {button("▦", t("text.table"), Boolean(editor?.isActive("table")), insertTable)}
        {editor?.isActive("table") && (
          <>
            {button("+F", "Añadir fila debajo", false, () =>
              editor.chain().focus().addRowAfter().run()
            )}
            {button("+C", "Añadir columna a la derecha", false, () =>
              editor.chain().focus().addColumnAfter().run()
            )}
            {button("−F", "Eliminar fila", false, () =>
              editor.chain().focus().deleteRow().run()
            )}
            {button("−C", "Eliminar columna", false, () =>
              editor.chain().focus().deleteColumn().run()
            )}
            {button("×T", "Eliminar tabla", false, () =>
              editor.chain().focus().deleteTable().run()
            )}
          </>
        )}
        <select
          className="font-family-select"
          title={t("text.font")}
          value={activeFontFamily}
          onChange={(event) => applyFontFamily(event.target.value)}
        >
          {fontFamilies.map((font) => (
            <option key={font.value || "default"} value={font.value}>
              {font.label}
            </option>
          ))}
        </select>
        <select
          className="spellcheck-language-select"
          title={t("text.spelling")}
          value={spellLanguage}
          onChange={(event) => onLanguageChange(event.target.value)}
        >
          <option value="none">{t("text.noSpellcheck")}</option>
          {SPELLCHECK_LANGUAGES.map((language) => (
            <option key={language.id} value={language.id}>
              {language.native}
            </option>
          ))}
        </select>
        <select
          className="font-size-select"
          title={t("text.fontSize")}
          value={activeFontSize}
          onChange={(event) => applyFontSize(event.target.value)}
        >
          <option value="default">{t("text.autoSize")}</option>
          {fontSizes.map((size) => (
            <option key={size} value={size}>
              {size}
            </option>
          ))}
        </select>
        <input
          className="text-color-input"
          type="color"
          title={t("text.color")}
          value={currentColor}
          onChange={(event) =>
            editor?.chain().focus().setColor(event.target.value).run()
          }
        />
        {button("⌫", t("text.clearColor"), false, () =>
          editor?.chain().focus().unsetColor().run()
        )}
        {button("ƒx", t("text.inlineMath"), false, insertInlineMath)}
        </div>
      )}
      <EditorContent editor={editor} />
      {suggestions &&
        createPortal(
          <div
            className="spellcheck-menu"
            contentEditable={false}
            style={{ left: suggestions.x, top: suggestions.y }}
          >
            <div className="spellcheck-menu-title">{suggestions.word}</div>
            {suggestions.items.length > 0 ? (
              suggestions.items.map((item) => (
                <button
                  key={item}
                  type="button"
                  onMouseDown={(event) => event.preventDefault()}
                  onClick={() => replaceSuggestion(item)}
                >
                  {item}
                </button>
              ))
            ) : (
              <span className="spellcheck-menu-empty">
                {t("text.noSuggestions")}
              </span>
            )}
            <div className="spellcheck-menu-divider" />
            <button
              type="button"
              className="spellcheck-menu-action"
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => ignoreWordOnce(suggestions.word)}
            >
              {t("text.ignoreOnce")}
            </button>
            <button
              type="button"
              className="spellcheck-menu-action"
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => addWordToDictionary(suggestions.word)}
            >
              {t("text.addDictionary")}
            </button>
          </div>,
          document.body
        )}
    </div>
  );
}

function parseFontSize(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }

  const match = value.trim().match(/^(\d+(?:\.\d+)?)(px|pt)$/i);
  if (!match) {
    return null;
  }

  const amount = Number(match[1]);
  if (!Number.isFinite(amount)) {
    return null;
  }

  const pixels = match[2].toLowerCase() === "pt" ? (amount * 96) / 72 : amount;
  return String(Math.round(pixels));
}

function toHexColor(value: unknown): string {
  if (typeof value !== "string") {
    return "#1f2937";
  }

  const color = value.trim().toLowerCase();
  if (/^#[0-9a-f]{6}$/.test(color)) {
    return color;
  }
  if (/^#[0-9a-f]{3}$/.test(color)) {
    return `#${color[1]}${color[1]}${color[2]}${color[2]}${color[3]}${color[3]}`;
  }

  const rgb = color.match(
    /^rgba?\(\s*(\d{1,3})\s*[, ]\s*(\d{1,3})\s*[, ]\s*(\d{1,3})/
  );
  if (rgb) {
    return `#${[rgb[1], rgb[2], rgb[3]]
      .map((channel) =>
        Math.min(255, Number(channel)).toString(16).padStart(2, "0")
      )
      .join("")}`;
  }

  const named: Record<string, string> = {
    black: "#000000",
    white: "#ffffff",
    red: "#ff0000",
    green: "#008000",
    blue: "#0000ff",
    yellow: "#ffff00",
    orange: "#ffa500",
    purple: "#800080",
    gray: "#808080",
    grey: "#808080"
  };

  return named[color] ?? "#1f2937";
}
