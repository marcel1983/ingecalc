import { describe, expect, it } from "vitest";
import {
  MIN_CONTENT_WIDTH_CHARS,
  isMathSourceEmpty,
  isTextHtmlEmpty
} from "./regionContent";

describe("region content", () => {
  it("uses a five-character minimum width", () => {
    expect(MIN_CONTENT_WIDTH_CHARS).toBe(5);
  });

  it("detects empty math sources", () => {
    expect(isMathSourceEmpty("")).toBe(true);
    expect(isMathSourceEmpty("   ")).toBe(true);
    expect(isMathSourceEmpty("\\placeholder{}")).toBe(true);
    expect(isMathSourceEmpty("x^2")).toBe(false);
  });

  it("detects empty text documents", () => {
    expect(isTextHtmlEmpty("")).toBe(true);
    expect(isTextHtmlEmpty("<p></p>")).toBe(true);
    expect(isTextHtmlEmpty("<p><br></p>")).toBe(true);
    expect(isTextHtmlEmpty("<p>&nbsp;</p>")).toBe(true);
    expect(isTextHtmlEmpty("<p>Texto</p>")).toBe(false);
    expect(isTextHtmlEmpty("<p><span data-inline-math></span></p>")).toBe(false);
    expect(isTextHtmlEmpty("<table><tbody><tr><td></td></tr></tbody></table>")).toBe(
      false
    );
    expect(isTextHtmlEmpty("<p><img src=\"x\"></p>")).toBe(false);
  });
});
