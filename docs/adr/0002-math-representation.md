# ADR-0002: LaTeX source + ASCII Math for the first editor

## Status

Accepted for the scaffold.

## Context

The plan selected MathJSON as the canonical math representation. However,
MathLive 0.111 (the version used by the scaffold) exposes:

- `mathfield.value` → LaTeX.
- `mathfield.getValue("latex")` → LaTeX.
- `mathfield.getValue("ascii-math")` → ASCII Math.
- `mathfield.getValue("math-json")` is **not** part of the `OutputFormat`
  union in this version, even though `math-json` is accepted as an input
  format.

Forcing `"math-json"` through a cast would be brittle at runtime.

## Decision

For the first editor iteration:

- Store the editable LaTeX source in `source`.
- Store the evaluable ASCII Math string in `ascii`.
- Parse `ascii` with the Rust `math-parse` crate and evaluate the resulting
  AST.
- Keep MathJSON as a future canonical representation once MathLive exposes a
  stable serializer or once we add our own LaTeX → MathJSON converter.

## Consequences

Positive:

- No fragile cast or private MathLive API.
- The evaluator gets a simple, testable text format.
- The document remains human-readable and diff-friendly.

Negative:

- The document stores a derived ASCII representation alongside LaTeX.
- A future MathJSON migration will require a schema version bump and a
  converter.

## Follow-up

- Evaluate MathLive updates for a MathJSON output API.
- If needed, implement a LaTeX → AST parser in Rust and drop `ascii`.
- Add round-trip tests for LaTeX/ASCII conversion.
