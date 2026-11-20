# Changelog

Todas las versiones relevantes de IngeCalc se documentan aquí.

## [Unreleased]

Sin cambios pendientes de publicar.

## [0.1.8] - 2026-11-20

### Changed

- Un clic en una zona vacía de la hoja coloca el cursor de inserción sin crear
  una región. La región matemática se crea al empezar a escribir.
- Con el cursor sobre la hoja, `Enter` desplaza el contenido inferior hacia
  abajo y `Backspace` lo desplaza hacia arriba cuando hay espacio.
- La ayuda integrada y el manual describen el nuevo cursor y el desplazamiento
  de contenido.

## [0.1.7] - 2026-11-17

### Added

- **Exportación a Excel (`.xlsx`)** desde **Exportar → Exportar Excel** y desde
  la CLI (`ingecalc export documento.smj xlsx salida.xlsx`).
  - Hoja **Documento**: vista formateada con títulos, texto, fórmulas y
    resultados, similar al documento.
  - Hoja **Cálculos**: una fila por fórmula, con la expresión original, la
    fórmula Excel equivalente y una celda con la fórmula real.
  - Las definiciones se enlazan por referencia de celda, de modo que al editar
    un valor Excel recalcula el resto.
  - Las expresiones con unidades o funciones no traducibles conservan el valor
    calculado y su unidad.

### Changed

- La ayuda integrada (F1) y `docs/manual.md` describen la exportación a Excel,
  sus hojas y el comportamiento con unidades.
- La lista de publicación exige actualizar la ayuda y el manual antes de cada
  release.

## [0.1.6] - 2026-11-17

### Fixed

- La ayuda integrada (F1) y el manual describen ahora los archivos recientes,
  las versiones del borrador, el aviso de cambios sin guardar, el contador de
  errores, `Tab`/`Shift+Tab`, `Ctrl+Z` local en fórmulas y el zoom con
  `Ctrl` + rueda.
- Se añadió una prueba que comprueba que la ayuda en español e inglés contenga
  estas funciones.

### Changed

- La lista de publicación en `docs/publicacion-github.md` exige actualizar la
  ayuda y el manual antes de cada release.

## [0.1.5] - 2026-11-17

### Added

- Botón **▾ Archivos recientes** junto a Abrir: guarda hasta 8 documentos y
  permite volver a abrirlos con un clic.
- Historial de borradores: instantáneas cada 5 minutos (máximo 10) y menú
  **Versiones** en la pancarta de recuperación.
- Aviso **“Tienes cambios sin guardar”** con opciones Guardar / Descartar /
  Cancelar al crear un documento nuevo, abrir otro archivo o cerrar la ventana.
- Indicador **●** junto al nombre de IngeCalc cuando hay cambios sin guardar.
- Botón **Errores (N)** y contador **N con error** para ir a la siguiente
  fórmula con error.
- Navegación con `Tab` / `Shift+Tab` entre fórmulas.
- Deshacer y rehacer local dentro de cada fórmula con `Ctrl+Z` / `Ctrl+Y`.

### Changed

- `Ctrl+C` / `Ctrl+V` respetan la selección de texto dentro de párrafos; el
  portapapeles de regiones sigue funcionando cuando no hay selección de texto.
- `Ctrl` + rueda cambia el zoom; el porcentaje vuelve al 100 % con un clic.
- Se añadieron `aria-keyshortcuts` y etiquetas accesibles a los botones
  principales.

## [0.1.4] - 2026-11-16

### Changed

- Actualización de dependencias: `zip` 9.5.1, `uuid` 1.29.2, grupo `@tiptap/*`
  3.33.1, `immer` 11.2.10 y `vite` 8.3.8 (desarrollo).
- `zip` corrige `raw_copy_file` para mantener descifrable una entrada cifrada.

## [0.1.3] - 2026-11-11

### Added

- Botón 👁 junto al de fijar en la barra superior para ocultar o mostrar las
  respuestas automáticas verdes. Las regiones que terminan en `=` conservan su
  resultado.

### Changed

- Se retiró del repositorio el ejemplo `examples/dinamico-cisterna.smj`.

## [0.1.2] - 2026-11-11

### Fixed

- Valor absoluto `|x|` en el parser y normalización de `|x|`, `⌊x⌋`, `⌈x⌉`
  y `‖x‖` generados por MathLive.
- Subíndices de varias letras (`p_{y2}` → `p_y2`) al editar fórmulas.
- `log₁₀`, raíces n-ésimas, `arcsin`/`arccos`/`arctan` y sumatorios/productorios
  visuales al convertirlos desde la paleta.

## [0.1.1] - 2026-11-11

### Fixed

- Arranque seguro en Windows cuando `NahimicService` está activo: IngeCalc
  solicita UAC, detiene el servicio, abre la aplicación y lo reactiva.
- Mitigación de puntos de extensión heredados en el proceso principal.
- Documentado el conflicto con `NahimicOSD.dll` y sus alternativas.

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
