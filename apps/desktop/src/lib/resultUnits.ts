export type ResultUnitSystem = "metric" | "imperial" | "custom" | "original";

export const RESULT_UNIT_SYSTEMS: Array<{
  id: ResultUnitSystem;
  label: string;
}> = [
  { id: "metric", label: "Métrico (SI)" },
  { id: "imperial", label: "Imperial" },
  { id: "custom", label: "Personalizado" },
  { id: "original", label: "Mantener unidades originales" }
];

export const METRIC_UNIT_DEFAULTS: Record<string, string> = {
  length: "m",
  mass: "kg",
  time: "s",
  temperature: "°C",
  force: "N",
  pressure: "Pa",
  energy: "J",
  power: "W",
  volume: "m³",
  area: "m²"
};

export const IMPERIAL_UNIT_DEFAULTS: Record<string, string> = {
  length: "ft",
  mass: "lb",
  time: "s",
  temperature: "°F",
  force: "lbf",
  pressure: "psi",
  energy: "ft·lb",
  power: "hp",
  volume: "ft³",
  area: "ft²"
};

export type ResultUnitGroup = {
  key: string;
  label: string;
  units: string[];
};

export const RESULT_UNIT_GROUPS: ResultUnitGroup[] = [
  {
    key: "length",
    label: "Longitud",
    units: ["m", "cm", "mm", "km", "in", "ft", "yd", "mi"]
  },
  {
    key: "mass",
    label: "Masa",
    units: ["kg", "g", "mg", "lb", "oz"]
  },
  {
    key: "time",
    label: "Tiempo",
    units: ["s", "min", "h", "d"]
  },
  {
    key: "temperature",
    label: "Temperatura",
    units: ["°C", "K", "°F"]
  },
  {
    key: "force",
    label: "Fuerza",
    units: ["N", "lbf"]
  },
  {
    key: "pressure",
    label: "Presión",
    units: ["Pa", "bar", "atm", "psi"]
  },
  {
    key: "energy",
    label: "Energía",
    units: ["J", "kWh", "ft·lb"]
  },
  {
    key: "power",
    label: "Potencia",
    units: ["W", "hp"]
  },
  {
    key: "volume",
    label: "Volumen",
    units: ["m³", "L", "mL", "ft³", "gal"]
  },
  {
    key: "area",
    label: "Área",
    units: ["m²", "ft²", "in²"]
  }
];

export function resultUnitDefaults(
  system: ResultUnitSystem
): Record<string, string> {
  if (system === "imperial") {
    return { ...IMPERIAL_UNIT_DEFAULTS };
  }
  return { ...METRIC_UNIT_DEFAULTS };
}

export function resultUnitSystemLabel(system: string): string {
  return (
    RESULT_UNIT_SYSTEMS.find((item) => item.id === system)?.label ??
    "Métrico (SI)"
  );
}
