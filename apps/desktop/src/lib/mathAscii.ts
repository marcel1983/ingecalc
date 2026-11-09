/**
 * Normalize the compact ASCII-math produced by MathLive.
 *
 * MathLive serializes assignments such as `x := 9` as `x : =9` because it
 * treats the colon as a relation. The evaluator expects a contiguous `:=`,
 * so collapse the whitespace and normalize the Unicode variants.
 */
export function normalizeAsciiMath(input: string, source?: string): string {
  let result = input
    .replace(/\u2254|\u2a3d|\u2a3e/g, ":=")
    .replace(/\u2236|\uff1a/g, ":")
    .replace(/\u2212/g, "-")
    .replace(/\\mathop\b/g, ":=")
    .replace(/-:/g, "/")
    .replace(/\s*:\s*=\s*/g, " := ")
    .trim();

  if (source) {
    for (const name of adjacentLetterRuns(source)) {
      result = joinSpacedName(result, name);
    }
  }

  return result;
}

function adjacentLetterRuns(source: string): string[] {
  const withoutCommands = source.replace(/\\[a-zA-Z]+/g, " ");
  const runs = withoutCommands.match(/[A-Za-z]{2,}/g) ?? [];
  return [...new Set(runs)].sort((left, right) => right.length - left.length);
}

function joinSpacedName(input: string, name: string): string {
  const spaced = name.split("").join(" ");
  const pattern = new RegExp(`(^|[^A-Za-z])${spaced}(?=$|[^A-Za-z])`, "g");
  return input.replace(pattern, (_match, prefix: string) => `${prefix}${name}`);
}
