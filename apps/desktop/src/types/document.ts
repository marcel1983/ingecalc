export type MathRegion = {
  id: string;
  type: "math";
  x: number;
  y: number;
  w: number;
  h: number;
  source: string;
  ascii: string;
  pinned?: boolean;
};

export type TextRegion = {
  id: string;
  type: "text";
  x: number;
  y: number;
  w: number;
  h: number;
  html: string;
  role?: "header" | "footer";
  pinned?: boolean;
};

export type ImageRegion = {
  id: string;
  type: "image";
  x: number;
  y: number;
  w: number;
  h: number;
  src: string;
  pinned?: boolean;
};

export type PlotSeries = {
  expression: string;
  expression2: string;
  kind: "explicit" | "parametric" | "implicit" | "surface";
  parameterMin: number;
  parameterMax: number;
  color: string;
  visible: boolean;
};

export type PlotRegion = {
  id: string;
  type: "plot";
  x: number;
  y: number;
  w: number;
  h: number;
  series: PlotSeries[];
  xMin: number;
  xMax: number;
  yMin: number | null;
  yMax: number | null;
  pinned?: boolean;
};

export type Region = MathRegion | TextRegion | ImageRegion | PlotRegion;

export type Page = {
  id: string;
  width: number;
  height: number;
  regions: Region[];
};

export type Metadata = {
  title: string;
  author: string;
  language: string;
};

export type CustomConstant = {
  name: string;
  expression: string;
  description?: string;
};

export type Settings = {
  units: string;
  unitOverrides?: Record<string, string>;
  customConstants?: CustomConstant[];
  personalDictionary?: string[];
  precision: number;
  fractions: string;
  notation?: string;
  trimZeros?: boolean;
  page: PageSetup;
};

export type PageSetup = {
  marginTop: number;
  marginRight: number;
  marginBottom: number;
  marginLeft: number;
  header: string;
  footer: string;
};

export type Document = {
  format: "ingecalc-json" | "mathdoc-json";
  version: 1;
  metadata: Metadata;
  settings: Settings;
  pages: Page[];
};

export type EvalResponse = {
  value: string | null;
  error: string | null;
};

export type DocumentEvalResult = {
  id: string;
  value: string | null;
  error: string | null;
};
