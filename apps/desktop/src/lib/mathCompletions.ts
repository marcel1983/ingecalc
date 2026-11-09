import type { Document } from "../types/document";
import { ENGINEERING_CONSTANTS } from "./engineeringConstants";

export type MathCompletionKind =
  | "variable"
  | "function"
  | "constant"
  | "operator";

export type MathCompletion = {
  id: string;
  name: string;
  label?: string;
  kind: MathCompletionKind;
  description: string;
  insert: string;
  selectionMode?: "after" | "placeholder" | "before" | "item";
  rank?: number;
};

function completionLatex(name: string, call = false): string {
  const identifier = name.length > 1 ? `\\text{${name}}` : name;
  return call ? `${identifier}(` : identifier;
}

const FUNCTIONS: Array<{ name: string; description: string }> = [
  { name: "sqrt", description: "Raíz cuadrada" },
  { name: "abs", description: "Valor absoluto" },
  { name: "sin", description: "Seno" },
  { name: "cos", description: "Coseno" },
  { name: "tan", description: "Tangente" },
  { name: "asin", description: "Arcoseno" },
  { name: "acos", description: "Arcocoseno" },
  { name: "atan", description: "Arcotangente" },
  { name: "atan2", description: "Arcotangente de dos argumentos" },
  { name: "sinh", description: "Seno hiperbólico" },
  { name: "cosh", description: "Coseno hiperbólico" },
  { name: "tanh", description: "Tangente hiperbólica" },
  { name: "asinh", description: "Arcoseno hiperbólico" },
  { name: "acosh", description: "Arcocoseno hiperbólico" },
  { name: "atanh", description: "Arcotangente hiperbólica" },
  { name: "ln", description: "Logaritmo natural" },
  { name: "log", description: "Logaritmo decimal" },
  { name: "exp", description: "Exponencial" },
  { name: "floor", description: "Redondear hacia abajo" },
  { name: "ceil", description: "Redondear hacia arriba" },
  { name: "round", description: "Redondear al entero más cercano" },
  { name: "trunc", description: "Parte entera" },
  { name: "frac", description: "Parte fraccionaria" },
  { name: "sign", description: "Signo" },
  { name: "mod", description: "Resto de la división" },
  { name: "gcd", description: "Máximo común divisor" },
  { name: "lcm", description: "Mínimo común múltiplo" },
  { name: "factorial", description: "Factorial" },
  { name: "comb", description: "Combinaciones" },
  { name: "perm", description: "Permutaciones" },
  { name: "min", description: "Mínimo" },
  { name: "max", description: "Máximo" },
  { name: "equal", description: "Comparación de igualdad" },
  { name: "re", description: "Parte real" },
  { name: "im", description: "Parte imaginaria" },
  { name: "conj", description: "Conjugado complejo" },
  { name: "simplify", description: "Simplificar una expresión" },
  { name: "expand", description: "Expandir productos y potencias" },
  { name: "factor", description: "Factorizar una expresión" },
  { name: "subs", description: "Sustituir una variable por una expresión" },
  { name: "substitute", description: "Sustituir una variable por una expresión" },
  { name: "diff", description: "Derivada simbólica" },
  { name: "integrate", description: "Primitiva elemental" },
  { name: "integral", description: "Integral definida numérica" },
  { name: "solve", description: "Resolver una ecuación" },
  { name: "roots", description: "Raíces reales en un intervalo" },
  { name: "polyroots", description: "Raíces de un polinomio" },
  { name: "eval", description: "Evaluar una expresión" },
  { name: "unit", description: "Definir una unidad propia" },
  { name: "if", description: "Condicional: si la condición se cumple" },
  { name: "for", description: "Bucle con contador o sobre una matriz" },
  { name: "sum", description: "Suma de elementos o de un bucle" },
  { name: "product", description: "Producto de elementos o de un bucle" },
  { name: "while", description: "Bucle mientras se cumpla una condición" },
  { name: "line", description: "Secuencia de expresiones con variables locales" },
  { name: "error", description: "Detener el cálculo con un mensaje" },
  { name: "text", description: "Crear un valor de texto" },
  { name: "trace", description: "Devolver un valor para depuración" },
  { name: "det", description: "Determinante" },
  { name: "inv", description: "Matriz inversa" },
  { name: "transpose", description: "Matriz transpuesta" },
  { name: "identity", description: "Matriz identidad" },
  { name: "row", description: "Fila de una matriz" },
  { name: "col", description: "Columna de una matriz" },
  { name: "augment", description: "Concatenar matrices por columnas" },
  { name: "stack", description: "Apilar matrices por filas" },
  { name: "minor", description: "Menor de una matriz" },
  { name: "vminor", description: "Submatriz al eliminar fila y columna" },
  { name: "cofactor", description: "Cofactor de una matriz" },
  { name: "mean", description: "Media aritmética" },
  { name: "range", description: "Vector con un rango de valores" },
  { name: "matrix", description: "Crear una matriz" },
  { name: "resize", description: "Cambiar el tamaño de una matriz" },
  { name: "reshape", description: "Cambiar la forma de una matriz" },
  { name: "flatten", description: "Convertir una matriz en vector" },
  { name: "size", description: "Dimensiones de una matriz" },
  { name: "length", description: "Número de elementos" },
  { name: "findrows", description: "Filas que contienen un valor" },
  { name: "sort", description: "Ordenar los elementos" },
  { name: "sortrows", description: "Ordenar las filas por una columna" },
  { name: "dot", description: "Producto escalar" },
  { name: "cross", description: "Producto vectorial" },
  { name: "norm", description: "Norma euclidiana" },
  { name: "lu", description: "Descomposición LU" },
  { name: "qr", description: "Descomposición QR" },
  { name: "rank", description: "Rango de una matriz" },
  { name: "rref", description: "Forma escalonada reducida" },
  { name: "eigenvalues", description: "Autovalores de una matriz simétrica" }
];

