import NSpell from "nspell";

export type SpellcheckLanguage = {
  id: string;
  label: string;
  native: string;
};

export const SPELLCHECK_LANGUAGES: SpellcheckLanguage[] = [
  { id: "es", label: "Español", native: "Español" },
  { id: "en", label: "Inglés", native: "English" },
  { id: "fr", label: "Francés", native: "Français" },
  { id: "hi", label: "Hindi", native: "हिन्दी" },
  { id: "ar", label: "Árabe", native: "العربية" }
];

const SUPPORTED = new Set(SPELLCHECK_LANGUAGES.map((item) => item.id));
const cache = new Map<string, Promise<NSpell | null>>();

function normalizeLanguage(language: string): string {
  const base = language.toLowerCase().split(/[-_]/)[0];
  return SUPPORTED.has(base) ? base : "";
}

export function isSpellcheckLanguage(language: string): boolean {
  return normalizeLanguage(language) !== "";
}

export async function getSpellchecker(
  language: string
): Promise<NSpell | null> {
  const id = normalizeLanguage(language);
  if (!id) {
    return null;
  }

  const cached = cache.get(id);
  if (cached) {
    return cached;
  }

  const pending = (async () => {
    try {
      const base = `${import.meta.env.BASE_URL || "/"}dictionaries/${id}/${id}`;
      const [affResponse, dicResponse] = await Promise.all([
        fetch(`${base}.aff`),
        fetch(`${base}.dic`)
      ]);
      if (!affResponse.ok || !dicResponse.ok) {
        return null;
      }
      const [aff, dic] = await Promise.all([
        affResponse.text(),
        dicResponse.text()
      ]);
      return NSpell(aff, dic);
    } catch {
      return null;
    }
  })();

  cache.set(id, pending);
  return pending;
}

export function isWordAccepted(
  word: string,
  accepted: Set<string>
): boolean {
  return accepted.has(word.toLocaleLowerCase());
}

export function isWordCorrect(checker: NSpell, word: string): boolean {
  if (checker.correct(word)) {
    return true;
  }

  const lower = word.toLocaleLowerCase();
  if (checker.correct(lower)) {
    return true;
  }

  const capitalized = `${lower.charAt(0).toLocaleUpperCase()}${lower.slice(1)}`;
  return checker.correct(capitalized);
}

export function suggestWords(
  checker: NSpell,
  word: string,
  limit = 8
): string[] {
  try {
    return checker.suggest(word).slice(0, limit);
  } catch {
    return [];
  }
}
