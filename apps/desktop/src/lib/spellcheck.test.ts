import { describe, expect, it } from "vitest";
import type NSpell from "nspell";
import {
  isSpellcheckLanguage,
  isWordAccepted,
  isWordCorrect,
  suggestWords
} from "./spellcheck";

function fakeChecker(correct: string[], suggestions: string[] = []): NSpell {
  return {
    correct: (word: string) => correct.includes(word),
    suggest: () => suggestions
  } as unknown as NSpell;
}

describe("spellcheck helpers", () => {
  it("accepts base language codes and rejects unsupported ones", () => {
    expect(isSpellcheckLanguage("es")).toBe(true);
    expect(isSpellcheckLanguage("en-US")).toBe(true);
    expect(isSpellcheckLanguage("fr_FR")).toBe(true);
    expect(isSpellcheckLanguage("zh")).toBe(false);
  });

  it("checks lower-case and capitalized variants", () => {
    const checker = fakeChecker(["hola", "Mundo"]);
    expect(isWordCorrect(checker, "hola")).toBe(true);
    expect(isWordCorrect(checker, "Hola")).toBe(true);
    expect(isWordCorrect(checker, "Mundo")).toBe(true);
    expect(isWordCorrect(checker, "mundo")).toBe(true);
    expect(isWordCorrect(checker, "otro")).toBe(false);
  });

  it("limits suggestions", () => {
    const checker = fakeChecker([], ["uno", "dos", "tres", "cuatro"]);
    expect(suggestWords(checker, "x", 2)).toEqual(["uno", "dos"]);
  });

  it("normalizes accepted personal words", () => {
    const accepted = new Set(["ingecalc", "apellido"]);
    expect(isWordAccepted("IngeCalc", accepted)).toBe(true);
    expect(isWordAccepted("APELLIDO", accepted)).toBe(true);
    expect(isWordAccepted("otro", accepted)).toBe(false);
  });
});
