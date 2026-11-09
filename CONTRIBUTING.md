# Contribuir a IngeCalc

Gracias por tu interés. IngeCalc se distribuye bajo **GPL-3.0-or-later**; al
enviar una contribución aceptas que se publique bajo esa licencia.

## Entorno

- Rust stable.
- Node.js 24+ y npm.
- Linux con las dependencias de Tauri 2 (WebKitGTK, librsvg, patchelf).
- Windows con Rust MSVC, Visual Studio Build Tools y WebView2.

```bash
npm install
npm run check
npm run test
npm run build
npm run tauri dev
```

Para el núcleo:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Pull requests

- Un cambio por PR, con una descripción clara del problema y la solución.
- Añadir o actualizar tests cuando se modifique el parser, el evaluador, el
  documento o la exportación.
- Mantener `cargo fmt`, `clippy`, `tsc` y Vitest sin errores.
- Actualizar `docs/adr/` si se cambia una decisión de arquitectura.
- Actualizar la documentación de usuario si cambia el comportamiento visible.
- No incluir binarios, fuentes propietarias ni contenido de SMath Studio.

Usa la plantilla de PR y marca las comprobaciones ejecutadas. Si el cambio afecta
al formato `.smj`, indica si mantiene compatibilidad con `mathdoc-json`.

## Documentación

- [`README.md`](README.md): presentación del proyecto.
- [`docs/manual.md`](docs/manual.md): manual de usuario y flujo de trabajo.
- [`docs/arquitectura.md`](docs/arquitectura.md): crates y flujo de evaluación.
- [`docs/guia-desarrollo.md`](docs/guia-desarrollo.md): desarrollo, CI, release y gobernanza.
- [`docs/vision.md`](docs/vision.md): visión y criterios de éxito.
- [`GOVERNANCE.md`](GOVERNANCE.md): roles y toma de decisiones.
- [`docs/publicacion-github.md`](docs/publicacion-github.md): checklist de publicación.
- [`examples/README.md`](examples/README.md): ejemplos y resultados esperados.

## Commits

Se recomienda un estilo tipo Conventional Commits:

```text
feat(math): add exact fraction evaluation
fix(ui): keep cursor position when updating a math field
docs(adr): document JSON document format
```

## Licencia de contribuciones

No se requiere CLA. Se usa el modelo de contribución directa bajo
GPL-3.0-or-later. Si añades una dependencia, verifica que su licencia sea
compatible y actualiza `THIRD_PARTY_NOTICES.md`.
