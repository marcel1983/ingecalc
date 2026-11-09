# Changelog

Todas las versiones relevantes de IngeCalc se documentan aquí.

## [Unreleased]

### Added

- Pestaña **Ayuda** con manual navegable, búsqueda y atajos.
- Botón `?` y atajo `F1` para abrir la ayuda desde cualquier parte.
- Manual de usuario completo en `docs/manual.md`.
- Guía de desarrollo, visión del proyecto y gobernanza.
- `CODEOWNERS` y plantilla de errores de cálculo.

### Changed

- Renombrado público de MathDoc a **IngeCalc**.
- Nuevos paquetes `ingecalc`, `@ingecalc/desktop`, `ingecalc-cli` y
  `ingecalc-desktop`.
- Nuevo identificador Tauri `org.ingecalc.app`.
- Variables de entorno `INGECALC_*`; se mantiene compatibilidad con `MATHDOC_*`.
- Formato de documento `ingecalc-json`; el cargador acepta `mathdoc-json`.
- Identidad visual multidisciplinar con nodos de civil, eléctrica, mecánica e
  hidráulica.
- Documentación de uso, arquitectura, publicación y contribución.

### Removed

- Ejemplo `perlin-clavador.smj` y sus exportaciones generadas.
- Referencias al ejemplo retirado en la documentación.

## [0.1.0]

- Primera base funcional del editor: documento JSON, regiones matemáticas y de
  texto, evaluación en Rust, unidades, matrices, cálculo simbólico, gráficos,
  autosave y exportación LaTeX/ODT/PDF.
