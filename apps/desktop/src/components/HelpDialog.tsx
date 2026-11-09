import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  getManualContent,
  searchManual,
  type ManualBlock
} from "../lib/manual";

type Props = {
  language: string;
  initialSection?: string;
  onClose: () => void;
};

export function HelpDialog({
  language,
  initialSection = "inicio",
  onClose
}: Props) {
  const { t } = useTranslation();
  const content = useMemo(() => getManualContent(language), [language]);
  const [query, setQuery] = useState("");
  const results = useMemo(
    () => searchManual(content, query),
    [content, query]
  );
  const [activeId, setActiveId] = useState(() =>
    content.sections.some((section) => section.id === initialSection)
      ? initialSection
      : content.sections[0]?.id ?? ""
  );
  const searchRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    searchRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  const activeSection =
    results.find((section) => section.id === activeId) ?? results[0] ?? null;

  useEffect(() => {
    if (results.length > 0 && !results.some((section) => section.id === activeId)) {
      setActiveId(results[0].id);
    }
  }, [activeId, results]);

  return (
    <div
      className="dialog-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        className="dialog help-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="help-dialog-title"
      >
        <aside className="help-sidebar">
          <div className="help-brand">
            <span className="help-brand-mark" aria-hidden="true">IC</span>
            <div>
              <strong>IngeCalc</strong>
              <span>{t("help.subtitle")}</span>
            </div>
          </div>

          <label className="help-search">
            <span className="help-search-icon" aria-hidden="true">⌕</span>
            <input
              ref={searchRef}
              type="search"
              value={query}
              placeholder={t("help.search")}
              aria-label={t("help.search")}
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>

          <div className="help-contents-label">{t("help.contents")}</div>
          <nav className="help-nav" aria-label={t("help.contents")}>
            {results.map((section) => (
              <button
                key={section.id}
                type="button"
                className={section.id === activeSection?.id ? "active" : ""}
                aria-current={section.id === activeSection?.id ? "page" : undefined}
                onClick={() => {
                  setActiveId(section.id);
                  setQuery("");
                }}
              >
                <span className="help-nav-icon" aria-hidden="true">
                  {section.icon}
                </span>
                <span>{section.title}</span>
              </button>
            ))}
            {results.length === 0 && (
              <p className="help-empty">{t("help.noResults")}</p>
            )}
          </nav>

          <div className="help-sidebar-footer">
            v{content.version} · GPL-3.0-or-later
          </div>
        </aside>

        <main className="help-main">
          <header className="help-main-header">
            <div>
              <p className="help-eyebrow">{t("help.title")}</p>
              <h2 id="help-dialog-title">
                {activeSection?.title ?? content.title}
              </h2>
            </div>
            <button
              type="button"
              className="help-close"
              title={t("help.close")}
              aria-label={t("help.close")}
              onClick={onClose}
            >
              ×
            </button>
          </header>

          {activeSection && (
            <>
              <p className="help-summary">{activeSection.summary}</p>
              <div className="help-content">
                {activeSection.blocks.map((block, index) =>
                  renderBlock(block, `${activeSection.id}-${index}`)
                )}
              </div>
            </>
          )}
        </main>
      </div>
    </div>
  );
}

function renderBlock(block: ManualBlock, key: string) {
  switch (block.type) {
    case "paragraph":
      return (
        <p key={key}>
          <InlineText text={block.text} />
        </p>
      );
    case "list":
      return (
        <ul key={key}>
          {block.items.map((item, index) => (
            <li key={`${key}-${index}`}>
              <InlineText text={item} />
            </li>
          ))}
        </ul>
      );
    case "code":
      return (
        <div className="help-code" key={key}>
          {block.title && <div className="help-code-title">{block.title}</div>}
          <pre>
            <code>{block.text}</code>
          </pre>
        </div>
      );
    case "table":
      return (
        <div className="help-table-wrap" key={key}>
          <table>
            <thead>
              <tr>
                {block.headers.map((header) => (
                  <th key={header}>
                    <InlineText text={header} />
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, rowIndex) => (
                <tr key={`${key}-${rowIndex}`}>
                  {row.map((cell, cellIndex) => (
                    <td key={`${key}-${rowIndex}-${cellIndex}`}>
                      <InlineText text={cell} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "note":
      return (
        <div className="help-note" key={key}>
          {block.title && <strong>{block.title}</strong>}
          <p>
            <InlineText text={block.text} />
          </p>
        </div>
      );
  }
}

function InlineText({ text }: { text: string }) {
  const parts = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g).filter(Boolean);
  return (
    <>
      {parts.map((part, index) => {
        if (part.startsWith("**") && part.endsWith("**")) {
          return <strong key={index}>{part.slice(2, -2)}</strong>;
        }
        if (part.startsWith("`") && part.endsWith("`")) {
          return <code key={index}>{part.slice(1, -1)}</code>;
        }
        return <span key={index}>{part}</span>;
      })}
    </>
  );
}
