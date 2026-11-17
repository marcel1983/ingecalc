# Plan de experiencia de usuario (UX)

Lista de tareas sugeridas a partir de la revisión del código. Cada tarea indica
prioridad, archivos probables y criterio de aceptación. El trabajo se ejecuta en
el orden de esta lista y se valida con `npm test`, `npm run check`, `cargo fmt`,
`cargo clippy -D warnings` y `cargo test --workspace`.

## Estado

| # | Tarea | Prioridad | Estado |
|---|---|---|---|
| 1 | Portapapeles en regiones de texto | Alta | Implementado |
| 2 | Archivos recientes | Alta | Implementado |
| 3 | Historial de borradores (versiones) | Media | Implementado |
| 4 | Confirmar al cerrar con cambios | Media | Implementado |
| 5 | Contador y navegación de fórmulas con error | Media | Implementado |
| 6 | Navegación entre regiones con Tab/Shift+Tab | Media | Implementado |
| 7 | Deshacer local en fórmulas | Media | Implementado |
| 8 | Zoom con Ctrl+rueda y restablecer con clic | Baja | Implementado |
| 9 | Accesibilidad: aria-keyshortcuts y foco visible | Baja | Implementado parcialmente |
| 10 | Documentación y release | Baja | En progreso (v0.1.5) |

## Detalle

### 1. Portapapeles en regiones de texto

- **Problema:** `App.tsx` desactiva Ctrl+C/X/V cuando el foco está en una
  región editable. En TipTap esto impide que el usuario copie texto seleccionado
  dentro de un párrafo.
- **Cambio:** solo interceptar los atajos de regiones cuando no hay selección de
  texto activa. TipTap debe conservar su portapapeles nativo.
- **Criterio:** seleccionar texto dentro de una región y copiar/pegar funciona;
  copiar cajas seleccionadas sigue funcionando.

### 2. Archivos recientes

- **Problema:** no hay memoria de documentos abiertos.
- **Cambio:** guardar hasta 8 rutas `.smj` en `localStorage`
  (`ingecalc-recent-files`), mostrarlas en el menú de la pestaña Inicio y
  permitir abrirlas con un clic. Al abrir desde la lista, quitar duplicados y
  poner la ruta más reciente al inicio.
- **Criterio:** abrir varios archivos, cerrar la app, volver a abrir y ver la
  lista; cada entrada carga el documento.

### 3. Historial de borradores

- **Problema:** solo existe `draft.smj`; se sobrescribe cada 1,2 s.
- **Cambio:** en `draftHistory.ts`, además del borrador actual, guardar
  instantáneas con marca de tiempo cada 5 minutos, máximo 10 copias en
  `localStorage` (`ingecalc-draft-history-v1`). En la pancarta de recuperación,
  ofrecer “Versiones” con la lista de copias.
- **Criterio:** tras varios cambios, aparecen instantáneas; se puede recuperar
  una versión y el documento carga con ese contenido.

### 4. Confirmar al cerrar con cambios

- **Problema:** `beforeunload` guarda el borrador pero no avisa.
- **Cambio:** mostrar un aviso en la barra superior **“Tienes cambios sin
  guardar”** con opciones **Guardar / Descartar / Cancelar** al crear un
  documento nuevo, abrir otro archivo o cerrar la ventana. Añadir un indicador
  `●` junto al nombre cuando hay cambios.
- **Criterio:** al cerrar o abrir otro archivo con cambios aparece el aviso;
  cancelar mantiene el documento; descartar ejecuta la acción; guardar escribe
  el archivo y luego continúa.

### 5. Contador y navegación de fórmulas con error

- **Problema:** los errores se ven en rojo por región, pero no hay resumen.
- **Cambio:** mostrar en la barra de estado `N con error` y añadir el botón
  **Errores (N)** en la pestaña Fórmulas para enfocar la siguiente región con
  error. Añadir un botón `●` junto al nombre del documento cuando hay cambios.
- **Criterio:** con un documento que tiene errores, el contador es correcto y
  lleva a la región; al corregir todos, desaparece.

### 6. Navegación entre regiones con Tab

- **Problema:** solo `Enter` crea una región debajo.
- **Cambio:** `Tab` dentro de una región matemática enfoca la siguiente región
  matemática; `Shift+Tab` enfoca la anterior. No debe interferir con el
  autocompletado ni con la navegación del teclado virtual.
- **Criterio:** recorrer una página solo con Tab y Shift+Tab; el cursor llega a
  cada campo en orden.

### 7. Deshacer local en fórmulas

- **Problema:** el deshacer global puede deshacer cambios de toda la página y
  no hay deshacer por campo.
- **Cambio:** integrar `Ctrl+Z` dentro de `MathField` usando el historial
  interno de MathLive (`element.undo()` / `element.redo()`), y evitar que el
  atajo global se dispare cuando el foco está en la fórmula.
- **Criterio:** editar una fórmula, deshacer y rehacer solo afecta a esa
  fórmula; el historial global sigue intacto.

### 8. Zoom con Ctrl+rueda

- **Problema:** solo hay botones y `Ctrl +/-`.
- **Cambio:** en `Worksheet`, escuchar `wheel` con `ctrlKey` y ajustar el zoom;
  hacer clic en el porcentaje restablece al 100%.
- **Criterio:** Ctrl+rueda cambia el zoom; el porcentaje vuelve a 100% con clic;
  la página no se desplaza al hacer zoom.

### 9. Accesibilidad

- **Cambio:** añadir `aria-keyshortcuts` a los botones principales, foco
  visible y respetar `prefers-reduced-motion` en transiciones.
- **Criterio:** recorrer la barra con Tab muestra foco; los atajos se anuncian;
  sin animaciones si el sistema lo pide.

### 10. Documentación y release

- Actualizar `docs/manual.md`, `docs/guia-desarrollo.md` y `CHANGELOG.md`.
- Ejecutar todas las validaciones.
- Crear `v0.1.5`, esperar el workflow y publicar con `SHA256SUMS.txt`.