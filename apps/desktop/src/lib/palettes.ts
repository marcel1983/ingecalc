export type PaletteKind = "function" | "unit";

export type PaletteItem = {
  id: string;
  label: string;
  latex: string;
  sample: string;
  description: string;
  category: string;
};

const functions: PaletteItem[] = [
  {
    id: "sin",
    label: "sin",
    latex: "\\sin\\left(\\placeholder{}\\right)",
    sample: "\\sin\\left(x\\right)",
    description: "Seno",
    category: "Trigonometría"
  },
  {
    id: "cos",
    label: "cos",
    latex: "\\cos\\left(\\placeholder{}\\right)",
    sample: "\\cos\\left(x\\right)",
    description: "Coseno",
    category: "Trigonometría"
  },
  {
    id: "tan",
    label: "tan",
    latex: "\\tan\\left(\\placeholder{}\\right)",
    sample: "\\tan\\left(x\\right)",
    description: "Tangente",
    category: "Trigonometría"
  },
  {
    id: "asin",
    label: "sin⁻¹",
    latex: "\\arcsin\\left(\\placeholder{}\\right)",
    sample: "\\arcsin\\left(x\\right)",
    description: "Arcoseno",
    category: "Trigonometría"
  },
  {
    id: "acos",
    label: "cos⁻¹",
    latex: "\\arccos\\left(\\placeholder{}\\right)",
    sample: "\\arccos\\left(x\\right)",
    description: "Arcocoseno",
    category: "Trigonometría"
  },
  {
    id: "atan",
    label: "tan⁻¹",
    latex: "\\arctan\\left(\\placeholder{}\\right)",
    sample: "\\arctan\\left(x\\right)",
    description: "Arcotangente",
    category: "Trigonometría"
  },
  {
    id: "sinh",
    label: "sinh",
    latex: "\\sinh\\left(\\placeholder{}\\right)",
    sample: "\\sinh\\left(x\\right)",
    description: "Seno hiperbólico",
    category: "Trigonometría"
  },
  {
    id: "cosh",
    label: "cosh",
    latex: "\\cosh\\left(\\placeholder{}\\right)",
    sample: "\\cosh\\left(x\\right)",
    description: "Coseno hiperbólico",
    category: "Trigonometría"
  },
  {
    id: "tanh",
    label: "tanh",
    latex: "\\tanh\\left(\\placeholder{}\\right)",
    sample: "\\tanh\\left(x\\right)",
    description: "Tangente hiperbólica",
    category: "Trigonometría"
  },
  {
    id: "asinh",
    label: "sinh⁻¹",
    latex: "\\text{asinh}\\left(\\placeholder{}\\right)",
    sample: "\\text{asinh}\\left(x\\right)",
    description: "Seno hiperbólico inverso",
    category: "Trigonometría"
  },
  {
    id: "acosh",
    label: "cosh⁻¹",
    latex: "\\text{acosh}\\left(\\placeholder{}\\right)",
    sample: "\\text{acosh}\\left(x\\right)",
    description: "Coseno hiperbólico inverso",
    category: "Trigonometría"
  },
  {
    id: "atanh",
    label: "tanh⁻¹",
    latex: "\\text{atanh}\\left(\\placeholder{}\\right)",
    sample: "\\text{atanh}\\left(x\\right)",
    description: "Tangente hiperbólica inversa",
    category: "Trigonometría"
  },
  {
    id: "ln",
    label: "ln",
    latex: "\\ln\\left(\\placeholder{}\\right)",
    sample: "\\ln\\left(x\\right)",
    description: "Logaritmo natural",
    category: "Álgebra"
  },
  {
    id: "log",
    label: "log",
    latex: "\\log\\left(\\placeholder{}\\right)",
    sample: "\\log\\left(x\\right)",
    description: "Logaritmo decimal",
    category: "Álgebra"
  },
  {
    id: "log10",
    label: "log₁₀",
    latex: "\\log_{10}\\left(\\placeholder{}\\right)",
    sample: "\\log_{10}\\left(x\\right)",
    description: "Logaritmo en base 10",
    category: "Álgebra"
  },
  {
    id: "exp",
    label: "eˣ",
    latex: "e^{\\placeholder{}}",
    sample: "e^{x}",
    description: "Exponencial",
    category: "Álgebra"
  },
  {
    id: "sqrt",
    label: "√",
    latex: "\\sqrt{\\placeholder{}}",
    sample: "\\sqrt{x}",
    description: "Raíz cuadrada",
    category: "Álgebra"
  },
  {
    id: "root",
    label: "ⁿ√",
    latex: "\\sqrt[\\placeholder{}]{\\placeholder{}}",
    sample: "\\sqrt[n]{x}",
    description: "Raíz n-ésima",
    category: "Álgebra"
  },
  {
    id: "abs",
    label: "|x|",
    latex: "\\left|\\placeholder{}\\right|",
    sample: "\\left|x\\right|",
    description: "Valor absoluto",
    category: "Álgebra"
  },
  {
    id: "frac",
    label: "a/b",
    latex: "\\frac{\\placeholder{}}{\\placeholder{}}",
    sample: "\\frac{a}{b}",
    description: "Fracción",
    category: "Álgebra"
  },
  {
    id: "power",
    label: "xⁿ",
    latex: "\\placeholder{}^{\\placeholder{}}",
    sample: "x^{n}",
    description: "Potencia",
    category: "Álgebra"
  },
  {
    id: "factorial",
    label: "n!",
    latex: "\\text{factorial}\\left(\\placeholder{}\\right)",
    sample: "\\text{factorial}\\left(5\\right)",
    description: "Factorial",
    category: "Álgebra"
  },
  {
    id: "binomial",
    label: "C(n,k)",
    latex: "\\text{comb}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{comb}\\left(5,2\\right)",
    description: "Coeficiente binomial",
    category: "Álgebra"
  },
  {
    id: "min",
    label: "min",
    latex: "\\min\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\min\\left(a,b\\right)",
    description: "Mínimo",
    category: "Álgebra"
  },
  {
    id: "max",
    label: "max",
    latex: "\\max\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\max\\left(a,b\\right)",
    description: "Máximo",
    category: "Álgebra"
  },
  {
    id: "floor",
    label: "⌊x⌋",
    latex: "\\left\\lfloor\\placeholder{}\\right\\rfloor",
    sample: "\\left\\lfloor x\\right\\rfloor",
    description: "Parte entera",
    category: "Álgebra"
  },
  {
    id: "ceil",
    label: "⌈x⌉",
    latex: "\\left\\lceil\\placeholder{}\\right\\rceil",
    sample: "\\left\\lceil x\\right\\rceil",
    description: "Techo",
    category: "Álgebra"
  },
  {
    id: "round",
    label: "round",
    latex: "\\text{round}\\left(\\placeholder{}\\right)",
    sample: "\\text{round}\\left(2.6\\right)",
    description: "Redondeo",
    category: "Álgebra"
  },
  {
    id: "trunc",
    label: "trunc",
    latex: "\\text{trunc}\\left(\\placeholder{}\\right)",
    sample: "\\text{trunc}\\left(2.9\\right)",
    description: "Parte entera por truncamiento",
    category: "Álgebra"
  },
  {
    id: "frac-part",
    label: "frac",
    latex: "\\text{frac}\\left(\\placeholder{}\\right)",
    sample: "\\text{frac}\\left(2.9\\right)",
    description: "Parte fraccionaria",
    category: "Álgebra"
  },
  {
    id: "sign",
    label: "sign",
    latex: "\\text{sign}\\left(\\placeholder{}\\right)",
    sample: "\\text{sign}\\left(-3\\right)",
    description: "Signo",
    category: "Álgebra"
  },
  {
    id: "mod",
    label: "mod",
    latex: "\\text{mod}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{mod}\\left(7,3\\right)",
    description: "Módulo",
    category: "Álgebra"
  },
  {
    id: "gcd",
    label: "gcd",
    latex: "\\text{gcd}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{gcd}\\left(12,18\\right)",
    description: "Máximo común divisor",
    category: "Álgebra"
  },
  {
    id: "lcm",
    label: "lcm",
    latex: "\\text{lcm}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{lcm}\\left(12,18\\right)",
    description: "Mínimo común múltiplo",
    category: "Álgebra"
  },
  {
    id: "sum",
    label: "Σ",
    latex: "\\sum_{i=\\placeholder{}}^{\\placeholder{}}\\placeholder{}",
    sample: "\\sum_{i=1}^{n}x",
    description: "Sumatorio",
    category: "Cálculo"
  },
  {
    id: "product",
    label: "Π",
    latex: "\\prod_{i=\\placeholder{}}^{\\placeholder{}}\\placeholder{}",
    sample: "\\prod_{i=1}^{n}x",
    description: "Productorio",
    category: "Cálculo"
  },
  {
    id: "integral",
    label: "∫",
    latex:
      "\\int_{\\placeholder{}}^{\\placeholder{}}\\placeholder{}\\,d\\placeholder{}",
    sample: "\\int_{a}^{b}f\\left(x\\right)\\,dx",
    description: "Integral definida",
    category: "Cálculo"
  },
  {
    id: "limit",
    label: "lím",
    latex: "\\lim_{\\placeholder{}\\to\\placeholder{}}\\placeholder{}",
    sample: "\\lim_{x\\to 0}f\\left(x\\right)",
    description: "Límite",
    category: "Cálculo"
  },
  {
    id: "derivative",
    label: "d/dx",
    latex: "\\frac{d}{d\\placeholder{}}\\left(\\placeholder{}\\right)",
    sample: "\\frac{d}{dx}\\left(f\\right)",
    description: "Derivada",
    category: "Cálculo"
  },
  {
    id: "partial",
    label: "∂/∂x",
    latex:
      "\\frac{\\partial}{\\partial\\placeholder{}}\\left(\\placeholder{}\\right)",
    sample: "\\frac{\\partial}{\\partial x}\\left(f\\right)",
    description: "Derivada parcial",
    category: "Cálculo"
  },
  {
    id: "matrix2",
    label: "matriz 2×2",
    latex:
      "\\begin{pmatrix}\\placeholder{} & \\placeholder{}\\\\\\placeholder{} & \\placeholder{}\\end{pmatrix}",
    sample:
      "\\begin{pmatrix}a & b\\\\c & d\\end{pmatrix}",
    description: "Matriz de 2 por 2",
    category: "Matrices"
  },
  {
    id: "det",
    label: "det",
    latex: "\\det\\left(\\placeholder{}\\right)",
    sample: "\\det\\left(\\begin{pmatrix}a & b\\\\c & d\\end{pmatrix}\\right)",
    description: "Determinante",
    category: "Matrices"
  },
  {
    id: "transpose",
    label: "Aᵀ",
    latex: "\\text{transpose}\\left(\\placeholder{}\\right)",
    sample: "\\text{transpose}\\left(\\begin{pmatrix}a & b\\\\c & d\\end{pmatrix}\\right)",
    description: "Transpuesta",
    category: "Matrices"
  },
  {
    id: "identity",
    label: "Iₙ",
    latex: "\\text{identity}\\left(\\placeholder{}\\right)",
    sample: "\\text{identity}\\left(3\\right)",
    description: "Matriz identidad",
    category: "Matrices"
  },
  {
    id: "inv",
    label: "A⁻¹",
    latex: "\\text{inv}\\left(\\placeholder{}\\right)",
    sample: "\\text{inv}\\left(\\begin{pmatrix}a & b\\\\c & d\\end{pmatrix}\\right)",
    description: "Matriz inversa",
    category: "Matrices"
  },
  {
    id: "solve",
    label: "solve",
    latex:
      "\\text{solve}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample:
      "\\text{solve}\\left(\\begin{pmatrix}a & b\\\\c & d\\end{pmatrix},\\begin{pmatrix}e\\\\f\\end{pmatrix}\\right)",
    description: "Sistema lineal A·x = b",
    category: "Matrices"
  },
  {
    id: "trace",
    label: "trace",
    latex: "\\text{trace}\\left(\\placeholder{}\\right)",
    sample: "\\text{trace}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix}\\right)",
    description: "Traza de una matriz",
    category: "Matrices"
  },
  {
    id: "row",
    label: "row",
    latex: "\\text{row}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{row}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix},2\\right)",
    description: "Fila de una matriz",
    category: "Matrices"
  },
  {
    id: "col",
    label: "col",
    latex: "\\text{col}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{col}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix},1\\right)",
    description: "Columna de una matriz",
    category: "Matrices"
  },
  {
    id: "augment",
    label: "augment",
    latex: "\\text{augment}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{augment}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix},\\begin{pmatrix}5\\\\6\\end{pmatrix}\\right)",
    description: "Concatenar columnas",
    category: "Matrices"
  },
  {
    id: "stack",
    label: "stack",
    latex: "\\text{stack}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{stack}\\left(\\begin{pmatrix}1 & 2\\end{pmatrix},\\begin{pmatrix}3 & 4\\end{pmatrix}\\right)",
    description: "Apilar filas",
    category: "Matrices"
  },
  {
    id: "minor",
    label: "minor",
    latex: "\\text{minor}\\left(\\placeholder{},\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{minor}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix},1,1\\right)",
    description: "Menor de una matriz",
    category: "Matrices"
  },
  {
    id: "cofactor",
    label: "cofactor",
    latex: "\\text{cofactor}\\left(\\placeholder{},\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{cofactor}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix},1,1\\right)",
    description: "Cofactor de una matriz",
    category: "Matrices"
  },
  {
    id: "sum-elements",
    label: "Σ",
    latex: "\\text{sum}\\left(\\placeholder{}\\right)",
    sample: "\\text{sum}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix}\\right)",
    description: "Suma de elementos",
    category: "Estadística"
  },
  {
    id: "mean",
    label: "mean",
    latex: "\\text{mean}\\left(\\placeholder{}\\right)",
    sample: "\\text{mean}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix}\\right)",
    description: "Media aritmética",
    category: "Estadística"
  },
  {
    id: "if-function",
    label: "if",
    latex:
      "\\text{if}\\left(\\placeholder{},\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{if}\\left(x<1,1,0\\right)",
    description: "Condicional: valor si la condición es verdadera, valor si es falsa",
    category: "Lógica y control"
  },
  {
    id: "for-loop",
    label: "for",
    latex:
      "\\text{for}\\left(i,1,\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{for}\\left(i,1,10,i^{2}\\right)",
    description: "Lista con el cuerpo evaluado para cada índice",
    category: "Bucles"
  },
  {
    id: "sum-loop",
    label: "Σ(i)",
    latex:
      "\\text{sum}\\left(i,1,\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{sum}\\left(i,1,10,i^{2}\\right)",
    description: "Suma el cuerpo para cada índice",
    category: "Bucles"
  },
  {
    id: "product-loop",
    label: "∏(i)",
    latex:
      "\\text{product}\\left(i,1,\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{product}\\left(i,1,10,i\\right)",
    description: "Multiplica el cuerpo para cada índice",
    category: "Bucles"
  },
  {
    id: "while-loop",
    label: "while",
    latex:
      "\\text{while}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample:
      "\\text{line}\\left(x\\coloneq0,\\text{while}\\left(x<5,x\\coloneq x+1\\right),x\\right)",
    description: "Repite el cuerpo mientras la condición sea verdadera",
    category: "Bucles"
  },
  {
    id: "line-block",
    label: "line",
    latex:
      "\\text{line}\\left(\\placeholder{},\\placeholder{},\\placeholder{}\\right)",
    sample:
      "\\text{line}\\left(x\\coloneq1,x\\coloneq x+1,x^{2}\\right)",
    description: "Ejecuta varias expresiones en secuencia y devuelve la última",
    category: "Lógica y control"
  },
  {
    id: "trace-debug",
    label: "trace",
    latex:
      "\\text{trace}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{trace}\\left(\\text{valor},x^{2}\\right)",
    description: "Devuelve el valor y lo marca para depuración",
    category: "Lógica y control"
  },
  {
    id: "error-debug",
    label: "error",
    latex: "\\text{error}\\left(\\placeholder{}\\right)",
    sample: "\\text{error}\\left(\\text{mensaje}\\right)",
    description: "Detiene el cálculo con un mensaje de error",
    category: "Lógica y control"
  },
  {
    id: "range",
    label: "range",
    latex: "\\text{range}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{range}\\left(1,10\\right)",
    description: "Vector con un rango inclusivo de valores",
    category: "Matrices"
  },
  {
    id: "eigenvalues",
    label: "eigen",
    latex: "\\text{eigenvalues}\\left(\\placeholder{}\\right)",
    sample: "\\text{eigenvalues}\\left(\\begin{pmatrix}2 & 1\\\\1 & 2\\end{pmatrix}\\right)",
    description: "Autovalores de una matriz simétrica",
    category: "Matrices"
  },
  {
    id: "rank",
    label: "rank",
    latex: "\\text{rank}\\left(\\placeholder{}\\right)",
    sample: "\\text{rank}\\left(\\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix}\\right)",
    description: "Rango de una matriz",
    category: "Matrices"
  },
  {
    id: "dot-product",
    label: "dot",
    latex:
      "\\text{dot}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{dot}\\left([1,2,3],[4,5,6]\\right)",
    description: "Producto escalar de dos vectores",
    category: "Matrices"
  },
  {
    id: "cross-product",
    label: "cross",
    latex:
      "\\text{cross}\\left(\\placeholder{},\\placeholder{}\\right)",
    sample: "\\text{cross}\\left([1,0,0],[0,1,0]\\right)",
    description: "Producto vectorial de dos vectores 3D",
    category: "Matrices"
  },
  {
    id: "vector",
    label: "vector",
    latex: "\\vec{\\placeholder{}}",
    sample: "\\vec{v}",
    description: "Vector",
    category: "Matrices"
  }
];