const MATH_CONSTANTS: Array<{
  name: string;
  description: string;
  insert?: string;
}> = [
  { name: "pi", description: "Número pi", insert: "\\pi" },
  { name: "e", description: "Número de Euler", insert: "e" },
  { name: "i", description: "Unidad imaginaria", insert: "i" }
];

const OPERATORS: Array<{
  name: string;
  description: string;
  insert: string;
}> = [
  { name: "+", description: "Suma", insert: "+" },
  { name: "-", description: "Resta", insert: "-" },
  { name: "*", description: "Multiplicación", insert: "\\cdot" },
  { name: "/", description: "División", insert: "/" },
  { name: "^", description: "Potencia", insert: "^" },
  { name: ":=", description: "Asignación o definición", insert: "\\coloneq" },
  { name: "=", description: "Igualdad o marcador de evaluación", insert: "=" },
  { name: "<", description: "Menor que", insert: "<" },
  { name: "<=", description: "Menor o igual que", insert: "\\le" },
  { name: ">", description: "Mayor que", insert: ">" },
  { name: ">=", description: "Mayor o igual que", insert: "\\ge" },
  { name: "==", description: "Igualdad lógica", insert: "==" },
  { name: "!=", description: "Desigualdad lógica", insert: "\\ne" },
  { name: "(", description: "Abrir paréntesis", insert: "(" },
  { name: ")", description: "Cerrar paréntesis", insert: ")" },
  { name: "%", description: "Porcentaje", insert: "%" }
];

const DEFINITION_PATTERN =
  /(?:^|[;\n])\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?:\(([^)]*)\))?\s*:=/g;

type DefinedItem = {
  name: string;
  params: string[];
};

function definedCompletions(document: Document): MathCompletion[] {
  const found = new Map<string, DefinedItem>();

  for (const page of document.pages) {
    for (const region of page.regions) {
      if (region.type !== "math" || !region.ascii) {
        continue;
      }

      for (const match of region.ascii.matchAll(DEFINITION_PATTERN)) {
        const name = match[1];
        const params = (match[2] ?? "")
          .split(",")
          .map((param) => param.trim())
          .filter(Boolean);
        if (!name || name === "unit" || found.has(name)) {
          continue;
        }
        found.set(name, { name, params });
      }
    }
  }

  return [...found.values()].flatMap<MathCompletion>((item) => {
    const base = completionLatex(item.name);
    if (item.params.length === 0) {
      return [
        {
          id: `variable:${item.name}:use`,
          name: item.name,
          label: item.name,
          kind: "variable" as const,
          description: "Usar variable definida",
          insert: base,
          rank: 0
        }
      ];
    }

    const signature = item.params.join(", ");
    const placeholders = item.params.map(() => "\\placeholder{}").join(",");
    return [
      {
        id: `function:${item.name}:evaluate`,
        name: item.name,
        label: `${item.name}(${signature}) =`,
        kind: "function" as const,
        description: "Evaluar función",
        insert: `${base}(${placeholders}) =`,
        selectionMode: "placeholder" as const,
        rank: 0
      },
      {
        id: `function:${item.name}:call`,
        name: item.name,
        label: `${item.name}(${signature})`,
        kind: "function" as const,
        description: "Insertar llamada",
        insert: `${base}(`,
        rank: 1
      }
    ];
  });
}

