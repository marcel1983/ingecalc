# Publicar IngeCalc en GitHub

Esta lista de comprobación deja el repositorio listo para publicarse sin mezclar
el cambio de marca con tareas de release.

## 1. Identidad del repositorio

- **Repositorio actual:** `marcel1983/ingecalc` (cuenta personal).
- **Organización futura opcional:** `ingecalc`.
- **Descripción:** `Hojas de cálculo de ingeniería: cálculo simbólico, unidades, matrices y gráficos. Open source.`
- **Temas sugeridos:** `engineering`, `math`, `calculator`, `worksheet`, `rust`, `tauri`, `react`, `mathlive`, `units`, `symbolic-math`
- **Sitio web:** déjalo vacío hasta tener una página pública.
- **Licencia:** GPL-3.0-or-later.

Si el proyecto se traslada a una organización, actualiza las URLs de:

- `Cargo.toml` (`workspace.package.repository`)
- `package.json` (`repository`)
- Los badges del `README.md`
- `.github/ISSUE_TEMPLATE/config.yml`
- El esquema `schemas/document.schema.json`
- `CODEOWNERS` y `GOVERNANCE.md`
- Cualquier enlace absoluto de la documentación

## 2. Recursos visuales

1. Sube [`ingecalc-avatar.png`](assets/branding/ingecalc-avatar.png) como avatar
   del repositorio u organización.
2. Sube [`ingecalc-banner.png`](assets/branding/ingecalc-banner.png) en
   **Settings → General → Social preview**.
3. Mantén el banner del README apuntando a
   `docs/assets/branding/ingecalc-banner.png`.
4. Si cambias la marca, actualiza también los iconos de Tauri:

```bash
cd apps/desktop
npx tauri icon ../../docs/assets/branding/ingecalc-logo.png
```

## 3. Primer push

Si el directorio aún no es un repositorio Git:

```bash
git init -b main
git add .
git commit -m "chore: prepare IngeCalc public repository"
git remote add origin https://github.com/marcel1983/ingecalc.git
git push -u origin main
```

Ajusta la URL si el propietario u organización es distinto.

## 4. Antes del primer push

- [ ] Revisar que no queden archivos de ejemplo retirados.
- [ ] Ejecutar `npm run check`.
- [ ] Ejecutar `npm run test`.
- [ ] Ejecutar `cargo fmt --all -- --check`.
- [ ] Ejecutar `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Ejecutar `cargo test --workspace`.
- [ ] Compilar el escritorio con `npm run tauri build`.
- [ ] Revisar que `.gitignore` excluya `target/`, `node_modules/`, `dist/`,
      perfiles de WebView2 y exportaciones generadas.
- [ ] Comprobar que no se suben PDFs, ODTs, `.tex` ni imágenes temporales.

## 5. Apariencia del repositorio

- [ ] Activar **Issues**.
- [ ] Activar **Discussions** si se quiere un espacio de preguntas.
- [ ] Configurar la rama principal como `main`.
- [ ] Proteger `main`: PR obligatorio, CI en verde y al menos una revisión.
- [ ] Activar **Private vulnerability reporting** en Security.
- [ ] Revisar las plantillas de incidencias y el template de PR de `.github/`.

## 6. Release

Antes de etiquetar una versión:

1. Actualizar la versión en:
   - `Cargo.toml` (`workspace.package.version`)
   - `package.json`
   - `apps/desktop/package.json`
   - `apps/desktop/src-tauri/tauri.conf.json`
2. Actualizar `CHANGELOG.md` con los cambios de la versión.
3. Regenerar `Cargo.lock` y `package-lock.json`.
4. Ejecutar la lista de comprobación de pruebas.
5. Crear un tag anotado:

```bash
git tag -a v0.1.0 -m "IngeCalc 0.1.0"
git push origin main --tags
```

6. Adjuntar los bundles generados por Tauri para Linux y Windows.
7. Incluir `LICENSE`, `THIRD_PARTY_NOTICES.md` y las licencias de las
   dependencias en el artefacto.
8. Publicar las notas de release con:
   - cambios de la versión;
   - requisitos de WebView2/WebKitGTK;
   - formato de documento aceptado;
   - limitaciones conocidas.

## 6. Compatibilidad

- Los documentos nuevos escriben `"format": "ingecalc-json"`.
- El cargador acepta documentos antiguos con `"format": "mathdoc-json"`.
- Las variables de entorno `INGECALC_*` son las nuevas; `MATHDOC_*` se acepta
  como compatibilidad.
- El identificador Tauri es `org.ingecalc.app`. Un cambio de identificador crea
  un directorio de datos nuevo; si hay usuarios previos, conviene añadir una
  migración de borradores antes de publicar una versión estable.
