# Changelog

Todas las versiones relevantes de IngeCalc se documentan aquí.

## [Unreleased]

Sin cambios pendientes de publicar.

## [0.1.0] - 2026-11-08

Primera versión pública. Bundles disponibles para Windows x64, Linux x64
(AppImage/DEB) y macOS Apple Silicon/Intel (DMG).

### Added

- Pestaña **Ayuda** con manual navegable, búsqueda y atajos.
- Botón `?` y atajo `F1` para abrir la ayuda desde cualquier parte.
- Manual de usuario completo en `docs/manual.md`.
- Guía de desarrollo, visión del proyecto y gobernanza.
- `CODEOWNERS` y plantilla de errores de cálculo.
- Documento `.smj`, regiones matemáticas y de texto, unidades, matrices,
  cálculo simbólico, control de flujo, gráficos 2D/3D y exportación
  LaTeX/ODT/PDF.
- Autoguardado, recuperación de borrador y corrector ortográfico.
- Interfaz en español, inglés, francés, hindi, árabe y chino simplificado.

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
- Dependabot configurado con actualizaciones agrupadas.

### Fixed

- Actualizado `zip` a 9.5.0 para corregir vulnerabilidades de extracción y
  autenticación.
- Documentada la dependencia transitiva `glib 0.18` de Tauri/GTK.

### Removed

- Ejemplo `perlin-clavador.smj` y sus exportaciones generadas.
- Referencias al ejemplo retirado en la documentación.
