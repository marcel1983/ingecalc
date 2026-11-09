import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CustomConstant } from "../types/document";
import { ENGINEERING_CONSTANTS } from "../lib/engineeringConstants";
import { isUnitRun } from "../lib/unitText";

type Props = {
  constants: CustomConstant[];
  onApply: (constants: CustomConstant[]) => void;
  onInsert: (name: string) => void;
  onClose: () => void;
};

const RESERVED = new Set(["pi", "e", "i", "unit"]);
const NAME_PATTERN = /^[A-Za-z]+$/;

export function ConstantsDialog({
  constants,
  onApply,
  onInsert,
  onClose
}: Props) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<CustomConstant[]>(() =>
    constants.map((constant) => ({ ...constant }))
  );
  const [name, setName] = useState("");
  const [expression, setExpression] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState("");
  const [editing, setEditing] = useState<string | null>(null);

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

  const resetForm = () => {
    setName("");
    setExpression("");
    setDescription("");
    setEditing(null);
    setError("");
  };

  const saveConstant = () => {
    const cleanName = name.trim();
    const cleanExpression = expression.trim();
    if (!NAME_PATTERN.test(cleanName)) {
      setError("El nombre debe contener solo letras.");
      return;
    }
    if (RESERVED.has(cleanName) || isUnitRun(cleanName)) {
      setError("Ese nombre está reservado por una constante o unidad del sistema.");
      return;
    }
    if (!cleanExpression) {
      setError("Escribe el valor o la expresión de la constante.");
      return;
    }
    if (
      ENGINEERING_CONSTANTS.some((constant) => constant.name === cleanName) ||
      draft.some(
        (constant) =>
          constant.name === cleanName && constant.name !== editing
      )
    ) {
      setError("Ya existe una constante con ese nombre.");
      return;
    }

    const next = {
      name: cleanName,
      expression: cleanExpression,
      description: description.trim()
    };
    setDraft((current) =>
      editing
        ? current.map((constant) =>
            constant.name === editing ? next : constant
          )
        : [...current, next]
    );
    resetForm();
  };

  const editConstant = (constant: CustomConstant) => {
    setName(constant.name);
    setExpression(constant.expression);
    setDescription(constant.description ?? "");
    setEditing(constant.name);
    setError("");
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
        className="dialog constants-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="constants-title"
        onSubmit={(event) => {
          event.preventDefault();
          onApply(draft);
        }}
      >
        <h2 id="constants-title">{t("dialog.constants.title")}</h2>
        <p className="dialog-hint">{t("dialog.constants.hint")}</p>

        <div className="constants-builtins">
          <div className="constants-section-title">
            {t("dialog.constants.builtins")}
          </div>
          <div className="constants-list">
            {ENGINEERING_CONSTANTS.map((constant) => (
              <button
                key={constant.name}
                type="button"
                className="constants-entry"
                title={`${constant.description}${constant.unit ? ` · ${constant.unit}` : ""}`}
                onClick={() => onInsert(constant.name)}
              >
                <strong>{constant.name}</strong>
                <span>{constant.description}</span>
                <em>{constant.unit || "—"}</em>
              </button>
            ))}
          </div>
        </div>

        <div className="constants-section-title">
          {t("dialog.constants.custom")}
        </div>
        {draft.length === 0 ? (
          <p className="dialog-hint">{t("dialog.constants.empty")}</p>
        ) : (
          <div className="constants-list custom">
            {draft.map((constant) => (
              <div key={constant.name} className="constants-entry custom">
                <strong>{constant.name}</strong>
                <span>{constant.description || constant.expression}</span>
                <em>{constant.expression}</em>
                <div className="constants-entry-actions">
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => editConstant(constant)}
                  >
                    {t("dialog.constants.edit")}
                  </button>
                  <button
                    type="button"
                    className="secondary danger"
                    onClick={() =>
                      setDraft((current) =>
                        current.filter(
                          (item) => item.name !== constant.name
                        )
                      )
                    }
                  >
                    {t("dialog.constants.remove")}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        <fieldset className="constants-form">
          <legend>
            {editing
              ? t("dialog.constants.editTitle")
              : t("dialog.constants.addTitle")}
          </legend>
          <div className="constants-form-grid">
            <label>
              {t("dialog.constants.name")}
              <input
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="rhoacero"
              />
            </label>
            <label>
              {t("dialog.constants.value")}
              <input
                value={expression}
                onChange={(event) => setExpression(event.target.value)}
                placeholder="7850 kg/m^3"
              />
            </label>
          </div>
          <label>
            {t("dialog.constants.description")}
            <input
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder={t("dialog.constants.descriptionPlaceholder")}
            />
          </label>
          <div className="constants-form-actions">
            <button type="button" className="secondary" onClick={saveConstant}>
              {editing
                ? t("dialog.constants.save")
                : t("dialog.constants.add")}
            </button>
            {editing && (
              <button type="button" className="secondary" onClick={resetForm}>
                {t("dialog.actions.cancel")}
              </button>
            )}
          </div>
        </fieldset>

        {error && <p className="dialog-error">{error}</p>}

        <div className="dialog-actions">
          <button type="button" className="secondary" onClick={onClose}>
            {t("dialog.actions.cancel")}
          </button>
          <button type="submit">{t("dialog.actions.apply")}</button>
        </div>
      </form>
    </div>
  );
}
