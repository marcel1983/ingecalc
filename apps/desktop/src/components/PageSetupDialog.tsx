import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { DEFAULT_PAGE_SETUP } from "../lib/pageSetup";
import type { PageSetup } from "../types/document";
import { UI_LANGUAGES } from "../lib/i18n";

type Props = {
  setup: PageSetup;
  language: string;
  onApply: (setup: PageSetup, language: string) => void;
  onClose: () => void;
};

export function PageSetupDialog({
  setup,
  language,
  onApply,
  onClose
}: Props) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<PageSetup>({ ...setup });
  const [draftLanguage, setDraftLanguage] = useState(language);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  const updateNumber = (
    field: "marginTop" | "marginRight" | "marginBottom" | "marginLeft",
    value: string
  ) => {
    const parsed = Number(value);
    setDraft((current) => ({
      ...current,
      [field]: Number.isFinite(parsed)
        ? Math.max(0, Math.min(300, Math.round(parsed)))
        : 0
    }));
  };

  return (
    <div
      className="dialog-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <form
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="page-setup-title"
        onSubmit={(event) => {
          event.preventDefault();
          onApply(draft, draftLanguage);
        }}
      >
        <h2 id="page-setup-title">{t("dialog.page.title")}</h2>
        <p className="dialog-hint">
          {t("dialog.page.hint", {
            page: "{page}"
          })}
        </p>

        <fieldset>
          <legend>{t("dialog.page.margins")}</legend>
          <div className="dialog-grid">
            <label>
              {t("dialog.page.top")}
              <input
                type="number"
                min={0}
                max={300}
                value={draft.marginTop}
                onChange={(event) =>
                  updateNumber("marginTop", event.target.value)
                }
              />
            </label>
            <label>
              {t("dialog.page.right")}
              <input
                type="number"
                min={0}
                max={300}
                value={draft.marginRight}
                onChange={(event) =>
                  updateNumber("marginRight", event.target.value)
                }
              />
            </label>
            <label>
              {t("dialog.page.bottom")}
              <input
                type="number"
                min={0}
                max={300}
                value={draft.marginBottom}
                onChange={(event) =>
                  updateNumber("marginBottom", event.target.value)
                }
              />
            </label>
            <label>
              {t("dialog.page.left")}
              <input
                type="number"
                min={0}
                max={300}
                value={draft.marginLeft}
                onChange={(event) =>
                  updateNumber("marginLeft", event.target.value)
                }
              />
            </label>
          </div>
        </fieldset>

        <label className="dialog-field">
          {t("dialog.page.header")}
          <input
            type="text"
            value={draft.header}
            placeholder={t("dialog.page.header")}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                header: event.target.value
              }))
            }
          />
        </label>

        <label className="dialog-field">
          {t("dialog.page.footer")}
          <input
            type="text"
            value={draft.footer}
            placeholder={t("dialog.page.footer")}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                footer: event.target.value
              }))
            }
          />
        </label>

        <label className="dialog-field">
          {t("dialog.page.language")}
          <select
            value={draftLanguage}
            onChange={(event) => setDraftLanguage(event.target.value)}
          >
            {UI_LANGUAGES.map((item) => (
              <option key={item.id} value={item.id}>
                {item.native}
              </option>
            ))}
            <option value="none">{t("text.noSpellcheck")}</option>
          </select>
        </label>
        <p className="dialog-hint">{t("dialog.page.languageHint")}</p>

        <div className="dialog-actions">
          <button
            type="button"
            className="secondary"
            onClick={() => setDraft({ ...DEFAULT_PAGE_SETUP })}
          >
            {t("dialog.actions.reset")}
          </button>
          <button type="button" className="secondary" onClick={onClose}>
            {t("dialog.actions.cancel")}
          </button>
          <button type="submit">{t("dialog.actions.apply")}</button>
        </div>
      </form>
    </div>
  );
}
