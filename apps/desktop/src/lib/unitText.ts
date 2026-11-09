import { PALETTES } from "./palettes";

const unitAliases = [
  "metro",
  "metros",
  "pulgada",
  "pulgadas",
  "pie",
  "pies",
  "libra",
  "libras",
  "grado Celsius",
  "grados Celsius",
  "grado Fahrenheit",
  "grados Fahrenheit",
  "libra fuerza",
  "caballo de fuerza"
];

export const UNIT_SYMBOLS = new Set<string>([
  ...PALETTES.unit.map((item) => item.label),
  ...unitAliases
]);

export const UNIT_TOKEN_CLASS = "ingecalc-unit-token";

const sortedUnitSymbols = [...UNIT_SYMBOLS].sort(
  (left, right) => right.length - left.length
);

export function isUnitRun(text: string): boolean {
  return UNIT_SYMBOLS.has(text);
}

const SUPERSCRIPTS: Record<string, string> = {
  "0": "⁰",
  "1": "¹",
  "2": "²",
  "3": "³",
  "4": "⁴",
  "5": "⁵",
  "6": "⁶",
  "7": "⁷",
  "8": "⁸",
  "9": "⁹",
  "-": "⁻"
};

export function formatUnitDisplay(text: string): string {
  return text.replace(/\^(-?\d+)/g, (_, digits: string) =>
    [...digits].map((digit) => SUPERSCRIPTS[digit] ?? digit).join("")
  );
}

export function nextStartsFunctionCall(
  nextText: string | null | undefined
): boolean {
  return Boolean(nextText?.trimStart().startsWith("("));
}

export type SplitResult = {
  value: string;
  unit: string;
};

export function splitResultUnit(display: string): SplitResult {
  const trimmed = display.trimEnd();
  if (!trimmed) {
    return { value: display, unit: "" };
  }

  for (const unit of sortedUnitSymbols) {
    if (!trimmed.endsWith(unit)) {
      continue;
    }

    const before = trimmed.slice(0, trimmed.length - unit.length);
    const separator = before.endsWith(" ");
    const compact = unit.startsWith("°") || unit.startsWith("%");
    if (!separator && !compact) {
      continue;
    }

    const value = before.trimEnd();
    if (!value || !/[0-9)\]}]$/.test(value)) {
      continue;
    }

    return {
      value,
      unit: `${separator ? " " : ""}${unit}`
    };
  }

  const generic = trimmed.match(
    /(?:^|\s)([A-Za-zµΩ°%][A-Za-zµΩ°%0-9²³·*/^]*)$/u
  );
  if (generic) {
    const unit = generic[1];
    const first = unit.match(/^[A-Za-zµΩ°%]+/u)?.[0] ?? "";
    const separator = trimmed
      .slice(0, trimmed.length - unit.length)
      .endsWith(" ");
    const value = trimmed.slice(0, trimmed.length - unit.length).trimEnd();
    if (
      UNIT_SYMBOLS.has(first) &&
      value &&
      /[0-9)\]}]$/.test(value)
    ) {
      return {
        value,
        unit: `${separator ? " " : ""}${unit}`
      };
    }
  }

  return { value: display, unit: "" };
}
