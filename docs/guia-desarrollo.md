# Guía de desarrollo de IngeCalc

> Documento base para organizar, publicar y mantener el repositorio de IngeCalc
> en GitHub. Describe el producto actual, el stack real, las reglas de
> contribución, la validación técnica y el proceso de release.
>
> **Versión del proyecto:** 0.1.3  
> **Última actualización:** 8 de noviembre de 2026  
> **Licencia:** GPL-3.0-or-later

## Contenido

1. [Visión y alcance](#1-visión-y-alcance)
2. [Estado actual](#2-estado-actual)
3. [Stack y estructura](#3-stack-y-estructura)
4. [Configuración inicial en GitHub](#4-configuración-inicial-en-github)
5. [Gobernanza y archivos obligatorios](#5-gobernanza-y-archivos-obligatorios)
6. [Flujo de trabajo con Git](#6-flujo-de-trabajo-con-git)
7. [Issues, etiquetas y planificación](#7-issues-etiquetas-y-planificación)
8. [Pull Requests y revisión](#8-pull-requests-y-revisión)
9. [Estándares de código](#9-estándares-de-código)
10. [Pruebas y validación técnica](#10-pruebas-y-validación-técnica)
11. [Integración continua](#11-integración-continua)
12. [Versionado y publicación](#12-versionado-y-publicación)
13. [Documentación](#13-documentación)
14. [Comunidad y gobernanza](#14-comunidad-y-gobernanza)
15. [Seguridad](#15-seguridad)
16. [Hoja de ruta](#16-hoja-de-ruta)
17. [Checklist de publicación](#17-checklist-de-publicación)
18. [Anexos](#18-anexos)

---

## 1. Visión y alcance

IngeCalc es un editor open source de **hojas de cálculo técnico** para ingeniería
civil, eléctrica, mecánica, hidráulica y otras disciplinas. Combina, en un único
documento:

- matemática numérica y simbólica;
- unidades y conversiones;
- matrices y vectores;
- gráficos 2D y superficies 3D;
- texto enriquecido y documentación;
- exportación a formatos abiertos.

No es una calculadora aislada ni una hoja de cálculo de propósito general. El
producto está pensado para **memorias de cálculo, comprobaciones y documentos
técnicos reproducibles**.

### Principios de producto

1. **Local-first:** el documento vive en el equipo del usuario; no hay cuentas ni
   servicios obligatorios.
2. **Formato abierto:** JSON versionado con extensión `.smj`.
3. **Cálculo trazable:** las definiciones se evalúan por dependencias y cada
   nombre tiene una sola definición.
4. **Entrada literal:** el editor sugiere, pero no autocorrige identificadores.
5. **Multidisciplinar:** la misma aplicación sirve para estructuras, circuitos,
   máquinas y fluidos.
6. **Exportación real:** LaTeX, ODT, PDF raster y PDF vectorial.
7. **Documentación integrada:** manual navegable dentro de la aplicación (`F1`).

### Plataformas objetivo

- Linux: objetivo principal y cubierto por CI.
- Windows: compilación y ejecución verificadas.
- macOS: bundles DMG para Apple Silicon e Intel; falta validar la ejecución nativa.

### Fuera del alcance de la primera publicación

- colaboración en la nube en tiempo real;
- CAS simbólico completo;
- marketplace de plugins;
- bibliotecas de perfiles y normas de ingeniería integradas;
- edición móvil.

---

## 2. Estado actual

IngeCalc es un **MVP funcional**, no un scaffold vacío.

| Área | Estado |
|---|---|
| Núcleo Rust | Evaluador, documento, parser, unidades, matrices, símbolo y exportación |
| Escritorio | Tauri 2 + React + MathLive + KaTeX + TipTap |
| Documento | `.smj` JSON, formato `ingecalc-json`, versión 1 |
| Compatibilidad | Carga documentos antiguos `mathdoc-json` y los migra al guardar |
| Cálculo | Reales, racionales, complejos, cantidades, texto y matrices |
| Unidades | SI, imperiales, localizadas, propias y conversión dimensional |
| Simbólico | `simplify`, `expand`, `factor`, `subs`, `diff`, `integrate`, `solve`, `roots`, `polyroots` |
| Control de flujo | `if`, `for`, `sum`, `product`, `while`, `line`, `trace`, `error` |
| Gráficos | 2D explícitos, paramétricos, implícitos y superficies 3D |
| Exportación | LaTeX, ODT, PDF raster y PDF vectorial |
| Ayuda | Pestaña **Ayuda**, manual integrado, búsqueda y `F1` |
| Idiomas de interfaz | Español, inglés, francés, hindi, árabe y chino simplificado |
| Correctores | Hunspell para español, inglés, francés, hindi y árabe |
| Tests frontend | 107 pruebas Vitest |
| Tests Rust | 94 pruebas |
| CI | Frontend, Tauri Linux/Windows y workspace Rust |
| Build Tauri | Verificado en Windows |

### Limitaciones conocidas

- Una definición por nombre; sin ámbitos anidados ni recursión.
- Matrices numéricas `f64`; no admiten cantidades heterogéneas.
- Unidades propias lineales, sin offsets.
- Racionales con enteros de 64 bits.
- Complejos con funciones básicas.
- Motor simbólico acotado, no un CAS completo.
- LaTeX/ODT conservan contenido y orden, pero no posiciones absolutas.
- La matemática inline en ODT se exporta como texto ASCII por compatibilidad.
- El PDF raster no tiene texto seleccionable.
- El desbordamiento de texto mueve la caja completa; falta el corte por párrafos.
- macOS: falta validar la ejecución nativa de los DMG; los bundles se generan en CI.

---

## 3. Stack y estructura

### Stack

| Capa | Tecnología |
|---|---|
| Núcleo | Rust estable, edición 2021 |
| Escritorio | Tauri 2 |
| Interfaz | React 19 + TypeScript |
| Build frontend | Vite |
| Matemática | MathLive + KaTeX |
| Texto rico | TipTap / ProseMirror |
| Estado | Zustand + Immer |
| Documento | JSON versionado `.smj` |
| Exportación | LaTeX, ODF, jsPDF, html2canvas |
| Pruebas | Vitest, `cargo test` |
| CI | GitHub Actions |

### Estructura del repositorio

```text
.github/                 Workflows, plantillas de issues y PR
apps/
  desktop/               Tauri + React
    src/                 Componentes, stores, lib, tipos
    src-tauri/           Backend Tauri y comandos
  cli/                   CLI ingecalc
crates/                  Núcleo Rust
  doc-core/              Modelo del documento
  doc-json/              Carga y guardado
  math-ast/              Árbol matemático
  math-json/             MathJSON
  math-parse/            Parser ASCII
  math-eval/             Evaluador
  math-numeric/          Aritmética
  math-symbolic/         Motor simbólico
  math-units/            Unidades
  export/                LaTeX, ODT y PDF vectorial
  plugin-api/            Contratos de plugins
  plugin-host/           Carga fuera de proceso
docs/                    Manual, arquitectura, ADRs y publicación
examples/                Documentos .smj de ejemplo
schemas/                 JSON Schema del documento
scripts/                 Utilidades de build
```

### Crates principales

| Crate | Responsabilidad |
|---|---|
| `doc-core` | Documento, páginas, regiones, ajustes y formato |
| `doc-json` | Persistencia y compatibilidad de formato |
| `math-ast` | Nodos del árbol matemático |
| `math-json` | Representación MathJSON |
| `math-parse` | Parser de la sintaxis del documento |
| `math-eval` | Evaluación, entorno, variables, funciones y control de flujo |
| `math-numeric` | Reales, racionales, complejos y utilidades |
| `math-symbolic` | Transformaciones simbólicas |
| `math-units` | Dimensiones, conversiones y offsets |
| `export` | Salida LaTeX, ODT y PDF vectorial |
| `plugin-api` | API pública de extensiones |
| `plugin-host` | Ejecución de plugins |
| `ingecalc-cli` | CLI de evaluación, validación y exportación |
| `ingecalc-desktop` | Binario Tauri |

### Comandos de desarrollo

```bash
npm install
npm run tauri dev
npm run dev
npm run tauri build
```

```bash
npm run check
npm run test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Variables de entorno

| Variable | Uso |
|---|---|
| `INGECALC_WEBVIEW_DATA_DIR` | Directorio de datos de WebView2 |
| `INGECALC_DRAFT_DIR` | Directorio del borrador autosave |
| `INGECALC_LATEX_ENGINE` | Ruta al ejecutable de LaTeX |

Las variables antiguas `MATHDOC_*` se aceptan como compatibilidad.

---

## 4. Configuración inicial en GitHub

### 4.1 Cuenta u organización

El repositorio actual está en **github.com/marcel1983/ingecalc** (cuenta
personal). Si el proyecto crece, se puede crear `github.com/ingecalc` como
organización y transferir el repositorio; el código y el historial se conservan.

Repositorios sugeridos:

| Repositorio | Propósito |
|---|---|
| `ingecalc` | Código principal y monorepo |
| `ingecalc-examples` | Hojas de ejemplo y validación (opcional) |
| `ingecalc-docs` | Sitio de documentación (opcional) |
| `.github` | Plantillas y perfil de la organización |

### 4.2 Identidad del repositorio

- **Repositorio:** `marcel1983/ingecalc`
- **Nombre:** `ingecalc`
- **Descripción:** `Hojas de cálculo de ingeniería: cálculo simbólico, unidades, matrices y gráficos. Open source.`
- **Temas:** `engineering`, `math`, `calculator`, `worksheet`, `rust`, `tauri`, `react`, `mathlive`, `units`, `symbolic-math`
- **Licencia:** GPL-3.0-or-later
- **Web:** vacío hasta tener una página pública
- **Avatar:** `docs/assets/branding/ingecalc-avatar.png`
- **Social preview:** `docs/assets/branding/ingecalc-banner.png`

Si el proyecto se traslada a otra cuenta u organización, actualizar:

- `Cargo.toml` (`workspace.package.repository`);
- `package.json` (`repository`);
- badges del `README.md`;
- URLs absolutas de `.github/ISSUE_TEMPLATE/config.yml`;
- `schemas/document.schema.json`;
- `CODEOWNERS` y `GOVERNANCE.md`;
- enlaces de `SECURITY.md` y documentación.

### 4.3 Equipos y permisos

| Equipo | Permiso | Responsabilidad |
|---|---|---|
| `maintainers` | Admin | Releases, seguridad y decisiones finales |
| `core` | Maintain | Crates Rust, parser, evaluador y formato |
| `gui` | Write | React, Tauri, accesibilidad y experiencia de usuario |
| `docs` | Write | Manual, guías, traducciones y ADRs |
| `engineering` | Write | Casos de validación y revisión técnica |
| `triage` | Triage | Clasificar issues y reproducir errores |

Mientras el repositorio esté en la cuenta personal `@marcel1983`, los roles son
organizativos y `CODEOWNERS` apunta al mantenedor. Al migrar a una organización,
se crearán los equipos y se actualizará `CODEOWNERS`.

### 4.4 Protección de `main`

En **Settings → Branches → Rulesets**:

- [x] Requerir Pull Request antes de fusionar.
- [x] Requerir al menos 1 aprobación; 2 para `crates/` y `src-tauri/`.
- [x] Descartar aprobaciones obsoletas.
- [x] Requerir revisión de `CODEOWNERS`.
- [x] Requerir CI en verde.
- [x] Requerir rama actualizada.
- [x] Resolver conversaciones.
- [x] Squash merge únicamente.
- [x] Bloquear force push y borrado.
- [x] Recomendar commits firmados.

### 4.5 Opciones del repositorio

- Activar **Issues**, **Discussions** y **Projects**.
- Desactivar Wiki: la documentación vive en `docs/`.
- Activar **Automatically delete head branches**.
- Activar **Dependabot alerts**, **Dependabot security updates** y **secret scanning**.
- Activar **Private vulnerability reporting**.
- Configurar **CodeQL** para Rust y JavaScript/TypeScript.

### 4.6 Primer push

Si el directorio aún no es un repositorio Git:

```bash
git init -b main
git add .
git commit -m "chore: prepare IngeCalc public repository"
git remote add origin https://github.com/marcel1983/ingecalc.git
git push -u origin main
```

---

## 5. Gobernanza y archivos obligatorios

| Archivo | Estado | Contenido |
|---|---|---|
| `README.md` | Existe | Presentación, capturas, funciones, instalación y enlaces |
| `LICENSE` | Existe | GPL-3.0-or-later |
| `CONTRIBUTING.md` | Existe | Entorno, PRs, tests y documentación |
| `CODE_OF_CONDUCT.md` | Existe | Contributor Covenant 2.1 |
| `SECURITY.md` | Existe | Reporte privado y alcance |
| `CHANGELOG.md` | Existe | Keep a Changelog |
| `GOVERNANCE.md` | Existe | Roles, decisiones y RFC |
| `CODEOWNERS` | Existe | Revisores por ruta |
| `docs/vision.md` | Existe | Visión, usuarios y criterios de éxito |
| `docs/manual.md` | Existe | Manual de usuario |
| `docs/arquitectura.md` | Existe | Arquitectura técnica |
| `docs/guia-desarrollo.md` | Existe | Este documento |
| `docs/publicacion-github.md` | Existe | Checklist de publicación |
| `.github/PULL_REQUEST_TEMPLATE.md` | Existe | Checklist de PR |
| `.github/ISSUE_TEMPLATE/*.yml` | Existe | Bug, feature y error de cálculo |

### Licencia

Se usa **GPL-3.0-or-later** para la aplicación y el núcleo. Cualquier dependencia
nueva debe ser compatible y quedar reflejada en `THIRD_PARTY_NOTICES.md`.

No se requiere CLA. Se usa el modelo de contribución directa y DCO
(`git commit -s`).

### `CODEOWNERS`

```text
# Por defecto
*                           @marcel1983

# Núcleo matemático y documento
/crates/math-*/             @marcel1983
/crates/doc-*/              @marcel1983
/crates/export/             @marcel1983
/crates/plugin-*/           @marcel1983
/schemas/                   @marcel1983

# Escritorio y CLI
/apps/desktop/src/          @marcel1983
/apps/desktop/src-tauri/    @marcel1983
/apps/cli/                  @marcel1983

# Validación y ejemplos
/examples/                  @marcel1983

# Documentación
/docs/                      @marcel1983
/README.md                  @marcel1983
/CONTRIBUTING.md            @marcel1983
/SECURITY.md                @marcel1983
/CODE_OF_CONDUCT.md         @marcel1983
/GOVERNANCE.md              @marcel1983
/CHANGELOG.md               @marcel1983

# Infraestructura
/.github/                   @marcel1983
/Cargo.toml                 @marcel1983
/package.json               @marcel1983
```

---

## 6. Flujo de trabajo con Git

### 6.1 Modelo de ramas

```text
main ──●──────●──────●──────●───▶  (siempre estable)
        \    /  \    /
         feat/   fix/
```

| Rama | Uso | Ejemplo |
|---|---|---|
| `main` | Código integrado y probado | — |
| `feat/<issue>-<desc>` | Nueva funcionalidad | `feat/42-plot-3d` |
| `fix/<issue>-<desc>` | Corrección | `fix/57-unit-conversion` |
| `docs/<desc>` | Documentación | `docs/manual-atajos` |
| `refactor/<desc>` | Reestructuración | `refactor/parser-errors` |
| `release/X.Y` | Mantenimiento de versión | `release/0.2` |

Reglas:

- Nadie hace push directo a `main`.
- Cada rama parte de un issue.
- Ramas cortas: menos de una semana y menos de 400 líneas cuando sea posible.

### 6.2 Contribuidores externos

```bash
git clone https://github.com/<usuario>/ingecalc.git
cd ingecalc
git remote add upstream https://github.com/marcel1983/ingecalc.git
git fetch upstream
git switch -c feat/42-plot-3d upstream/main
git commit -s -m "feat(plots): add surface rotation controls"
git push origin feat/42-plot-3d
```

### 6.3 Conventional Commits

```text
<tipo>(<ámbito>): <descripción en imperativo>

[cuerpo opcional]

[Closes #42]
Signed-off-by: Nombre <correo>
```

Tipos: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.

Ámbitos: `core`, `parser`, `eval`, `units`, `matrices`, `symbolic`, `document`,
`gui`, `export`, `cli`, `plugins`, `docs`, `ci`.

Ejemplos:

```text
feat(units): add pressure offset conversion
fix(parser): preserve literal identifiers
docs(manual): add troubleshooting section
test(eval): cover nested conditional expressions
```

---

## 7. Issues, etiquetas y planificación

### 7.1 Plantillas

Existen plantillas para:

- `bug_report.yml`: fallo reproducible, entorno, logs y versión.
- `feature_request.yml`: problema, propuesta, alternativas y área.
- `calculation_error.yml`: resultado incorrecto, valor esperado, fuente de
  validación y archivo `.smj`.
- `config.yml`: enlaces a manual, seguridad y contribución.

Los errores de cálculo son **prioridad crítica**. Toda corrección debe incluir una
prueba de regresión.

### 7.2 Etiquetas

| Grupo | Etiquetas |
|---|---|
| Tipo | `bug`, `feature`, `enhancement`, `docs`, `refactor`, `calculation` |
| Área | `area: core`, `area: parser`, `area: eval`, `area: units`, `area: gui`, `area: export`, `area: docs` |
| Prioridad | `priority: critical`, `priority: high`, `priority: medium`, `priority: low` |
| Estado | `status: triage`, `status: confirmed`, `status: blocked`, `status: needs-info` |
| Comunidad | `good first issue`, `help wanted`, `discussion` |
| Validación | `validation`, `norma`, `unit-test` |

### 7.3 Projects y milestones

- Columnas: **Triage → Backlog → Listo → En progreso → En revisión → Hecho**.
- Campos: prioridad, área, estimación, iteración, versión objetivo.
- Un milestone por versión (`v0.1.0`, `v0.2.0`, …).
- Triage semanal: clasificar, pedir información, cerrar duplicados y marcar
  `good first issue`.

---

## 8. Pull Requests y revisión

### 8.1 Plantilla de PR

La plantilla actual incluye:

- resumen del cambio;
- tipo de cambio;
- comprobaciones `npm run check`, `npm run test`, `cargo fmt`, `clippy` y
  `cargo test`;
- tests añadidos;
- documentación actualizada;
- confirmación de que no se incluyen binarios ni contenido propietario.

### 8.2 Reglas de revisión

| Regla | Detalle |
|---|---|
| Aprobaciones | 1 general; 2 en `crates/`, `src-tauri/` y cambios de formato |
| Primera respuesta | Menos de 3 días hábiles |
| Tamaño | Menos de 400 líneas recomendado |
| Autor | No fusiona su propio PR sin aprobación |
| Tono | Comentarios sobre el código, no sobre la persona |

Prefijos:

- `blocking:` debe corregirse antes de fusionar.
- `sugerencia:` mejora opcional.
- `nit:` detalle menor.
- `pregunta:` aclaración.

### 8.3 Checklist del revisor

- [ ] ¿Resuelve el issue sin añadir alcance extra?
- [ ] ¿Mantiene la separación núcleo/UI?
- [ ] ¿Las unidades se propagan correctamente?
- [ ] ¿Hay tests para cero, negativos, incompatibilidades y límites?
- [ ] ¿Se conserva la compatibilidad de `.smj`?
- [ ] ¿La entrada matemática sigue siendo literal?
- [ ] ¿Se actualizó el manual o la ayuda integrada?
- [ ] ¿El cambio afecta resultados numéricos? ¿Incluye validación?

---

## 9. Estándares de código

### 9.1 Rust

- Edición 2021, formato con `cargo fmt`.
- Clippy sin warnings: `cargo clippy --workspace --all-targets -- -D warnings`.
- Errores con `thiserror` o tipos propios; evitar `unwrap`/`expect` en rutas de
  ejecución.
- El núcleo no depende de Tauri ni de React.
- Los comandos Tauri devuelven `Result` y traducen errores a mensajes claros.
- Los cambios de formato deben aceptar documentos antiguos cuando sea posible.
- Documentar decisiones de arquitectura en `docs/adr/`.
- No introducir `eval` de lenguajes ni ejecución de shell desde documentos.

### 9.2 TypeScript y React

- `strict: true` en TypeScript; `npm run check` debe pasar.
- Componentes funcionales y estado en Zustand cuando sea global.
- Evitar lógica de cálculo en el frontend: el documento se evalúa en Rust.
- Mantener las claves de i18n para toda cadena visible.
- Respetar la entrada literal de MathLive y el autocompletado sin autocorrección.
- Los diálogos deben cerrar con `Escape` y respetar `aria-modal`.
- Reutilizar los estilos y patrones de diálogo existentes.

### 9.3 Documento y compatibilidad

- Formato actual: `ingecalc-json`, versión `1`.
- El cargador acepta `mathdoc-json` y migra al guardar.
- No cambiar el significado de campos existentes sin una versión nueva.
- Toda migración debe tener prueba de carga.
- El esquema vive en `schemas/document.schema.json`.

### 9.4 Idioma

- Código, nombres y comentarios técnicos: inglés.
- Documentación de usuario: español e inglés.
- Mensajes de interfaz: a través de `src/lib/i18n.ts`.
- Manual integrado: español e inglés; la interfaz de ayuda usa los seis idiomas.

---

## 10. Pruebas y validación técnica

### 10.1 Suites actuales

| Suite | Comando | Cobertura |
|---|---|---|
| Frontend | `npm run test` | Geometría, store, documento, paletas, exportación, i18n, manual y UI |
| TypeScript | `npm run check` | Tipado estricto |
| Rust | `cargo test --workspace` | Parser, evaluador, unidades, símbolos, documento, export y CLI |
| Formato | `cargo fmt --all -- --check` | Estilo Rust |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | Calidad Rust |

Estado de referencia: **107 pruebas frontend y 94 pruebas Rust**.

### 10.2 Niveles de prueba

| Nivel | Dónde | Qué verifica |
|---|---|---|
| Unitaria | `crates/*/src` y `src/lib/*.test.ts` | Funciones aisladas |
| Integración | `crates/doc-json`, `export`, `apps/cli` | Documento completo, carga, guardado y exportación |
| Regresión | Tests junto al módulo corregido | Un caso por bug de cálculo |
| Validación | `examples/` y futuros `tests/validation` | Resultados contra fuentes publicadas |
| UI | Vitest y pruebas manuales Tauri | Geometría, diálogos, ayuda y estados |

### 10.3 Cómo añadir una prueba Rust

- Añadir el test en el módulo correspondiente con `#[cfg(test)]`.
- Usar `Document::default()` o un `.smj` de `examples/`.
- Para resultados numéricos, comparar con tolerancia relativa o absoluta.
- Para unidades, comprobar valor y dimensión, no solo el número.
- Para formato, probar carga de `ingecalc-json` y `mathdoc-json`.

### 10.4 Cómo añadir una prueba frontend

- Crear o ampliar `*.test.ts` junto al módulo.
- Usar Vitest.
- Evitar depender de Tauri en tests unitarios; usar funciones puras.
- Para el manual, probar búsqueda, idioma y secciones.
- Para geometría, cubrir solapamientos, márgenes y reflujo.

### 10.5 Validación de ingeniería

Los casos de validación deben documentar:

- norma, edición y ecuación;
- datos de entrada y unidades;
- resultado esperado con tolerancia;
- archivo `.smj` reproducible;
- fuente del valor esperado.

Ejemplo futuro:

```text
examples/validation/aisc-E1A.smj
Resultado esperado: φPn = 1130 kip (LRFD)
Fuente: AISC Design Examples v16.0, Example E.1A
Tolerancia: 0,5 %
```

No se deben incluir tablas, fuentes ni contenido propietario de SMath Studio.

### 10.6 Política de errores de cálculo

1. Etiquetar `calculation` y `priority: critical`.
2. Reproducir con un `.smj` mínimo.
3. Escribir una prueba de regresión que falle.
4. Corregir y publicar un patch release.
5. Anunciar versiones afectadas en el CHANGELOG y Discussions.

---

## 11. Integración continua

### 11.1 Workflow actual

`.github/workflows/ci.yml` ejecuta:

| Job | Contenido |
|---|---|
| `frontend` | `npm ci`, `check`, `test`, `build` |
| `desktop` | Linux y Windows, `npm ci`, build y `cargo check -p ingecalc-desktop` |
| `rust` | `rustfmt`, Clippy del workspace y `cargo test --workspace` |

El job Rust instala dependencias de Tauri y construye el frontend antes de
compilar el escritorio.

### 11.2 Release y workflows recomendados

| Workflow | Estado | Disparador | Acción |
|---|---|---|---|
| `release.yml` | Activo | Tag `v*` | Bundles Tauri para Windows, Linux y macOS; release en borrador |
| `docs.yml` | Pendiente | Push a `main` | Sitio de documentación o verificación de enlaces |
| `codeql.yml` | Pendiente | Semanal y PR | Análisis de seguridad |
| `stale.yml` | Pendiente | Diario | Marcar issues inactivos, nunca `calculation` |

El workflow de release genera:

- Windows x64: instalador NSIS `.exe`.
- Linux x64: AppImage y `.deb`.
- macOS Apple Silicon: `.dmg`.
- macOS Intel: `.dmg`.

### 11.3 Dependabot

```yaml
version: 2
updates:
  - package-ecosystem: npm
    directory: "/"
    schedule: { interval: weekly }
  - package-ecosystem: cargo
    directory: "/"
    schedule: { interval: weekly }
  - package-ecosystem: github-actions
    directory: "/"
    schedule: { interval: monthly }
```

---

## 12. Versionado y publicación

### 12.1 Semantic Versioning

| Cambio | Versión |
|---|---|
| Corrección compatible | `0.1.0` → `0.1.1` |
| Funcionalidad compatible | `0.1.0` → `0.2.0` |
| Cambio incompatible de API o formato | `0.1.0` → `1.0.0` |

Antes de `1.0.0`, la API y el formato pueden cambiar, pero toda ruptura debe
documentarse y, si es posible, incluir migrador.

### 12.2 Fuentes de versión

Actualizar siempre:

- `Cargo.toml` (`workspace.package.version`);
- `package.json`;
- `apps/desktop/package.json`;
- `apps/desktop/src-tauri/tauri.conf.json`;
- `CHANGELOG.md`.

### 12.3 Proceso de release

1. Congelar el milestone.
2. Crear `release/X.Y` si hay mantenimiento.
3. Actualizar versiones y CHANGELOG.
4. Ejecutar `npm run check`, `npm run test`, `cargo fmt`, Clippy y tests.
5. Compilar con `npm run tauri build`.
6. Crear tag anotado:

```bash
git tag -a v0.1.3 -m "IngeCalc 0.1.3"
git push origin main --tags
```

7. Adjuntar bundles de Linux y Windows.
8. Incluir `LICENSE`, `THIRD_PARTY_NOTICES.md` y avisos de dependencias.
9. Publicar notas con cambios, requisitos y limitaciones.

### 12.4 Soporte

- La última versión menor recibe todas las correcciones.
- La anterior recibe errores de cálculo y seguridad durante 6 meses.
- Los errores de cálculo críticos pueden motivar un patch inmediato.

### 12.5 Formato de documento

El formato `.smj` tiene su propia versión (`1`), independiente de la versión de
la aplicación. Los documentos nuevos escriben `ingecalc-json`; los antiguos
`mathdoc-json` se aceptan y migran al guardar.

---

## 13. Documentación

La fuente de verdad son los Markdown del repositorio:

| Documento | Público | Contenido |
|---|---|---|
| `README.md` | Usuarios | Presentación, capturas, funciones e instalación |
| `docs/manual.md` | Usuarios | Manual completo y sintaxis |
| `docs/arquitectura.md` | Contribuidores | Crates, frontend y flujo de evaluación |
| `docs/guia-desarrollo.md` | Contribuidores | Este documento |
| `docs/publicacion-github.md` | Mantenedores | Checklist de publicación |
| `docs/analisis-funcional-smath.md` | Producto | Comparación y plan |
| `docs/adr/` | Mantenedores | Decisiones de arquitectura |
| `docs/vision.md` | Comunidad | Visión y criterios de éxito |
| Ayuda integrada (`F1`) | Usuarios | Manual navegable dentro de la app |

Reglas:

- Un PR que cambia comportamiento visible debe actualizar README, manual o ayuda.
- Las capturas viven en `docs/assets/screenshots/`.
- Los recursos de marca viven en `docs/assets/branding/`.
- Los enlaces relativos deben comprobarse antes del PR.
- El sitio de documentación es opcional; la fuente principal es Markdown.

---

## 14. Comunidad y gobernanza

### 14.1 Modelo

Al inicio: **BDFL con consejo**. El fundador decide y consulta a los
mantenedores. Al alcanzar cinco mantenedores activos, se puede pasar a un comité
técnico con votación.

### 14.2 Roles

| Rol | Cómo se obtiene | Permisos |
|---|---|---|
| Usuario | — | Issues y Discussions |
| Contribuidor | 1 PR fusionado | Reconocimiento en `CONTRIBUTORS` |
| Triager | Participación constante | Etiquetar y clasificar |
| Revisor | 5 PRs de calidad en un área | Aprobar PRs de su área |
| Mantenedor | Propuesta y votación | Fusionar y publicar |

### 14.3 RFC

Para cambios de formato, API de plugins o arquitectura:

1. Abrir una Discussion **RFC**.
2. Periodo mínimo de comentarios: 14 días.
3. Decisión registrada como ADR.

### 14.4 Canales

| Canal | Uso |
|---|---|
| GitHub Issues | Errores y tareas concretas |
| GitHub Discussions | Preguntas, ideas, RFC y anuncios |
| Discord/Matrix | Conversación opcional en tiempo real |
| Reunión mensual | Planificación y actas en Discussions |

### 14.5 Atraer contribuidores

- Mantener de 5 a 10 issues `good first issue`.
- Responder rápido a los primeros PRs.
- Reconocer documentación, validación y traducción, no solo código.
- Validar normas con ingenieros aunque no programen.

---

## 15. Seguridad

### 15.1 Riesgos principales

- Documentos `.smj` maliciosos con expresiones o recursos.
- Inyección de HTML/LaTeX en la exportación o impresión.
- Plugins de terceros.
- Acceso a archivos locales desde imágenes o LaTeX.
- Perfiles de WebView2 y borradores.
- Dependencias desactualizadas.

### 15.2 Controles actuales

- El evaluador usa parser y AST propios; no ejecuta código arbitrario.
- La exportación LaTeX usa `-no-shell-escape`.
- Los plugins se cargan fuera de proceso.
- `SECURITY.md` describe el reporte privado.
- El identificador Tauri es `org.ingecalc.app`.
- Las variables de entorno y rutas de datos son configurables.

### 15.3 Recomendaciones

- Activar CodeQL, Dependabot y secret scanning.
- Firmar instaladores y publicar checksums.
- Revisar dependencias nuevas y actualizar `THIRD_PARTY_NOTICES.md`.
- No compilar documentos de terceros sin revisarlos.
- No publicar detalles de vulnerabilidades antes de una corrección.

---

## 16. Hoja de ruta

| Versión | Objetivo | Contenido principal |
|---|---|---|
| **v0.1** — MVP | Producto usable | Documento, cálculo, unidades, matrices, símbolos, gráficos, exportación, ayuda |
| **v0.2** — Estabilidad | Calidad | Más validación, migración de borradores, firma de instaladores, documentación web |
| **v0.3** — Ingeniería | Profundidad | Paquetes de validación, más unidades y funciones, mejores gráficos |
| **v0.4** — Ecosistema | Extensibilidad | Plugins, catálogos de perfiles y normas |
| **v1.0** — Estable | Producción | API y formato estables, instaladores firmados, validación completa |

Las fechas no son compromisos; los milestones se ajustan con el trabajo real.

---

## 17. Checklist de publicación

### 17.1 Antes del primer push

- [x] README con banner, capturas y funciones.
- [x] Licencia GPL-3.0-or-later.
- [x] Manual de usuario.
- [x] Ayuda integrada con `F1`.
- [x] CONTRIBUTING, CODE_OF_CONDUCT y SECURITY.
- [x] CHANGELOG, GOVERNANCE y CODEOWNERS.
- [x] Plantillas de issues y PR.
- [x] CI para frontend, Tauri y Rust.
- [x] Iconos y recursos de marca.
- [x] Ejemplo Perlin retirado.
- [ ] Revisar enlaces y capturas finales.
- [ ] Crear el repositorio remoto.
- [ ] Configurar protección de `main`.
- [ ] Subir avatar y social preview.
- [ ] Activar Discussions, Projects y seguridad.

### 17.2 Antes de una release

- [ ] Actualizar versiones en Cargo, npm y Tauri.
- [ ] Actualizar CHANGELOG.
- [ ] Ejecutar `npm run check`.
- [ ] Ejecutar `npm run test`.
- [ ] Ejecutar `cargo fmt --all -- --check`.
- [ ] Ejecutar `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Ejecutar `cargo test --workspace`.
- [ ] Compilar `npm run tauri build`.
- [ ] Probar apertura, cálculo, ayuda y exportación en Linux y Windows.
- [ ] Adjuntar bundles y licencias.
- [ ] Publicar notas y tag.

### 17.3 Después de publicar

- [ ] Revisar issues y Discussions.
- [ ] Confirmar que Dependabot no abre alertas críticas.
- [ ] Actualizar roadmap y milestones.
- [ ] Anunciar la release.
- [ ] Archivar ramas de release antiguas.

---

## 18. Anexos

### 18.1 Comandos rápidos

```bash
# Instalar
npm install

# Desarrollo
npm run tauri dev
npm run dev

# Calidad
npm run check
npm run test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Compilar
npm run tauri build

# CLI
cargo run -p ingecalc-cli -- eval-text "2 + 3 * 4"
cargo run -p ingecalc-cli -- validate examples/avanzado.smj
cargo run -p ingecalc-cli -- eval-document examples/avanzado.smj
cargo run -p ingecalc-cli -- export examples/sample.smj odt examples/sample.odt
```

### 18.2 Rutas clave

| Ruta | Contenido |
|---|---|
| `apps/desktop/src/App.tsx` | Shell, ribbon y diálogos |
| `apps/desktop/src/components/` | Regiones, editor, gráficos y ayuda |
| `apps/desktop/src/lib/` | Evaluación, geometría, i18n, manual y utilidades |
| `apps/desktop/src/stores/` | Documento, historial y selección |
| `apps/desktop/src-tauri/src/lib.rs` | Comandos Tauri y ventana |
| `crates/math-eval/` | Evaluador y control de flujo |
| `crates/math-units/` | Unidades y dimensiones |
| `crates/math-symbolic/` | Motor simbólico |
| `crates/export/` | LaTeX, ODT y PDF |
| `examples/` | Documentos de ejemplo |
| `schemas/` | Esquema JSON |
| `docs/` | Manual, arquitectura y gobernanza |

### 18.3 Enlaces

- Manual: [`docs/manual.md`](manual.md)
- Arquitectura: [`docs/arquitectura.md`](arquitectura.md)
- Publicación: [`docs/publicacion-github.md`](publicacion-github.md)
- Visión: [`docs/vision.md`](vision.md)
- Gobernanza: [`GOVERNANCE.md`](../GOVERNANCE.md)
- Contribuir: [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- Seguridad: [`SECURITY.md`](../SECURITY.md)
- Código de conducta: [`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md)
- Changelog: [`CHANGELOG.md`](../CHANGELOG.md)

### 18.4 Compatibilidad heredada

- Formato nuevo: `ingecalc-json`.
- Formato aceptado: `mathdoc-json` (se migra al guardar).
- Variables nuevas: `INGECALC_*`.
- Variables heredadas: `MATHDOC_*`.
- Identificador Tauri actual: `org.ingecalc.app`.

---

*Mantener esta guía en `docs/guia-desarrollo.md` y actualizarla mediante PR
cuando cambien las reglas, el stack, el formato o el proceso de release.*
