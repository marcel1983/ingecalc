import { cpSync, existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(root, "node_modules/mathlive/fonts");
const destination = resolve(root, "apps/desktop/public/fonts");

if (!existsSync(source)) {
  console.warn("MathLive fonts not found. Run npm install first.");
  process.exit(0);
}

mkdirSync(destination, { recursive: true });
cpSync(source, destination, { recursive: true });
console.log(`Copied MathLive fonts to ${destination}`);
