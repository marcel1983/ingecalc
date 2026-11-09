# ADR-0003: LaTeX and ODT export

## Status

Accepted.

## Context

The MVP must export worksheets to LaTeX and ODT. These formats are flow-based,
while the IngeCalc editor is page/region-based. A one-to-one layout mapping is
not possible without a full desktop-publishing renderer.

## Decision

### LaTeX

- Generate a `.tex` file with a portable preamble.
- Text regions are converted from TipTap HTML to paragraphs, headings, lists,
  bold, italic, underline, code and links.
- Math regions use the LaTeX stored by MathLive in `source`.
- Images are copied to an `assets/` directory next to the `.tex` file.
- Page boundaries become `\newpage`.

### ODT

- Generate a valid ODF text package with:
  - `mimetype` (stored first, uncompressed);
  - `content.xml`, `styles.xml`, `meta.xml`, `settings.xml`;
  - `META-INF/manifest.xml`;
  - `Pictures/` for embedded images.
- Text regions become ODF paragraphs, headings and lists.
- Math regions become **MathML** derived from the parsed ASCII Math AST.
- If a formula cannot be parsed, its LaTeX source is exported as monospaced
  text with a clear fallback style.
- Images are embedded in the package.

## Consequences

Positive:

- No external office suite or Pandoc dependency.
- ODT is editable in LibreOffice and other ODF consumers.
- LaTeX is editable and can be compiled with pdfLaTeX, XeLaTeX or LuaLaTeX.

Negative:

- Absolute region positions and page layout are not preserved; the export is a
  semantic flow.
- MathML coverage is limited by the current parser/AST. Complex formulas may
  fall back to text until the MathML exporter or the AST is extended.
- ODT styling is intentionally minimal in the MVP.

## PDF (MVP addendum)

PDF export is implemented in the frontend to avoid adding a native layout
engine to the first release:

- static KaTeX rendering for math regions;
- sanitized TipTap HTML for text regions;
- `html2canvas` page capture;
- multipage composition with `jsPDF`;
- a Tauri `save_pdf` command writes the Base64 result to disk.

The output is raster. A vector PDF backend remains a phase-2 goal.

## Follow-up

- Add a `math-latex`/`math-mathml` crate when the AST grows.
- Add a Pandoc/LibreOffice backend as an optional high-fidelity path.
- Add vector PDF export via LaTeX compilation or an internal renderer.
- Add golden tests for ODT packages, LaTeX documents and PDF page counts.
