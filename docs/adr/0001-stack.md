# ADR-0001: Rust core + Tauri 2 + MathLive

## Status

Accepted. See ADR-0002 for the initial LaTeX + ASCII Math representation.

## Context

We are building an open-source mathematical worksheet editor for Linux. The
product needs:

- A reliable numeric and symbolic core.
- Rich mathematical editing.
- A document-oriented UI with regions, pages and printing/export.
- A plugin model that does not depend on the original SMath binary ABI.
- A GPL-3.0-or-later distribution.

## Decision

- Use **Rust** for the core, evaluator, document model and plugin host.
- Use **Tauri 2** for the desktop shell.
- Use **React + Vite + TypeScript** for the frontend.
- Use **MathLive** for mathematical editing and **KaTeX** for static rendering.
- Use **TipTap/ProseMirror** for rich text.
- Use a versioned **JSON** document format (`.smj`).
- Use an out-of-process JSON-RPC plugin API for the MVP.

## Consequences

Positive:

- Fast iteration on the UI.
- MathLive solves the hardest part of the MVP: editable mathematics.
- Rust provides a strong core and a clean CLI/headless path.
- Plugins can be implemented in any language and isolated from the host.

Negative:

- The UI is a WebView (WebKitGTK on Linux), not a native toolkit.
- The frontend is TypeScript, not Rust.
- PDF export may be rasterized in the MVP.
- Long-term native UI would require a new shell, but the Rust core remains reusable.

## Alternatives considered

- **GTK4-rs / Iced**: more native, but much more work for math editing and text input.
- **Electron**: Chromium consistency, but heavier and not Tauri.
- **`.sm` compatibility**: rejected for the MVP; JSON is the native format.
- **SMath plugin ABI compatibility**: rejected; new API instead.
