# Arquitectura de IngeCalc

IngeCalc separa el editor de escritorio del núcleo de cálculo. La interfaz nunca
evalúa matemática por su cuenta: envía el documento al backend Rust y recibe
valores, errores y datos de gráficos.

## Vista general

```text
┌─────────────────────────────────────────────────────────────┐
│ apps/desktop                                                │
│ React + Vite + Tauri 2 + MathLive + KaTeX + TipTap          │
│ Regiones, páginas, ribbon, paletas, autocompletado, export  │
└──────────────────────────────┬──────────────────────────────┘
                               │ comandos Tauri
┌──────────────────────────────▼──────────────────────────────┐
│ apps/desktop/src-tauri                                      │
│ Ventana, diálogos, autosave, imágenes, PDF y puente Rust    │
└──────────────────────────────┬──────────────────────────────┘
                               │ crates del workspace
┌──────────────────────────────▼──────────────────────────────┐
│ doc-core / doc-json / math-* / export / plugin-*            │
│ Documento, parser, evaluador, unidades, símbolos y salida   │
└─────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Responsabilidad |
|---|---|
| `doc-core` | Modelo del documento, páginas, regiones, ajustes y formato `ingecalc-json`. |
| `doc-json` | Carga y guardado JSON; acepta `ingecalc-json` y el formato legado `mathdoc-json`. |
| `math-ast` | Nodos del árbol matemático. |
| `math-json` | Representación MathJSON usada por la CLI y la API. |
| `math-parse` | Parser de la sintaxis ASCII matemática del documento. |
| `math-eval` | Evaluador, entorno, variables, funciones, unidades, matrices y control de flujo. |
| `math-numeric` | Aritmética numérica, racionales, complejos y utilidades. |
| `math-symbolic` | Simplificación, expansión, factorización, derivadas, integrales y resolución. |
| `math-units` | Catálogo de unidades, dimensiones, conversiones y offsets. |
| `export` | Exportación LaTeX, ODT y PDF vectorial. |
| `plugin-api` | Contratos públicos para extensiones. |
| `plugin-host` | Carga de plugins fuera de proceso. |
| `ingecalc-cli` | CLI `ingecalc`: evaluar, validar, evaluar documentos y exportar. |
| `ingecalc-desktop` | Binario Tauri y comandos del escritorio. |

## Flujo de evaluación

1. El frontend mantiene el documento en Zustand.
2. Cada región matemática guarda su LaTeX y su versión ASCII normalizada.
3. Al calcular, el frontend invoca `evaluate_document` con el documento completo.
4. `math-eval` construye un grafo de dependencias entre variables y funciones.
5. Las definiciones se evalúan en orden topológico; los ciclos y nombres
   duplicados se reportan como error.
6. El resultado vuelve como valor tipado: real, racional, complejo, cantidad con
   unidades, texto o matriz.
7. El frontend muestra el resultado y los errores por región.

## Documento

Un `.smj` es JSON versionado. El campo `format` identifica el formato y `version`
la revisión del esquema. La extensión `.smj` se conserva para compatibilidad con
la etapa anterior del proyecto.

```json
{
  "format": "ingecalc-json",
  "version": 1,
  "metadata": { "title": "", "author": "", "language": "es" },
  "settings": {},
  "pages": []
}
```

El esquema está en [`schemas/document.schema.json`](../schemas/document.schema.json).

## Frontend

| Ruta | Contenido |
|---|---|
| `apps/desktop/src/App.tsx` | Shell, ribbon, diálogos y comandos de documento. |
| `apps/desktop/src/components/` | Página, regiones, editor de texto, campos matemáticos, gráficos y paletas. |
| `apps/desktop/src/stores/` | Estado del documento, historial, selección y reflujo. |
| `apps/desktop/src/lib/` | Geometría, evaluación, unidades, formato, exportación y utilidades. |
| `apps/desktop/src/i18n/` | Traducciones de la interfaz. |

## Exportación

- **LaTeX:** `export` genera un `.tex` con resultados ya calculados. El PDF
  vectorial compila con XeLaTeX, LuaLaTeX o pdfLaTeX.
- **ODT:** se construye un ODF con objetos Math embebidos, tablas e imágenes.
- **PDF raster:** el frontend renderiza cada página con KaTeX y html2canvas y la
  compone con jsPDF.
- **Gráficos:** se exportan como SVG/PNG desde el componente de parcela.

## Pruebas y calidad

```bash
npm run check
npm run test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

El CI de GitHub ejecuta frontend, comprobación de Tauri en Linux/Windows y la
suite Rust con `fmt` y `clippy`.
