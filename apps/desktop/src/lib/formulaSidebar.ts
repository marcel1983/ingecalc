import {
  PALETTES,
  type PaletteItem,
  type PaletteKind
} from "./palettes";

export type FormulaSection = {
  id: string;
  title: string;
  items: PaletteItem[];
  morePalette?: PaletteKind;
};

export const ASSIGNMENT_ITEM: PaletteItem = {
  id: "assign",
  label: ":=",
  latex: ":=",
  sample: "x := 1",
  description: "Asignación o definición",
  category: "Asignación"
};

const variableItem: PaletteItem = {
  id: "assign-variable",
  label: "Variable",
  latex: "x := \\placeholder{}",
  sample: "x := 1",
  description: "Definir una variable",
  category: "Asignación"
};

const functionItem: PaletteItem = {
  id: "assign-function",
  label: "Función",
  latex: "f(x) := \\placeholder{}",
  sample: "f(x) := x^2",
  description: "Definir una función",
  category: "Asignación"
};

const basicItems: PaletteItem[] = [
  {
    id: "op-add",
    label: "+",
    latex: "+",
    sample: "1 + 2",
    description: "Suma",
    category: "Básico"
  },
  {
    id: "op-sub",
    label: "−",
    latex: "-",
    sample: "3 - 2",
    description: "Resta",
    category: "Básico"
  },
  {
    id: "op-mul",
    label: "×",
    latex: "\\times",
    sample: "2 \\times 3",
    description: "Multiplicación",
    category: "Básico"
  },
  {
    id: "op-div",
    label: "÷",
    latex: "\\div",
    sample: "6 \\div 2",
    description: "División",
    category: "Básico"
  },
  {
    id: "op-power",
    label: "xⁿ",
    latex: "\\placeholder{}^{\\placeholder{}}",
    sample: "2^3",
    description: "Potencia",
    category: "Básico"
  },
  {
    id: "op-paren",
    label: "( )",
    latex: "\\left(\\placeholder{}\\right)",
    sample: "\\left(1 + 2\\right)",
    description: "Paréntesis",
    category: "Básico"
  },
  {
    id: "op-abs",
    label: "|x|",
    latex: "\\left|\\placeholder{}\\right|",
    sample: "\\left|3\\right|",
    description: "Valor absoluto",
    category: "Básico"
  },
  {
    id: "op-fraction",
    label: "a/b",
    latex: "\\frac{\\placeholder{}}{\\placeholder{}}",
    sample: "\\frac{1}{2}",
    description: "Fracción",
    category: "Básico"
  },
  {
    id: "op-sqrt",
    label: "√",
    latex: "\\sqrt{\\placeholder{}}",
    sample: "\\sqrt{2}",
    description: "Raíz cuadrada",
    category: "Básico"
  },
  {
    id: "op-square",
    label: "x²",
    latex: "\\placeholder{}^{2}",
    sample: "x^2",
    description: "Cuadrado",
    category: "Básico"
  }
];

const supportedFunctions = new Set([
  "sin",
  "cos",
  "tan",
  "asin",
  "acos",
  "atan",
  "sinh",
  "cosh",
  "tanh",
  "asinh",
  "acosh",
  "atanh",
  "ln",
  "log",
  "exp",
  "sqrt",
  "abs",
  "min",
  "max",
  "floor",
  "ceil",
  "round",
  "trunc",
  "frac-part",
  "sign",
  "mod",
  "gcd",
  "lcm",
  "factorial",
  "binomial",
  "sum-elements",
  "mean"
]);

const symbolItems: PaletteItem[] = [
  {
    id: "symbol-pi",
    label: "π",
    latex: "\\pi",
    sample: "2 \\pi",
    description: "Número pi",
    category: "Símbolos"
  },
  {
    id: "symbol-e",
    label: "e",
    latex: "e",
    sample: "e",
    description: "Número de Euler",
    category: "Símbolos"
  },
  {
    id: "symbol-i",
    label: "i",
    latex: "i",
    sample: "2 + 3i",
    description: "Unidad imaginaria",
    category: "Símbolos"
  }
];

type GreekEntry = [label: string, latex: string, name: string];

function greekPalette(
  entries: GreekEntry[],
  prefix: string,
  category: string,
  suffix: string
): PaletteItem[] {
  return entries.map(([label, latex, name]) => ({
    id: `greek-${prefix}-${name.toLowerCase().replace(/\s+/g, "-")}`,
    label,
    latex,
    sample: latex,
    description: `${name} ${suffix}`,
    category
  }));
}

