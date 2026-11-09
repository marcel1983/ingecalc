import { describe, expect, it } from "vitest";
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
});
