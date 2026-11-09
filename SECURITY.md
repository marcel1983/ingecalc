# Seguridad

## Versiones soportadas

IngeCalc está en desarrollo activo. Las correcciones de seguridad se aplican a la
rama `main` y a la última versión publicada.

| Versión | Soporte |
|---|---|
| `main` | Sí |
| Última release | Sí |
| Versiones anteriores | Solo si el fallo es crítico |

## Cómo reportar una vulnerabilidad

No abras una incidencia pública para fallos de seguridad.

Usa **Security → Report a vulnerability** en GitHub para enviar un aviso privado
a los mantenedores. Incluye:

- versión o commit afectado;
- sistema operativo y versión de WebView2/WebKitGTK;
- pasos mínimos para reproducirlo;
- impacto esperado;
- prueba de concepto, si existe.

No publiques detalles hasta que exista una corrección o una fecha coordinada.

## Alcance

Son especialmente relevantes:

- ejecución de comandos o acceso a archivos desde documentos `.smj`;
- escapes del sandbox de la ventana Tauri;
- inyección de HTML/LaTeX en la exportación o en la impresión;
- carga de imágenes o plugins;
- manipulación del directorio de borradores o de los datos de WebView2.

La exportación LaTeX usa `-no-shell-escape`, pero el motor LaTeX puede leer
archivos locales. No compiles documentos de terceros sin revisarlos.

## Dependencias transitivas conocidas

- `glib 0.18` en Linux (cadena Tauri/GTK): aviso `GHSA-wrw7-89jp-8q8g`
  sobre `glib::VariantStrIter`. IngeCalc no usa ese tipo directamente. Se
  actualizará cuando Tauri migre a `glib 0.20`; mientras tanto, la alerta queda
  documentada y revisada.

## Buenas prácticas

- Mantén Rust, Node.js, Tauri y WebView2 actualizados.
- No ejecutes binarios descargados de fuentes no confiables.
- Revisa los documentos antes de exportarlos a PDF vectorial.
- Usa el reporte privado para cualquier detalle que pueda facilitar un exploit.
