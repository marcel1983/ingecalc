import { describe, expect, it } from "vitest";
import { convertLatexToAsciiMath } from "mathlive/ssr";
import { normalizeAsciiMath } from "./mathAscii";

describe("ASCII math normalization", () => {
  it("repairs the assignment spacing produced by MathLive", () => {
    expect(normalizeAsciiMath("x : =9")).toBe("x := 9");
    expect(normalizeAsciiMath("x:=9")).toBe("x := 9");
    expect(normalizeAsciiMath("x ≔ 9")).toBe("x := 9");
    expect(normalizeAsciiMath("x\\mathop 5")).toBe("x := 5");
  });

  it("normalizes unicode minus signs", () => {
    expect(normalizeAsciiMath("x − 1")).toBe("x - 1");
    expect(normalizeAsciiMath("5 -: 2")).toBe("5 / 2");
  });

  it("keeps adjacent letters from the source as one identifier", () => {
    expect(normalizeAsciiMath("f g := 56", "fg:=56")).toBe("fg := 56");
    expect(normalizeAsciiMath("f g + 1", "fg+1")).toBe("fg + 1");
    expect(normalizeAsciiMath("x y", "x y")).toBe("x y");
    expect(normalizeAsciiMath("2x y", "2xy")).toBe("2xy");
  });

  it("converts visual delimiters to function calls", () => {
    expect(normalizeAsciiMath("|p_(y2)-p_(y1)|")).toBe("abs(p_y2-p_y1)");
    expect(normalizeAsciiMath("\\lfloorx\\rfloor")).toBe("floor(x)");
    expect(normalizeAsciiMath("\\lceil x\\rceil")).toBe("ceil(x)");
    expect(normalizeAsciiMath("\\|x\\|")).toBe("norm(x)");
    expect(normalizeAsciiMath("||x||")).toBe("norm(x)");
    expect(normalizeAsciiMath("|x|+|y|")).toBe("abs(x)+abs(y)");
  });

  it("joins multi-letter subscripts serialized by MathLive", () => {
    expect(normalizeAsciiMath("M_(p S h i d r o)")).toBe("M_pShidro");
    expect(normalizeAsciiMath("p_(y2) + p_(y1)")).toBe("p_y2 + p_y1");
    expect(normalizeAsciiMath("x_(n+1)")).toBe("x_(n+1)");
  });

  it("normalizes the ASCII produced by MathLive for visual delimiters", () => {
    expect(
      normalizeAsciiMath(
        convertLatexToAsciiMath("\\left|p_{y2}-p_{y1}\\right|")
      )
    ).toBe("abs(p_y2-p_y1)");
    expect(
      normalizeAsciiMath(
        convertLatexToAsciiMath("\\left\\lfloor x\\right\\rfloor")
      )
    ).toBe("floor(x)");
    expect(
      normalizeAsciiMath(
        convertLatexToAsciiMath("\\left\\lceil x\\right\\rceil")
      )
    ).toBe("ceil(x)");
    expect(
      normalizeAsciiMath(convertLatexToAsciiMath("\\log_{10}\\left(100\\right)"))
    ).toBe("log(100)");
    expect(
      normalizeAsciiMath(convertLatexToAsciiMath("\\sqrt[3]{8}"))
    ).toBe("(8)^(1/(3))");
    expect(
      normalizeAsciiMath(convertLatexToAsciiMath("\\arctan\\left(x\\right)"))
    ).toBe("atan(x)");
    expect(
      normalizeAsciiMath(convertLatexToAsciiMath("\\sum_{i=1}^{n}x"))
    ).toBe("sum(i,1,n,x)");
    expect(
      normalizeAsciiMath(convertLatexToAsciiMath("\\prod_{i=1}^{n}x"))
    ).toBe("product(i,1,n,x)");
  });
});
