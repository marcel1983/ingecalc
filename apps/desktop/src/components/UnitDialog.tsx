import { useEffect, useState } from "react";

type Props = {
  onApply: (name: string, definition: string) => void;
  onClose: () => void;
};

const UNIT_NAME = /^[\p{L}\p{N}_]+$/u;

export function UnitDialog({ onApply, onClose }: Props) {
  const [name, setName] = useState("mi_unidad");
  const [definition, setDefinition] = useState("2.54 cm");
  const [error, setError] = useState("");

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
        className="dialog unit-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="unit-title"
        onSubmit={(event) => {
          event.preventDefault();
          const trimmedName = name.trim();
          const trimmedDefinition = definition.trim();

          if (!UNIT_NAME.test(trimmedName)) {
            setError("El nombre solo puede contener letras, números y guiones bajos.");
            return;
          }
          if (!trimmedDefinition) {
            setError("La definición no puede estar vacía.");
            return;
          }

          onApply(trimmedName, trimmedDefinition);
        }}
      >
        <h2 id="unit-title">Nueva unidad</h2>
        <p className="dialog-hint">
          La definición debe ser una cantidad, por ejemplo <code>2.54 cm</code> o{" "}
          <code>12 in</code>. Después podrás insertarla desde la paleta de
          unidades.
        </p>

        <label className="dialog-field">
          Nombre
          <input
            type="text"
            autoFocus
            value={name}
            onChange={(event) => {
              setName(event.target.value);
              setError("");
            }}
          />
        </label>

        <label className="dialog-field">
          Definición
          <input
            type="text"
            value={definition}
            onChange={(event) => {
              setDefinition(event.target.value);
              setError("");
            }}
          />
        </label>

        {error && <p className="dialog-error">{error}</p>}

        <div className="dialog-actions">
          <button type="button" className="secondary" onClick={onClose}>
            Cancelar
          </button>
          <button type="submit">Crear unidad</button>
        </div>
      </form>
    </div>
  );
}
