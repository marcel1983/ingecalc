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

  result = replaceDelimited(result, "||", "||", "norm");
  result = replaceDelimited(result, "\\|", "\\|", "norm");
  result = replaceDelimited(result, "\\lfloor", "\\rfloor", "floor");
  result = replaceDelimited(result, "\\lceil", "\\rceil", "ceil");
  result = replaceDelimited(result, "|", "|", "abs");
  result = joinMultiLetterSubscripts(result);
  result = result
    .replace(/\barcsin\s*\(/g, "asin(")
    .replace(/\barccos\s*\(/g, "acos(")
    .replace(/\barctan\s*\(/g, "atan(")
    .replace(/log\s*_(?:\(10\)|10)\s*\(/g, "log(")
    .replace(
      /root\s*\(([^()]*)\)\s*\(([^()]*)\)/g,
      "($2)^(1/($1))"
    )
    .replace(
      /\b(sum|prod)\s*_\(([A-Za-z])\s*=\s*([^)]+)\)\s*\^\s*([A-Za-z0-9]+)\s+([A-Za-z0-9_]+)/g,
      (_match, fn: string, index: string, start: string, end: string, body: string) =>
        `${fn === "prod" ? "product" : "sum"}(${index},${start},${end},${body})`
    );

  if (source) {
    for (const name of adjacentLetterRuns(source)) {
      result = joinSpacedName(result, name);
    }
  }

  return result;
}

/**
 * MathLive serializes `\left|...\right|` as `|...|`, which the evaluator does
 * not accept. Convert visual delimiters to the equivalent function calls.
 */
function replaceDelimited(
  input: string,
  open: string,
  close: string,
  functionName: string
): string {
  let output = "";
  let index = 0;

  while (index < input.length) {
    if (input.startsWith(open, index)) {
      const end = findClosing(input, index + open.length, open, close);
      if (end >= 0) {
        const inner = input.slice(index + open.length, end).trim();
        output += `${functionName}(${replaceDelimited(inner, open, close, functionName)})`;
        index = end + close.length;
        continue;
      }
    }

    output += input[index];
    index += 1;
  }

  return output;
}

function findClosing(
  input: string,
  start: number,
  open: string,
  close: string
): number {
  if (open === close) {
    return input.indexOf(close, start);
  }

  let depth = 1;
  let index = start;

  while (index < input.length) {
    if (input.startsWith(open, index)) {
      depth += 1;
      index += open.length;
      continue;
    }
    if (input.startsWith(close, index)) {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
      index += close.length;
      continue;
    }
    index += 1;
  }

  return -1;
}

/**
 * MathLive writes multi-letter subscripts as `p_(y2)` or `M_(p S h i d r o)`.
 * The evaluator expects a single identifier such as `p_y2` or `M_pShidro`.
 */
function joinMultiLetterSubscripts(input: string): string {
  return input.replace(
    /_\(([A-Za-z0-9]+(?:\s+[A-Za-z0-9]+)*)\)/g,
    (_match, body: string) => `_${body.replace(/\s+/g, "")}`
  );
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
