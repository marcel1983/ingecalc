import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { DEFAULT_VIEW_OPTIONS, type ViewOptions } from "../lib/viewOptions";

type Props = {
  options: ViewOptions;
  onApply: (options: ViewOptions) => void;
  onClose: () => void;
};

export function ViewOptionsDialog({ options, onApply, onClose }: Props) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<ViewOptions>({ ...options });

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
        aria-labelledby="view-options-title"
        onSubmit={(event) => {
          event.preventDefault();
          onApply(draft);
        }}
      >
        <h2 id="view-options-title">{t("dialog.view.title")}</h2>
        <p className="dialog-hint">{t("dialog.view.hint")}</p>

        <label className="view-option">
          <input
            type="checkbox"
            checked={draft.showGrid}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                showGrid: event.target.checked
              }))
            }
          />
          {t("dialog.view.grid")}
        </label>

        <label className="view-option">
          <input
            type="checkbox"
            checked={draft.snapToGrid}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                snapToGrid: event.target.checked
              }))
            }
          />
          {t("dialog.view.snap")}
        </label>

        <label className="view-option">
          <input
            type="checkbox"
            checked={draft.lockPositions}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                lockPositions: event.target.checked
              }))
            }
          />
          {t("dialog.view.lock")}
        </label>

        <div className="dialog-actions">
          <button
            type="button"
            className="secondary"
            onClick={() => setDraft({ ...DEFAULT_VIEW_OPTIONS })}
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
