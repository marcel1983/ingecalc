export function isTauri(): boolean {
  return (
    typeof globalThis.window !== "undefined" &&
    "__TAURI_INTERNALS__" in globalThis.window
  );
}
