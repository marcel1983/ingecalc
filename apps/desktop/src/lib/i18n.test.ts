import { describe, expect, it } from "vitest";
import i18n, { UI_LANGUAGES, normalizeUiLanguage } from "./i18n";

describe("user interface languages", () => {
  it("normalizes unsupported language codes to Spanish", () => {
    expect(normalizeUiLanguage("en")).toBe("en");
    expect(normalizeUiLanguage("ar")).toBe("ar");
    expect(normalizeUiLanguage("xx")).toBe("es");
    expect(normalizeUiLanguage(null)).toBe("es");
  });

  it("loads the six interface languages", () => {
    expect(UI_LANGUAGES.map((language) => language.id)).toEqual([
      "es",
      "en",
      "fr",
      "hi",
      "ar",
      "zh"
    ]);
    for (const language of UI_LANGUAGES) {
      expect(i18n.hasResourceBundle(language.id, "translation")).toBe(true);
    }
  });

  it("translates the main actions", () => {
    expect(i18n.getFixedT("es")("quick.new")).toBe("Nuevo");
    expect(i18n.getFixedT("en")("quick.new")).toBe("New");
    expect(i18n.getFixedT("fr")("quick.new")).toBe("Nouveau");
    expect(i18n.getFixedT("hi")("quick.new")).toBe("नया");
    expect(i18n.getFixedT("ar")("quick.new")).toBe("جديد");
    expect(i18n.getFixedT("zh")("quick.new")).toBe("新建");
    expect(i18n.getFixedT("es")("tabs.help")).toBe("Ayuda");
    expect(i18n.getFixedT("en")("tabs.help")).toBe("Help");
    expect(i18n.getFixedT("ar")("tabs.help")).toBe("مساعدة");
  });
});
