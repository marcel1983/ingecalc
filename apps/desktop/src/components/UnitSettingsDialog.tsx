import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  RESULT_UNIT_GROUPS,
  RESULT_UNIT_SYSTEMS,
  resultUnitDefaults,
  type ResultUnitSystem
} from "../lib/resultUnits";

type Props = {
  units: string;
  overrides: Record<string, string>;
  onApply: (system: ResultUnitSystem, overrides: Record<string, string>) => void;
  onClose: () => void;
};

function normalizedSystem(units: string): ResultUnitSystem {
  return RESULT_UNIT_SYSTEMS.some((item) => item.id === units)
    ? (units as ResultUnitSystem)
    : "metric";
}

export function UnitSettingsDialog({
  units,
  overrides,
  onApply,
  onClose
}: Props) {
  const { t } = useTranslation();
  const initialSystem = normalizedSystem(units);
  const [system, setSystem] = useState<ResultUnitSystem>(initialSystem);
  const [values, setValues] = useState<Record<string, string>>(() => ({
    ...resultUnitDefaults(initialSystem === "imperial" ? "imperial" : "metric"),
    ...overrides
  }));

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

  const chooseSystem = (next: ResultUnitSystem) => {
    setSystem(next);
    if (next === "metric" || next === "imperial") {
      setValues({ ...resultUnitDefaults(next) });
    }
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
        className="dialog unit-settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="unit-settings-title"
        onSubmit={(event) => {
          event.preventDefault();
          onApply(
            system,
            system === "custom" ? { ...values } : {}
          );
        }}
      >
        <h2 id="unit-settings-title">{t("dialog.units.title")}</h2>
        <p className="dialog-hint">
          Elige el sistema en el que se muestran las respuestas con unidades.
          El cálculo interno siempre se hace en unidades SI; esta opción solo
          cambia la presentación del resultado.
        </p>

        <label className="unit-system-field">
          <span>Sistema de resultados</span>
          <select
            value={system}
            onChange={(event) =>
              chooseSystem(event.target.value as ResultUnitSystem)
            }
          >
            {RESULT_UNIT_SYSTEMS.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
        </label>

        {system === "custom" && (
          <>
            <p className="dialog-hint">
              Personaliza la unidad de cada magnitud. Las magnitudes derivadas
              se componen con estas unidades base cuando no tienen una unidad
              propia seleccionada.
            </p>
            <div className="unit-settings-grid">
              {RESULT_UNIT_GROUPS.map((group) => (
                <label key={group.key} className="unit-settings-field">
                  <span>{group.label}</span>
                  <select
                    value={values[group.key] ?? ""}
                    onChange={(event) =>
                      setValues((current) => ({
                        ...current,
                        [group.key]: event.target.value
                      }))
                    }
                  >
                    {group.units.map((unit) => (
                      <option key={unit} value={unit}>
                        {unit}
                      </option>
                    ))}
                  </select>
                </label>
              ))}
            </div>
          </>
        )}

        <div className="dialog-actions">
          <button
            type="button"
            className="secondary"
            onClick={() => {
              setSystem("metric");
              setValues({ ...resultUnitDefaults("metric") });
            }}
          >
            Restablecer métrico
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
