import { describe, expect, it } from "vitest";
import { newDocument } from "../stores/document";
import {
  buildMathCompletions,
  buildTokenCompletions,
  matchMathCompletions,
  mathConstantNames
} from "./mathCompletions";

describe("math completions", () => {
  it("collects variables, functions and constants", () => {
    const document = newDocument();
    document.pages[0].regions = [
      {
        id: "math-1",
        type: "math",
        x: 0,
        y: 0,
        w: 100,
        h: 40,
        source: "x := 3",
        ascii: "x := 3"
      },
      {
        id: "math-2",
        type: "math",
        x: 0,
        y: 60,
        w: 100,
        h: 40,
        source: "f(t) := t^2",
        ascii: "f(t) := t^2"
      }
    ];
    document.settings.customConstants = [
      { name: "gcustom", expression: "9.81 m/s^2", description: "Gravedad" }
    ];

    const completions = buildMathCompletions(document);
    expect(completions.find((item) => item.name === "x")?.kind).toBe(
      "variable"
    );
    expect(completions.find((item) => item.name === "f")?.kind).toBe(
      "function"
    );
    expect(completions.find((item) => item.name === "gn")?.kind).toBe(
      "constant"
    );
    expect(completions.find((item) => item.name === "gcustom")?.kind).toBe(
      "constant"
    );
  });

  it("includes built-in functions and basic operators", () => {
    const document = newDocument();
    const completions = buildMathCompletions(document);

    expect(completions.find((item) => item.name === "sin")?.kind).toBe(
      "function"
    );
    expect(completions.find((item) => item.name === "solve")?.kind).toBe(
      "function"
    );
    expect(completions.find((item) => item.name === "while")?.kind).toBe(
      "function"
    );
    expect(completions.find((item) => item.name === "+")?.kind).toBe(
      "operator"
    );
    expect(completions.find((item) => item.name === ":=")?.kind).toBe(
      "operator"
    );
  });

  it("matches function prefixes and opens the full list", () => {
    const document = newDocument();
    const completions = buildMathCompletions(document);

    const matches = matchMathCompletions(completions, "si").map(
      (item) => item.name
    );
    expect(matches).toContain("sin");
    expect(matches).toContain("sign");
    expect(matches).toContain("simplify");

    const all = matchMathCompletions(
      completions,
      "",
      completions.length
    );
    expect(all.length).toBeGreaterThan(50);
    expect(all.some((item) => item.kind === "operator")).toBe(true);
    expect(all.some((item) => item.kind === "function")).toBe(true);
  });

  it("suggests variable use and evaluation for defined functions", () => {
    const document = newDocument();
    document.pages[0].regions = [
      {
        id: "variable",
        type: "math",
        x: 0,
        y: 0,
        w: 100,
        h: 40,
        source: "Rev := 0.38",
        ascii: "Rev := 0.38"
      },
      {
        id: "function",
        type: "math",
        x: 0,
        y: 60,
        w: 100,
        h: 40,
        source: "f(x, y) := x + y",
        ascii: "f(x, y) := x + y"
      }
    ];

    const completions = buildMathCompletions(document);
    const variableUse = completions.find(
      (item) => item.id === "variable:Rev:use"
    );
    const variableDefine = completions.find(
      (item) => item.id === "variable:Rev:define"
    );
    const functionEvaluate = completions.find(
      (item) => item.id === "function:f:evaluate"
    );

    expect(variableDefine).toBeUndefined();
    expect(variableUse?.label).toBe("Rev");
    expect(functionEvaluate?.label).toBe("f(x, y) =");
    expect(functionEvaluate?.insert).toContain("\\placeholder{}");
    expect(functionEvaluate?.insert?.endsWith(" =")).toBe(true);
  });

  it("offers := only for new variable names", () => {
    const document = newDocument();
    document.pages[0].regions = [
      {
        id: "variable",
        type: "math",
        x: 0,
        y: 0,
        w: 100,
        h: 40,
        source: "Rev := 0.38",
        ascii: "Rev := 0.38"
      }
    ];

    const completions = buildMathCompletions(document);
    const existing = buildTokenCompletions(completions, "Rev");
    expect(existing.map((item) => item.label)).toEqual(["Rev"]);

    const fresh = buildTokenCompletions(completions, "Momento");
    expect(fresh[0]?.label).toBe("Momento :=");
    expect(fresh[0]?.description).toBe("Definir nueva variable");
    expect(fresh[0]?.insert).toContain("\\coloneq");
  });

  it("suggests an exact built-in function such as if", () => {
    const document = newDocument();
    const completions = buildMathCompletions(document);

    const items = buildTokenCompletions(completions, "if");
    expect(items[0]?.name).toBe("if");
    expect(items[0]?.kind).toBe("function");
    expect(items[0]?.insert).toBe("\\text{if}(");
    expect(items.some((item) => item.id.startsWith("new-variable:"))).toBe(
      false
    );
  });

  it("suggests exact built-in functions without turning them into variables", () => {
    const document = newDocument();
    const completions = buildMathCompletions(document);

    const sin = buildTokenCompletions(completions, "sin");
    expect(sin[0]?.name).toBe("sin");
    expect(sin[0]?.id).toBe("builtin:sin");
    expect(sin.some((item) => item.id.startsWith("new-variable:"))).toBe(
      false
    );
  });

  it("matches prefixes with variables before constants", () => {
    const completions = [
      {
        id: "constant:Grav",
        name: "Grav",
        kind: "constant" as const,
        description: "",
        insert: "Grav"
      },
      {
        id: "variable:grado:use",
        name: "grado",
        kind: "variable" as const,
        description: "",
        insert: "grado"
      },
      {
        id: "constant:gn",
        name: "gn",
        kind: "constant" as const,
        description: "",
        insert: "gn"
      }
    ];

    expect(matchMathCompletions(completions, "g").map((item) => item.name)).toEqual([
      "grado",
      "gn",
      "Grav"
    ]);
    expect(matchMathCompletions(completions, "gn").map((item) => item.name)).toEqual([
      "gn"
    ]);
  });

  it("does not style a constant that is redefined by the document", () => {
    const document = newDocument();
    document.pages[0].regions = [
      {
        id: "math-1",
        type: "math",
        x: 0,
        y: 0,
        w: 100,
        h: 40,
        source: "gn := 10",
        ascii: "gn := 10"
      }
    ];

    expect(mathConstantNames(document)).not.toContain("gn");
    expect(mathConstantNames(document)).toContain("clight");
  });
});
