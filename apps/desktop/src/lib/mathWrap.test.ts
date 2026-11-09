import { describe, expect, it } from "vitest";
import { wrapSelectedMath } from "./mathWrap";

describe("selected math wrapping", () => {
  it("wraps a selection in parentheses and square roots", () => {
    expect(
      wrapSelectedMath("\\left(\\placeholder{}\\right)", "x + 1")
    ).toBe("\\left(x + 1\\right)");
    expect(wrapSelectedMath("\\sqrt{\\placeholder{}}", "x + 1")).toBe(
      "\\sqrt{x + 1}"
    );
    expect(wrapSelectedMath("\\left|\\placeholder{}\\right|", "x")).toBe(
      "\\left|x\\right|"
    );
  });

  it("wraps powers with parentheses", () => {
    expect(
      wrapSelectedMath("\\placeholder{}^{\\placeholder{}}", "x + 1")
    ).toBe("\\left(x + 1\\right)^{\\placeholder{}}");
    expect(wrapSelectedMath("\\placeholder{}^{2}", "x + 1")).toBe(
      "\\left(x + 1\\right)^{2}"
    );
  });

  it("wraps selections before arithmetic operators", () => {
    expect(wrapSelectedMath("+", "a + b")).toBe("\\left(a + b\\right)+");
    expect(wrapSelectedMath("-", "a + b")).toBe("\\left(a + b\\right)-");
    expect(wrapSelectedMath("\\times", "a + b")).toBe(
      "\\left(a + b\\right)\\times"
    );
    expect(wrapSelectedMath("\\div", "a + b")).toBe(
      "\\left(a + b\\right)\\div"
    );
  });

  it("returns null when there is no placeholder or no selection", () => {
    expect(wrapSelectedMath(":=", "x")).toBeNull();
    expect(wrapSelectedMath("\\sqrt{\\placeholder{}}", "  ")).toBeNull();
  });
});
