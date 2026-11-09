import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  DEFAULT_NUMBER_FORMAT,
  FRACTION_MODES,
  NUMBER_NOTATIONS,
  formatPreviewNumber,
  formatPreviewRational,
  normalizeNumberFormat,
  type NumberFormatSettings
} from "../lib/numberFormat";

type Props = {
  notation: string;
  precision: number;
  fractions: string;
  trimZeros: boolean;
  onApply: (format: NumberFormatSettings) => void;
  onClose: () => void;
};

export function NumberFormatDialog({
  notation,
  precision,
  fractions,
  trimZeros,
  onApply,
  onClose
}: Props) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<NumberFormatSettings>(() =>
    normalizeNumberFormat({ notation, precision, fractions, trimZeros })
  );

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

  const previews = [
    formatPreviewNumber(6627.6535433071, draft),
    formatPreviewNumber(0.000000009771908, draft),
    formatPreviewNumber(1234567890123, draft),
    formatPreviewRational(1, 3, draft)
  ];

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
        className="dialog number-format-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="number-format-title"
        onSubmit={(event) => {
          event.preventDefault();
          onApply(normalizeNumberFormat(draft));
        }}
      >
        <h2 id="number-format-title">{t("dialog.numbers.title")}</h2>
        <p className="dialog-hint">
          Define cómo se muestran los resultados. El cálculo interno conserva
          toda la precisión; este formato solo afecta a la presentación y a las
          exportaciones.
        </p>

        <div className="number-format-fields">
          <label className="unit-settings-field">
            <span>Notación</span>
            <select
              value={draft.notation}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  notation: event.target.value as NumberFormatSettings["notation"]
                }))
              }
            >
              {NUMBER_NOTATIONS.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>

          <label className="unit-settings-field">
            <span>Decimales</span>
            <input
              type="number"
              min={0}
              max={12}
              value={draft.precision}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  precision: Math.max(
                    0,
                    Math.min(12, Number(event.target.value) || 0)
                  )
                }))
              }
            />
          </label>

          <label className="unit-settings-field">
            <span>Fracciones</span>
            <select
              value={draft.fractions}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  fractions: event.target.value as "auto" | "decimal"
                }))
              }
            >
              {FRACTION_MODES.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
        </div>

        <label className="view-option">
          <input
            type="checkbox"
            checked={draft.trimZeros}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                trimZeros: event.target.checked
              }))
            }
          />
          Quitar ceros finales innecesarios
        </label>

        <div className="number-format-preview">
          <span className="number-format-preview-title">Vista previa</span>
          <div className="number-format-preview-grid">
            {["6627.6535433071", "9.771908e-9", "1234567890123", "1/3"].map(
              (source, index) => (
                <div key={source} className="number-format-preview-row">
                  <code>{source}</code>
                  <strong>{previews[index]}</strong>
                </div>
              )
            )}
          </div>
        </div>

        <div className="dialog-actions">
          <button
            type="button"
            className="secondary"
            onClick={() => setDraft({ ...DEFAULT_NUMBER_FORMAT })}
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