const lowercaseGreek = greekPalette(
  [
    ["α", "\\alpha", "Alfa"],
    ["β", "\\beta", "Beta"],
    ["γ", "\\gamma", "Gamma"],
    ["δ", "\\delta", "Delta"],
    ["ε", "\\epsilon", "Épsilon"],
    ["ζ", "\\zeta", "Dseta"],
    ["η", "\\eta", "Eta"],
    ["θ", "\\theta", "Theta"],
    ["ι", "\\iota", "Iota"],
    ["κ", "\\kappa", "Kappa"],
    ["λ", "\\lambda", "Lambda"],
    ["μ", "\\mu", "Mu"],
    ["ν", "\\nu", "Nu"],
    ["ξ", "\\xi", "Xi"],
    ["ο", "o", "Ómicron"],
    ["π", "\\pi", "Pi"],
    ["ρ", "\\rho", "Rho"],
    ["σ", "\\sigma", "Sigma"],
    ["τ", "\\tau", "Tau"],
    ["υ", "\\upsilon", "Ípsilon"],
    ["φ", "\\phi", "Fi"],
    ["χ", "\\chi", "Ji"],
    ["ψ", "\\psi", "Psi"],
    ["ω", "\\omega", "Omega"]
  ],
  "lower",
  "Griegas minúsculas",
  "minúscula"
);

const uppercaseGreek = greekPalette(
  [
    ["Α", "\\mathrm{A}", "Alfa"],
    ["Β", "\\mathrm{B}", "Beta"],
    ["Γ", "\\Gamma", "Gamma"],
    ["Δ", "\\Delta", "Delta"],
    ["Ε", "\\mathrm{E}", "Épsilon"],
    ["Ζ", "\\mathrm{Z}", "Dseta"],
    ["Η", "\\mathrm{H}", "Eta"],
    ["Θ", "\\Theta", "Theta"],
    ["Ι", "\\mathrm{I}", "Iota"],
    ["Κ", "\\mathrm{K}", "Kappa"],
    ["Λ", "\\Lambda", "Lambda"],
    ["Μ", "\\mathrm{M}", "Mu"],
    ["Ν", "\\mathrm{N}", "Nu"],
    ["Ξ", "\\Xi", "Xi"],
    ["Ο", "\\mathrm{O}", "Ómicron"],
    ["Π", "\\Pi", "Pi"],
    ["Ρ", "\\mathrm{P}", "Rho"],
    ["Σ", "\\Sigma", "Sigma"],
    ["Τ", "\\mathrm{T}", "Tau"],
    ["Υ", "\\Upsilon", "Ípsilon"],
    ["Φ", "\\Phi", "Fi"],
    ["Χ", "\\mathrm{X}", "Ji"],
    ["Ψ", "\\Psi", "Psi"],
    ["Ω", "\\Omega", "Omega"]
  ],
  "upper",
  "Griegas mayúsculas",
  "mayúscula"
);

const commonUnitSymbols = [
  "m",
  "cm",
  "mm",
  "km",
  "kg",
  "g",
  "s",
  "min",
  "h",
  "K",
  "°C",
  "N",
  "J",
  "W",
  "Pa",
  "V",
  "A"
];

export function buildFormulaSections(
  customUnits: PaletteItem[]
): FormulaSection[] {
  const matrixItems = PALETTES.function
    .filter((item) => item.category === "Matrices")
    .map((item) =>
      item.id === "matrix2" ? { ...item, label: "2×2" } : item
    );
  const functionItems = PALETTES.function.filter((item) =>
    supportedFunctions.has(item.id)
  );

  const units = new Map<string, PaletteItem>();
  for (const symbol of commonUnitSymbols) {
    const item = PALETTES.unit.find((candidate) => candidate.label === symbol);
    if (item) {
      units.set(item.id, item);
    }
  }
  for (const item of customUnits) {
    units.set(item.id, item);
  }

  return [
    {
      id: "assignment",
      title: "Asignación",
      items: [ASSIGNMENT_ITEM, variableItem, functionItem]
    },
    {
      id: "basic",
      title: "Básico",
      items: basicItems
    },
    {
      id: "functions",
      title: "Funciones",
      items: functionItems,
      morePalette: "function"
    },
    {
      id: "matrices",
      title: "Matrices",
      items: matrixItems
    },
    {
      id: "symbols",
      title: "Símbolos",
      items: symbolItems
    },
    {
      id: "greek-lower",
      title: "Griegas minúsculas",
      items: lowercaseGreek
    },
    {
      id: "greek-upper",
      title: "Griegas mayúsculas",
      items: uppercaseGreek
    },
    {
      id: "units",
      title: "Unidades",
      items: [...units.values()],
      morePalette: "unit"
    }
  ];
}