const units: PaletteItem[] = [
  { symbol: "m", description: "Metro", category: "SI básicas" },
  { symbol: "kg", description: "Kilogramo", category: "SI básicas" },
  { symbol: "s", description: "Segundo", category: "SI básicas" },
  { symbol: "A", description: "Amperio", category: "SI básicas" },
  { symbol: "K", description: "Kelvin", category: "SI básicas" },
  { symbol: "mol", description: "Mol", category: "SI básicas" },
  { symbol: "cd", description: "Candela", category: "SI básicas" },
  { symbol: "Hz", description: "Hercio", category: "SI derivadas" },
  { symbol: "N", description: "Newton", category: "SI derivadas" },
  { symbol: "Pa", description: "Pascal", category: "SI derivadas" },
  { symbol: "J", description: "Julio", category: "SI derivadas" },
  { symbol: "W", description: "Vatio", category: "SI derivadas" },
  { symbol: "C", description: "Culombio", category: "SI derivadas" },
  { symbol: "V", description: "Voltio", category: "SI derivadas" },
  { symbol: "F", description: "Faradio", category: "SI derivadas" },
  { symbol: "Ω", description: "Ohmio", category: "SI derivadas" },
  { symbol: "S", description: "Siemens", category: "SI derivadas" },
  { symbol: "Wb", description: "Weber", category: "SI derivadas" },
  { symbol: "T", description: "Tesla", category: "SI derivadas" },
  { symbol: "H", description: "Henrio", category: "SI derivadas" },
  { symbol: "°C", description: "Grado Celsius", category: "SI derivadas" },
  { symbol: "lm", description: "Lumen", category: "SI derivadas" },
  { symbol: "lx", description: "Lux", category: "SI derivadas" },
  { symbol: "Bq", description: "Becquerel", category: "SI derivadas" },
  { symbol: "Gy", description: "Gray", category: "SI derivadas" },
  { symbol: "Sv", description: "Sievert", category: "SI derivadas" },
  { symbol: "kat", description: "Katal", category: "SI derivadas" },
  { symbol: "L", description: "Litro", category: "Comunes" },
  { symbol: "mL", description: "Mililitro", category: "Comunes" },
  { symbol: "min", description: "Minuto", category: "Comunes" },
  { symbol: "h", description: "Hora", category: "Comunes" },
  { symbol: "d", description: "Día", category: "Comunes" },
  { symbol: "g", description: "Gramo", category: "Comunes" },
  { symbol: "cm", description: "Centímetro", category: "Comunes" },
  { symbol: "mm", description: "Milímetro", category: "Comunes" },
  { symbol: "km", description: "Kilómetro", category: "Comunes" },
  { symbol: "µm", description: "Micrómetro", category: "Comunes" },
  { symbol: "m/s", description: "Metro por segundo", category: "Comunes" },
  { symbol: "m/s²", description: "Metro por segundo al cuadrado", category: "Comunes" },
  { symbol: "km/h", description: "Kilómetro por hora", category: "Comunes" },
  { symbol: "kg/m³", description: "Kilogramo por metro cúbico", category: "Comunes" },
  { symbol: "N·m", description: "Newton metro", category: "Comunes" },
  { symbol: "kWh", description: "Kilovatio hora", category: "Comunes" },
  { symbol: "m²", description: "Metro cuadrado", category: "Comunes" },
  { symbol: "m³", description: "Metro cúbico", category: "Comunes" },
  { symbol: "bar", description: "Bar", category: "Comunes" },
  { symbol: "atm", description: "Atmósfera", category: "Comunes" },
  { symbol: "psi", description: "Libra por pulgada cuadrada", category: "Comunes" },
  { symbol: "kgf", description: "Kilogramo fuerza", category: "Comunes" },
  { symbol: "ksi", description: "Kilolibra por pulgada cuadrada", category: "Imperiales" },
  { symbol: "in", description: "Pulgada", category: "Imperiales" },
  { symbol: "ft", description: "Pie", category: "Imperiales" },
  { symbol: "yd", description: "Yarda", category: "Imperiales" },
  { symbol: "mi", description: "Milla", category: "Imperiales" },
  { symbol: "lb", description: "Libra", category: "Imperiales" },
  { symbol: "oz", description: "Onza", category: "Imperiales" },
  { symbol: "gal", description: "Galón estadounidense", category: "Imperiales" },
  { symbol: "qt", description: "Cuarto estadounidense", category: "Imperiales" },
  { symbol: "pt", description: "Pinta estadounidense", category: "Imperiales" },
  { symbol: "acre", description: "Acre", category: "Imperiales" },
  { symbol: "in²", description: "Pulgada cuadrada", category: "Imperiales" },
  { symbol: "ft²", description: "Pie cuadrado", category: "Imperiales" },
  { symbol: "ft³", description: "Pie cúbico", category: "Imperiales" },
  { symbol: "°F", description: "Grado Fahrenheit", category: "Imperiales" },
  { symbol: "lbf", description: "Libra fuerza", category: "Imperiales" },
  { symbol: "ft·lb", description: "Pie-libra", category: "Imperiales" },
  { symbol: "hp", description: "Caballo de fuerza", category: "Imperiales" },
  { symbol: "°", description: "Grado angular", category: "Comunes" },
  { symbol: "%", description: "Por ciento", category: "Comunes" },
  { symbol: "rad", description: "Radián", category: "Comunes" },
  { symbol: "sr", description: "Estereorradián", category: "Comunes" }
].map(({ symbol, description, category }) => ({
  id: `unit-${symbol}`,
  label: symbol,
  latex: unitLatex(symbol),
  sample: unitLatex(symbol),
  description,
  category
}));

export const PALETTES: Record<PaletteKind, PaletteItem[]> = {
  function: functions,
  unit: units
};

export const PALETTE_TITLES: Record<PaletteKind, string> = {
  function: "Funciones y plantillas",
  unit: "Unidades"
};

function unitSegment(symbol: string): string {
  return `\\textcolor{blue}{\\text{${symbol}}}`;
}

function unitLatex(symbol: string): string {
  const composite: Record<string, string> = {
    "m/s": `${unitSegment("m")}/${unitSegment("s")}`,
    "m/s²": `${unitSegment("m")}/${unitSegment("s")}^{2}`,
    "km/h": `${unitSegment("km")}/${unitSegment("h")}`,
    "kg/m³": `${unitSegment("kg")}/${unitSegment("m")}^{3}`,
    "N·m": `${unitSegment("N")}\\cdot${unitSegment("m")}`,
    "°C": unitSegment("°C"),
    "°": unitSegment("°"),
    "Ω": unitSegment("Ω"),
    "µm": unitSegment("µm"),
    "%": unitSegment("\\%"),
    kWh: unitSegment("kWh")
  };
  return composite[symbol] ?? unitSegment(symbol);
}
