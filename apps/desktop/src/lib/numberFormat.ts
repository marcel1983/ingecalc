export type NumberNotation = "auto" | "fixed" | "scientific" | "engineering";

export type NumberFormatSettings = {
  notation: NumberNotation;
  precision: number;
  fractions: "auto" | "decimal";
  trimZeros: boolean;
};

export const DEFAULT_NUMBER_FORMAT: NumberFormatSettings = {
  notation: "auto",
  precision: 4,
  fractions: "auto",
  trimZeros: true
};

export const NUMBER_NOTATIONS: Array<{
  id: NumberNotation;
  label: string;
}> = [
  { id: "auto", label: "Automática" },
  { id: "fixed", label: "Decimal fija" },
  { id: "scientific", label: "Científica" },
  { id: "engineering", label: "Ingeniería" }
];

export const FRACTION_MODES = [
  { id: "auto", label: "Automáticas (fracciones exactas)" },
  { id: "decimal", label: "Siempre decimal" }
] as const;

export function normalizeNumberFormat(settings: {
  notation?: string;
  precision?: number;
  fractions?: string;
  trimZeros?: boolean;
}): NumberFormatSettings {
  const notation = NUMBER_NOTATIONS.some(
    (item) => item.id === settings.notation
  )
    ? (settings.notation as NumberNotation)
    : DEFAULT_NUMBER_FORMAT.notation;
  const precision = Number.isFinite(settings.precision)
    ? Math.max(0, Math.min(12, Math.round(settings.precision ?? 4)))
    : DEFAULT_NUMBER_FORMAT.precision;
  const fractions =
    settings.fractions === "decimal" ? "decimal" : "auto";
  return {
    notation,
    precision,
    fractions,
    trimZeros: settings.trimZeros ?? true
  };
}

function trimFixed(text: string): string {
  if (!text.includes(".")) {
    return text === "-0" ? "0" : text;
  }
  const trimmed = text.replace(/0+$/, "").replace(/\.$/, "");
  return trimmed === "-0" ? "0" : trimmed;
}

function formatFixed(
  value: number,
  decimals: number,
  trimZeros: boolean
): string {
  const text = value.toFixed(decimals);
  return trimZeros ? trimFixed(text) : text;
}

function formatScientific(
  value: number,
  decimals: number,
  trimZeros: boolean
): string {
  const exponent = Math.floor(Math.log10(Math.abs(value)));
  const mantissa = value / 10 ** exponent;
  const text = formatFixed(mantissa, decimals, trimZeros);
  return `${text}e${exponent}`;
}

function formatEngineering(
  value: number,
  decimals: number,
  trimZeros: boolean
): string {
  if (value === 0) {
    return formatFixed(value, decimals, trimZeros);
  }
  const exponent = Math.floor(Math.log10(Math.abs(value)) / 3) * 3;
  const mantissa = value / 10 ** exponent;
  const text = formatFixed(mantissa, decimals, trimZeros);
  return exponent === 0 ? text : `${text}e${exponent}`;
}

export function formatPreviewNumber(
  value: number,
  format: NumberFormatSettings
): string {
  if (!Number.isFinite(value)) {
    return String(value);
  }
  const decimals = Math.max(0, Math.min(12, Math.round(format.precision)));
  if (value === 0) {
    return format.trimZeros ? "0" : value.toFixed(decimals);
  }

  switch (format.notation) {
    case "fixed":
      return formatFixed(value, decimals, format.trimZeros);
    case "scientific":
      return formatScientific(value, decimals, format.trimZeros);
    case "engineering":
      return formatEngineering(value, decimals, format.trimZeros);
    default: {
      const threshold = Math.max(10 ** -decimals, 1e-6);
      return Math.abs(value) < threshold || Math.abs(value) >= 1e12
        ? formatScientific(value, decimals, format.trimZeros)
        : formatFixed(value, decimals, format.trimZeros);
    }
  }
}

export function formatPreviewRational(
  numerator: number,
  denominator: number,
  format: NumberFormatSettings
): string {
  if (format.fractions === "decimal" || denominator === 1) {
    return formatPreviewNumber(numerator / denominator, format);
  }
  return `${numerator}/${denominator}`;
}