export function buildMathCompletions(document: Document): MathCompletion[] {
  const defined = definedCompletions(document);
  const definedNames = new Set(defined.map((item) => item.name));
  const functions: MathCompletion[] = FUNCTIONS.map((item) => ({
    id: `builtin:${item.name}`,
    name: item.name,
    kind: "function",
    description: item.description,
    insert: completionLatex(item.name, true)
  }));
  const constants: MathCompletion[] = ENGINEERING_CONSTANTS.map(
    (constant) => ({
      id: `engineering:${constant.name}`,
      name: constant.name,
      kind: "constant",
      description: constant.description,
      insert: completionLatex(constant.name)
    })
  );
  const mathConstants: MathCompletion[] = MATH_CONSTANTS.map((item) => ({
    id: `math:${item.name}`,
    name: item.name,
    kind: "constant",
    description: item.description,
    insert: item.insert ?? completionLatex(item.name)
  }));
  const custom = (document.settings.customConstants ?? []).map(
    (constant) => ({
      id: `custom:${constant.name}`,
      name: constant.name,
      kind: "constant" as const,
      description: constant.description || constant.expression,
      insert: completionLatex(constant.name)
    })
  );
  const operators: MathCompletion[] = OPERATORS.map((item) => ({
    id: `operator:${item.name}`,
    name: item.name,
    kind: "operator",
    description: item.description,
    insert: item.insert
  }));

  const completions = new Map<string, MathCompletion>();
  for (const item of defined) {
    completions.set(item.id, item);
  }
  for (const item of [
    ...functions,
    ...custom,
    ...constants,
    ...mathConstants,
    ...operators
  ]) {
    if (definedNames.has(item.name)) {
      continue;
    }
    if (!completions.has(item.id)) {
      completions.set(item.id, item);
    }
  }

  return [...completions.values()];
}

export function mathConstantNames(document: Document): string[] {
  const defined = new Set(
    definedCompletions(document).map((completion) => completion.name)
  );
  const names = new Set<string>();
  for (const constant of ENGINEERING_CONSTANTS) {
    if (!defined.has(constant.name)) {
      names.add(constant.name);
    }
  }
  for (const constant of document.settings.customConstants ?? []) {
    if (!defined.has(constant.name)) {
      names.add(constant.name);
    }
  }
  return [...names];
}

export function matchMathCompletions(
  completions: MathCompletion[],
  prefix: string,
  limit = 20
): MathCompletion[] {
  const needle = prefix.toLowerCase();
  const matches = needle
    ? completions.filter(
        (completion) =>
          // Exact names are kept for constants, operators, user-defined
          // symbols and built-in functions, so typing `if` or `sin` still
          // offers the corresponding call template.
          (completion.name !== prefix ||
            completion.kind === "constant" ||
            completion.kind === "operator" ||
            completion.id.startsWith("variable:") ||
            completion.id.startsWith("function:") ||
            completion.id.startsWith("builtin:")) &&
          completion.name.toLowerCase().startsWith(needle)
      )
    : [...completions];

  return matches
    .sort((left, right) => {
      const exact = (completion: MathCompletion) =>
        needle && completion.name.toLowerCase() === needle ? 0 : 1;
      const priority = (kind: MathCompletionKind) => {
        if (!needle) {
          switch (kind) {
            case "operator":
              return 0;
            case "variable":
              return 1;
            case "function":
              return 2;
            case "constant":
              return 3;
          }
        }
        switch (kind) {
          case "variable":
            return 0;
          case "function":
            return 1;
          case "constant":
            return 2;
          case "operator":
            return 3;
        }
      };
      return (
        exact(left) - exact(right) ||
        priority(left.kind) - priority(right.kind) ||
        (left.rank ?? 0) - (right.rank ?? 0) ||
        (left.label ?? left.name).localeCompare(right.label ?? right.name)
      );
    })
    .slice(0, limit);
}

const IDENTIFIER_PATTERN = /^[A-Za-z][A-Za-z0-9_]*$/;

/**
 * Suggestions for a partially typed token. If the token is a new identifier,
 * offer a variable definition template. Existing variables only offer their
 * use template because IngeCalc does not allow redefinitions.
 */
export function buildTokenCompletions(
  completions: MathCompletion[],
  token: string,
  limit = 20
): MathCompletion[] {
  const matches = matchMathCompletions(completions, token, limit);
  if (!token || !IDENTIFIER_PATTERN.test(token)) {
    return matches;
  }

  const exact = completions.some((completion) => completion.name === token);
  if (exact || matches.length > 0) {
    return matches;
  }

  return [
    {
      id: `new-variable:${token}`,
      name: token,
      label: `${token} :=`,
      kind: "variable" as const,
      description: "Definir nueva variable",
      insert: `${completionLatex(token)} \\coloneq `,
      rank: 0
    }
  ];
}
