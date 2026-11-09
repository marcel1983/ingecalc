# Análisis funcional de SMath Studio y comparación con IngeCalc

**Fecha:** 5 de noviembre de 2026  
**Fuente principal:** documentación oficial en español, <https://docs.smath.com/es/>  
**Objeto:** inventariar las capacidades documentadas de SMath Studio, compararlas con el estado real de IngeCalc y convertirlas en un plan de desarrollo priorizado.

---

## 1. Resumen ejecutivo

SMath Studio es una **hoja de cálculo matemático WYSIWYG** pensada para ingeniería: regiones libres sobre una página, recálculo en tiempo real, motor numérico y simbólico, unidades, matrices, gráficos 2D/3D, programación, plugins y herramientas de publicación. Su valor no está en la interfaz —que es deliberadamente parecida a papel— sino en el **motor de cálculo integrado** y en la cantidad de dominios que cubre sin salir del documento.

IngeCalc ya tiene una base sólida de producto:

- shell Tauri 2 + React/TypeScript funcionando en Windows y Linux;
- documento JSON versionado `.smj` con páginas y regiones matemáticas, de texto e imagen;
- edición matemática con MathLive y texto enriquecido con TipTap;
- selección, arrastre, redimensionado y borrado de regiones;
- exportación LaTeX y ODT validadas, y PDF raster validado de extremo a extremo;
- CLI con `eval`, `eval-text`, `validate`, `eval-document` y `export`;
- compilación release y pruebas del núcleo en verde.

La brecha principal **no es la interfaz**, sino el motor:

1. el evaluador devuelve `Value`, pero solo cubre reales, cantidades con unidades y matrices numéricas;
2. las variables y funciones de usuario funcionan con un grafo de dependencias; faltan ámbitos anidados, redefiniciones y control de flujo;
3. faltan simbólico, cálculo, resolución de ecuaciones y gráficos;
4. el editor aún carece de formato por región, fondo, paleta de símbolos y gráficos;
5. los plugins, snippets, asistencia dinámica e i18n están en estado de contrato o no existen.

Tras la primera versión de este análisis se implementó el primer bloque de mejoras del editor: evaluación inicial, undo/redo, autosave con recuperación, varias páginas con tamaño y reordenado, configuración de página e impresión, inserción de imágenes, selección múltiple y por rectángulo, arrastre grupal, portapapeles de regiones, navegación con Tab, matemática inline, paletas de funciones/unidades, formato de texto con color/tamaño/subrayado, tablas, enlaces, modelo de valores con variables/unidades/matrices/racionales/complejos, zoom, atajos, tests de frontend y PDF vectorial con LaTeX. En bloques posteriores se añadieron el panel lateral reordenable con búsqueda y griegas, la cuadrícula configurable, las regiones de encabezado/pie, los resultados inline con `=` final, el modo oscuro, las fórmulas ODT como objetos Math embebidos, la ampliación de funciones trigonométricas, hiperbólicas, combinatorias y de matrices, el autoajuste de cajas math/text al contenido, el bloqueo de posiciones global o por caja, el coloreado automático de unidades, la conversión configurable de las unidades de resultado y el formato numérico configurable. Las tablas siguientes reflejan ese estado.

| Estado | Capacidades inventariadas |
|---|---:|
| ✅ Implementadas | 39 |
| 🟡 Parciales | 34 |
| ❌ Ausentes | 61 |
| 🚫 Fuera de alcance | 8 |

**Conclusión:** IngeCalc está por delante de SMath en formato abierto y pipeline de exportación, pero muy por detrás en profundidad de cálculo. Intentar replicar las ~90 páginas de documentación a la vez sería contraproducente. El plan recomendado es:

1. completar el editor de hoja;
2. construir el modelo de valores y el entorno de evaluación;
3. añadir unidades, matrices y gráficos 2D;
4. añadir simbólico, cálculo y resolución;
5. añadir programación, depuración y archivos;
6. cerrar con plugins, snippets, i18n, formato, exportación vectorial y empaquetado Linux.

---

## 2. Método, alcance y leyenda

Se revisaron las páginas del portal en español: guía del usuario, matrices, unidades, gráficos, animaciones, programación, `if`, `for`, `while`, bloque `line`, resolución de ecuaciones, `eval`, `diff`, splines, `solve`, asistencia dinámica, descripciones, formato de regiones, atajos, selección, snippets, complementos, administrador de extensiones, creación de extensiones, Modeller Region, ventana de resultados, manipulación de archivos, soporte multilingüe, integración, comparación con Mathcad y SMath Studio Drawing. Las páginas de licenciamiento, nube, Enterprise y servidores de licencias se revisaron solo para excluirlas del alcance.

**Estado en IngeCalc:**

| Símbolo | Significado |
|---|---|
| ✅ | Implementado y validado |
| 🟡 | Parcial, prototipo o solo el tipo de dato |
| ❌ | Ausente |
| 🚫 | Fuera de alcance |

**Prioridad:**

| Código | Significado |
|---|---|
| P0 | Núcleo del MVP; bloquea flujos básicos |
| P1 | Importante para paridad funcional |
| P2 | Diferible o de nicho |
| — | Fuera de alcance |

**Tamaño orientativo** para un desarrollador con experiencia en el proyecto: **S** = 1–3 días, **M** = 1–2 semanas, **L** = 3–5 semanas, **XL** = 6+ semanas.

---

## 3. Inventario de SMath Studio y comparación

### 3.1 Documento y espacio de trabajo

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Hoja WYSIWYG con regiones libres de matemática, texto, gráfico e imagen | ✅ | Regiones math/text/image/plot con posicionamiento libre, cuadrícula tenue con ajuste configurable, autoajuste al contenido en math/text, creación de una nueva región matemática con Enter debajo de la actual, encabezado/pie como regiones, selección, arrastre, redimensionado, bloqueo de posición y creación de una región matemática al hacer clic en una zona vacía | — | — |
| Recálculo en tiempo real mientras se edita | ✅ | Evaluación con debounce de 240 ms, grafo de dependencias, recálculo en cascada y F9 | — | — |
| Regiones dentro de regiones | ❌ | SMath lo destaca como ventaja; requiere modelo recursivo y layout | P2 | L |
| Sin solapamiento y dentro de márgenes | ✅ | Reflujo vertical al crecer, rechazo de arrastres que invaden otra caja, ancho automático limitado al área imprimible y reajuste al cambiar márgenes; el texto aún no se divide por párrafos | - | - |
| Varias páginas, paginación e impresión | 🟡 | Añadir, eliminar, reordenar, A4/Carta, encabezado/pie como regiones y `window.print()` con CSS de impresión; falta validar el diálogo nativo en Linux | P1 | M |
| Selección por rectángulo, multiselección y Ctrl+clic | ✅ | Marquee sobre el fondo, Ctrl/Shift para añadir, Ctrl+clic y arrastre grupal con límites de página | — | — |
| Mover y redimensionar regiones | ✅ | Arrastre con manijas, límites de página y ajuste a cuadrícula; las regiones matemáticas y de texto se ajustan automáticamente al contenido con mínimo de 5 caracteres y las demás conservan el mínimo 120×48; posiciones fijables por caja o globalmente | — | — |
| Eliminar regiones | ✅ | Botón ×, `Supr`/`Retroceso` y borrado automático de cajas matemáticas o de texto vacías al perder el foco, con protección mientras se edita | — | — |
| Deshacer y rehacer | ✅ | Historial con transacciones; un arrastre o redimensionado cuenta como una operación | — | — |
| Cortar, copiar, pegar y duplicar regiones | ✅ | Ctrl+C/X/V, botones Copiar/Pegar y duplicado; el pegado desplaza y selecciona | — | — |
| Zoom y ajuste de página | ✅ | 40–200 %, botón Ajustar y atajos Ctrl+=/Ctrl+-/Ctrl+0 | — | — |
| Autosave, journal y recuperación | ✅ | Journal `draft.smj` en el directorio de datos, recuperación/descarte y sin límite de 4 MB; fallback web en `localStorage` | — | — |
| Propiedades del documento (título, autor, descripción, idioma, atributos) | 🟡 | `metadata` existe en el JSON; no hay diálogo de propiedades | P1 | S |
| Eliminación automática de elementos vacíos | ❌ | SMath la menciona como ventaja; hoy una región vacía permanece | P2 | S |
| Impresión y configuración de página | 🟡 | Márgenes, encabezado/pie con `{page}`, guía visual, exportación coherente a LaTeX/ODT/PDF y `window.print()`; falta validación nativa en Linux | P1 | M |
| Región de imagen | ✅ | Inserción nativa, data URL, render y export LaTeX/ODT/PDF; el borrador limita el tamaño | — | — |
| Región de gráfico 2D/3D | 🟡 | Región 2D con series explícitas, paramétricas e implícitas, colores, zoom/paneo y export SVG/PNG; faltan 3D y gráficos con unidades | P1 | L |
| Región de dibujo / Modeller Region | ❌ | SMath lo ofrece como plugin; puede ser el plugin de ejemplo de IngeCalc | P2 | XL |
| Formato por región: tamaño de letra, color de texto y fondo | 🟡 | Barra de formato con tamaño, color, subrayado, negrita y cursiva sobre texto seleccionado; falta formato por región, fondo y matemática | P1 | M |
| Formato de varias regiones a la vez | ❌ | Depende de multiselección | P2 | M |
| Atajos de teclado de edición y documento | 🟡 | Ctrl+N/O/S/P/Z/Y/D, F9, zoom y Ctrl+C/X/V; faltan más atajos y menús | P0 | M |
| Menús, paletas y menú contextual | 🟡 | Ribbon por pestañas, panel lateral de fórmulas persistente, reordenable y con búsqueda, `:=`, griegas minúsculas/mayúsculas y secciones, y paletas modales de funciones/unidades; faltan menús y menú contextual | P1 | M |
| Barra lateral y menú extensibles por plugins | ❌ | El contrato de plugins lo prevé; no hay UI | P2 | L |

### 3.2 Entrada y edición matemática

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Notación matemática natural | 🟡 | MathLive permite editar visualmente; la representación guardada es LaTeX + ASCII Math | P0 | M |
| Fracciones, potencias, raíces, subíndices y superíndices | 🟡 | MathLive los edita; el evaluador solo interpreta `+ - * / ^` y funciones simples | P0 | M |
| Letras griegas y símbolos | ✅ | Panel lateral con 24 minúsculas y 24 mayúsculas; MathLive las inserta y el parser las usa como identificadores; falta el atajo Ctrl+G | — | — |
| Matrices desde el teclado o diálogo | 🟡 | MathLive tiene plantillas y la paleta incluye una matriz 2×2; faltan Ctrl+M, diálogo de dimensiones y redimensionado | P1 | M |
| Diálogo “Insertar función” (Ctrl+E) | 🟡 | Paleta **Funciones** con búsqueda, categorías y plantillas; falta el atajo Ctrl+E | P1 | M |
| Diálogo “Insertar unidad” (Ctrl+W) y sintaxis `'` | 🟡 | Paleta **Unidades**, diálogo **Unidad…** para unidades propias y registro dinámico; falta la sintaxis `'` y Ctrl+W | P1 | L |
| Insertar imagen (Ctrl+T) | 🟡 | Botón **+ Imagen** con selector nativo y comando `read_image`; falta el atajo Ctrl+T | P0 | S |
| Insertar gráfico 2D con `@` | 🟡 | Botón **+ Gráfico** con editor de series; falta el atajo `@` | P1 | L |
| Editor/constructor de fórmulas y paletas | 🟡 | MathLive cubre la edición básica; la paleta aporta funciones, plantillas y unidades; falta un constructor propio | P1 | M |
| Texto enriquecido: negrita, cursiva, subrayado, listas, encabezados, enlaces | ✅ | TipTap con B/I/U, color, tamaño, H2, listas, enlaces y tablas; export LaTeX/ODT/PDF | — | — |
| Matemática inline dentro de regiones de texto | ✅ | Nodo `inlineMath` con KaTeX, inserción/edición y export LaTeX/PDF; en ODT se usa fallback ASCII | P0 | L |
| Descripciones de variables, funciones y gráficos | ❌ | SMath las muestra en la asistencia dinámica y las expone con `description()` | P2 | M |
| Asistencia dinámica / autocompletado | ✅ | Popup agrupado con variables, funciones definidas, todas las funciones integradas (incluida la coincidencia exacta de `if`), constantes y operadores básicos; los nombres nuevos sugieren `:=`, las variables existentes solo se ofrecen para usar y las funciones sugieren `=`. La entrada es literal: no se aplican atajos de MathLive ni se eliminan paréntesis automáticamente. No autocorrige: Enter sin navegación solo cierra el menú. `Ctrl+Espacio` abre la lista completa. Faltan descripciones dinámicas de unidades y funciones en todos los idiomas | - | - |
| Renderizado estático con KaTeX al perder foco | ❌ | Hoy MathLive permanece siempre activo; el ADR-0002 preveía KaTeX en reposo | P2 | M |
| Deshacer/rehacer, cortar/copiar/pegar, Tab entre regiones | ✅ | Undo/redo, portapapeles de regiones y Tab/Shift+Tab implementados | — | — |
| Formato de fuente, color y fondo en regiones math/text | 🟡 | Color, tamaño, subrayado y familia tipográfica (sans, serif, monoespaciada y fuentes comunes) en texto seleccionado; faltan fondo, formato por región y matemática | P1 | M |

### 3.3 Motor de cálculo

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Modelo de valores: real, racional, complejo, cantidad, texto y matriz | 🟡 | `math_numeric::Value` incluye texto; las comparaciones usan 1/0; faltan booleanos tipados, listas heterogéneas y expresiones simbólicas como valor | - | - |
| Aritmética `+ - * / ^`, paréntesis y multiplicación implícita | ✅ | Parser y evaluador sobre `Value` | — | — |
| Funciones `sqrt`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, `ln`, `log`, `abs`, `exp`, `floor`, `ceil`, `round`, `trunc`, `frac`, `sign`, `mod`, `gcd`, `lcm`, `factorial`, `comb`, `perm`, `min`, `max`, `equal` | ✅ | Subconjunto ampliado con trigonometría inversa, hiperbólicas, redondeos, combinatoria y teoría de números | — | — |
| Constantes `pi`/`π` y `e` | ✅ | Implementadas | — | — |
| Variables y definiciones `:=` | ✅ | `Value` + `Environment`, definiciones por nombre, evaluación topológica y resultados por región; una definición por nombre | P0 | L |
| Funciones definidas por el usuario | ✅ | `f(x) := …` con parámetros, variables globales, llamadas y detección de recursión/ciclos | P0 | L |
| Grafo de dependencias y detección de ciclos | ✅ | Aristas por variables/funciones, orden topológico y error `DependencyCycle`; faltan redefiniciones y ámbitos | P0 | M |
| Evaluación simbólica con `→` y numérica con `=` | 🟡 | El `=` final evalúa y muestra el resultado dentro de la caja en negrita; el resto de resultados se muestran pequeños en verde fuera de la caja. No hay operador simbólico | P0 | XL |
| `eval()` para forzar cálculo numérico | ❌ | El evaluador ya es numérico, pero no existe la función | P2 | S |
| Fracciones exactas | ✅ | `Rational` con reducción, operaciones y potencias; fuera del rango de 64 bits se usa coma flotante | — | — |
| Números complejos | 🟡 | `Complex` con aritmética, potencias enteras, `sqrt`, `abs`, `re`, `im` y `conj`; faltan funciones trascendentes complejas | P2 | M |
| Matrices y vectores | ✅ | Matrices numéricas con literales, suma/resta/producto, escalar, potencia, indexación 1-based, `det`, `transpose`, `identity`, `inv`, `solve`, `trace`, `row`, `col`, `augment`, `stack`, `minor`, `cofactor`, `sum` y `mean` | — | — |
| Matrices dinámicas y redimensionado | 🟡 | `matrix`, `resize`, `reshape`, `flatten` y `size`; falta el redimensionado arrastrando en la UI | P1 | M |
| Matrices heterogéneas (números, texto, unidades) y matrices anidadas | ❌ | Documentado; el modelo `Value` debe permitir elementos heterogéneos | P2 | L |
| Álgebra lineal: suma, producto, transpuesta, determinante, inversa | ✅ | Suma, producto, transpuesta, determinante, inversa y resolución de sistemas cuadrados; faltan descomposiciones y sistemas no cuadrados | — | — |
| Funciones de matriz: `augment`, `stack`, `minor`, `cofactor`, `vminor` | 🟡 | Implementadas `augment`, `stack`, `minor`, `cofactor`, `trace`, `row`, `col`, `sum` y `mean`; falta `vminor` | P1 | M |
| Filtrado y ordenación: `findrows`, `sorting` | ❌ | Documentadas por SMath | P2 | M |
| Unidades, conversión automática y unidades propias | ✅ | Registro dinámico SI/imperial/common, dimensiones, factores, offsets, conversión, operaciones con simplificación de símbolos, definiciones `unit(nombre) := …`, coloreado azul de unidades (incluidas las escritas a mano) y sistema de unidades de resultado configurable (métrico, imperial, personalizado u original); las unidades propias deben ser lineales | — | — |
| Unidades localizadas y `description()` | 🟡 | Alias en español e inglés (`metro`, `pulgada`, `grado Celsius`) y unidades imperiales; faltan descripciones localizadas y una localización completa de la UI | P2 | M |
| Simbólico: simplificar, expandir, sustituir, factorizar | ✅ | `simplify`, `expand`, `factor` y `substitute` con reglas conservadoras y tests | - | - |
| Derivadas `diff` simbólicas/numéricas, parciales, de orden superior y jacobiano | 🟡 | `diff` simbólico, parciales y orden superior; faltan derivada numérica y jacobiano | P1 | M |
| Integrales numéricas | ✅ | `integral` con Simpson adaptativo y `integrate` con reglas elementales | - | - |
| Resolución `solve`, `roots`, `polyroots` | ❌ | Documentado; requiere parser de ecuaciones y métodos numéricos | P1 | L |
| Sistemas de ecuaciones lineales/no lineales | 🟡 | `solve(A,b)` para sistemas cuadrados con solución única; faltan sistemas no lineales, sobredeterminados y mínimos cuadrados | P1 | L |
| Interpolación `linterp`, `cinterp`, `ainterp` | ❌ | Documentada | P2 | M |
| `mod`, `numden`, `time`, `trace`, `error`, `description` | ❌ | Documentadas; `time` y `trace` son de depuración | P2 | M |
| Booleanos y comparaciones (`=`, `≠`, `≤`, `≥`, `&`, OR lógico, `~`) | 🟡 | Solo `equal` devuelve 1/0; faltan operadores y tipo booleano | P1 | M |
| Rangos y vectores (`2,4..19`) | ❌ | SMath considera un rango como vector | P1 | M |
| Precisión, cifras significativas y formato de números | 🟡 | Notación automática/fija/científica/ingeniería, 0–12 decimales, ceros finales, fracciones o decimal, vista previa y campo rápido de decimales; falta el modo explícito de cifras significativas | P1 | S |
| Modo de optimización numérica vs simbólica | ❌ | No hay modo simbólico | P2 | M |
| Caché, cancelación e interrupción de cálculos largos | ❌ | La evaluación es inmediata y sin cancelación | P1 | M |
| Tiempo de cálculo visible | ❌ | SMath lo muestra; útil para depurar bucles | P2 | S |
| Resaltado de una variable en toda la expresión al pasar el ratón | ❌ | Requiere AST con posiciones/orígenes | P2 | M |
| Dividir una ecuación en dos líneas | ❌ | Documentado como ventaja de edición | P2 | S |
| WaterSteamPro / CoolProp | ❌ | SMath las integra; en IngeCalc deberían ser plugins o una fase posterior | P2 | XL |
| Evaluación de un documento completo desde CLI | 🟡 | La CLI evalúa expresiones y MathJSON, no documentos | P1 | M |

### 3.4 Gráficos y animación

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Gráfico 2D de ecuaciones | ✅ | Funciones explícitas, paramétricas e implícitas (`F(x,y)=0`) con varias series y colores | — | — |
| Gráfico 3D | ✅ | Superficie `z=f(x,y)` con rotación, zoom, malla y exportación SVG/PNG | - | - |
| Varias ecuaciones/matrices en un mismo gráfico | ❌ | Depende del modelo de series | P2 | M |
| Representaciones especiales: texto, puntos, colores, tamaños | ❌ | Documentado con detalle | P2 | L |
| Dominios mediante `if`/booleanos | ❌ | Depende de booleanos y del evaluador de series | P2 | M |
| Vectores representados por índice | ❌ | Depende de vectores | P2 | M |
| Unidades dentro de gráficos | ❌ | Depende de unidades | P2 | M |
| Zoom, paneo, rotación y reset de vista | 🟡 | Zoom con rueda, paneo y reset en 2D; falta rotación y vista 3D | P1 | M |
| Gráficos animados con parámetro `t`, frames, velocidad y repetición | ❌ | Documentado; fase posterior | P2 | XL |
| Modeller Region / figuras interactivas | ❌ | Plugin de ejemplo posible | P2 | XL |
| Exportar gráficos a SVG/PNG | ✅ | Exportación directa del SVG renderizado y PNG a través de canvas | — | — |

### 3.5 Programación y depuración

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| `if(condición, verdadero, falso)` | ✅ | Comparaciones `<`, `<=`, `>`, `>=`, `==`, `!=`, evaluación perezosa y ramas numéricas, con unidades o texto | - | - |
| `for` recorriendo elementos de una matriz | ❌ | Requiere iteración sobre matrices y ámbitos | P1 | L |
| `for` con contador | ✅ | `for(i, inicio, fin, cuerpo)` devuelve una lista; el rango es inclusivo y admite orden descendente | — | — |
| `while(condición, cuerpo)` | ✅ | `while` con límite de 100.000 iteraciones y variables locales dentro de `line` | - | - |
| Bloque `line` / sentencia compuesta | ✅ | `line(expr1, expr2, ...)` con variables locales y valor final | - | - |
| Funciones y procedimientos de usuario | ✅ | `f(x) := ...`, varios parámetros y dependencias globales | — | — |
| Variables locales y alcance | ✅ | Parámetros de funciones, variable de bucles y variables locales de `line`; faltan ámbitos anidados complejos | - | - |
| Ventana de resultados y `trace()` | ❌ | Requiere un canal de logs de evaluación | P2 | M |
| `error()` para detener/ilustrar errores | ❌ | Requiere modelo de errores | P2 | S |
| `wfile`, `rfile`, `dfile` | ❌ | Requiere carpeta de usuario y validación de rutas | P2 | M |
| Depurador integrado, interrupción de bucles | ❌ | SMath lo destaca; fase posterior | P2 | XL |
| Optimización por bloque (numérica/simbólica) | ❌ | Depende de ambos motores | P2 | M |

### 3.6 Extensibilidad

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Manifiesto de plugin y capacidades | 🟡 | `plugin-api` valida manifiesto y versión; no ejecuta nada | P0 | S |
| Host de plugins fuera de proceso con JSON-RPC | ❌ | `plugin-host` es un placeholder; el ADR-0005 fija stdio/JSON-RPC | P1 | L |
| Plugins de evaluación numérica/simbólica | ❌ | Interfaz prevista, sin runtime | P1 | L |
| Plugins de regiones personalizadas | ❌ | Permite gráficos/Modeller sin bloquear el núcleo | P1 | L |
| Plugins de importación/exportación | ❌ | Útil para DOCX/RTF y formatos de ingeniería | P2 | L |
| Plugins de menú, barra lateral y UI | ❌ | Requiere API de UI estable | P2 | L |
| Administrador de extensiones local y galería online | ❌ | Empezar por instalación local desde carpeta/ZIP | P2 | L |
| Snippets reutilizables con metadatos y atajo | ❌ | Se pueden implementar sobre documentos `.smj` | P2 | M |
| Plugin de ejemplo (gráfico 2D o Modeller) | ❌ | Valida el ciclo completo del host | P1 | L |
| Timeouts, límites, kill on hang y sandbox | 🟡 | El manifiesto declara capacidades; no hay control de proceso | P1 | M |
| Compatibilidad con plugins binarios .NET de SMath | 🚫 | Requisito descartado; la API es nueva y fuera de proceso | — | — |

### 3.7 Exportación, archivo e integración

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Guardar/abrir JSON propio `.smj` | ✅ | `doc-json` carga/guarda y valida formato/versión | — | — |
| JSON Schema y validación estructural | 🟡 | Existe `schemas/document.schema.json`; la carga solo comprueba formato/versión | P1 | S |
| Migraciones de versión del documento | ❌ | Necesario antes de cambiar el modelo de regiones | P1 | M |
| Autosave y recuperación | ✅ | Journal en archivo con recuperación/descarte; fallback web en `localStorage` | — | — |
| Archivos recientes | ❌ | Persistencia sencilla | P2 | S |
| Importar `.sm`, `.smz`, `.smw` | 🚫 | No requerido; el formato propio es `.smj` | — | — |
| Exportar LaTeX | ✅ | Texto, matemática, resultados en negrita e imágenes; compilado con XeLaTeX y pdfLaTeX | — | — |
| Exportar ODT | ✅ | ODF + MathML con flujo correcto de párrafos, matrices, griegas, `:=`, resultados y encabezado/pie; los gráficos se exportan como nota | — | — |
| Exportar PDF | ✅ | PDF raster con KaTeX + html2canvas + jsPDF y PDF vectorial con LaTeX/MiKTeX | — | — |
| PDF vectorial mediante LaTeX/MiKTeX | ✅ | Comando `export_pdf_latex`, detección de motor, configuración manual y exportación por CLI; validado con MiKTeX | — | — |
| Exportar PNG/SVG | 🟡 | Gráficos 2D exportables a SVG/PNG; falta exportar regiones completas o el documento como imagen | P1 | M |
| Exportar DOCX/RTF | ❌ | SMath lo cubre vía plugins; no es prioritario | P2 | L |
| Imprimir y configurar página | ✅ | Impresión del documento, tamaño A4/Carta, márgenes, encabezado/pie con `{page}` y reglas `@media print` | — | — |
| CLI `eval`, `eval-text`, `validate`, `eval-document`, `export` | ✅ | Subconjunto funcional; exportación LaTeX, ODT y PDF vectorial, y evaluación de documentos completos | — | — |
| CLI evaluando documentos completos | ✅ | `eval-document` recorre páginas y regiones en orden y devuelve el valor o error de cada región | — | — |
| Servicio de cálculo persistente / modo silencioso | ❌ | SMath Enterprise lo ofrece; IngeCalc puede usar CLI/socket | P2 | L |
| Integración COM | 🚫 | Windows-only y ligada a SMath; no aplica | — | — |
| SMath Viewer (EXE de solo lectura) | 🚫 | Producto separado; podría sustituirse por PDF/HTML | — | — |
| E/S de archivos desde el documento con sandbox | ❌ | Requiere diseño de permisos | P2 | M |

### 3.8 Internacionalización, accesibilidad y producto

| Capacidad de SMath | IngeCalc | Brecha y notas | Prio | Tamaño |
|---|---|---|---|---|
| Interfaz multiidioma | 🟡 | Selector con español, inglés, francés, hindi, árabe (RTL) y chino simplificado; la estructura principal, cintas, panel y diálogos están traducidos; los catálogos profundos de funciones/unidades aún se están localizando | P1 | M |
| Contenido del documento en varios idiomas por región | 🟡 | Idioma global del documento para el corrector ortográfico y selector en la barra de texto; falta el override independiente por región | P2 | L |
| Constantes de ingeniería y autocompletado | ✅ | 24 constantes integradas, constantes propias persistentes, evaluación previa a las regiones, sugerencias de variables/funciones/constantes al escribir y estilo violeta serif en las regiones matemáticas | — | — |
| Corrector ortográfico | ✅ | Hunspell con carga diferida para español, inglés, francés, hindi y árabe, subrayado ondulado, sugerencias al hacer clic derecho, reemplazo, omisión temporal, diccionario personalizado persistente y ocultación del subrayado al imprimir/exportar; chino sin corrector Hunspell | — | — |
| Unidades, funciones y metadatos localizados | ❌ | Depende de unidades e i18n | P2 | XL |
| Accesibilidad: roles, etiquetas, navegación por teclado | 🟡 | Las regiones ya tienen `role="group"` y etiquetas; faltan navegación y menús accesibles | P1 | M |
| Temas claro/oscuro | ✅ | Ribbon, UI, hoja, cajas matemáticas y texto con tema persistente; la impresión fuerza papel blanco y texto oscuro | — | — |
| Build de Windows | ✅ | Verificado: compila, abre y exporta | — | — |
| Build y paquetes de Linux | ❌ | Objetivo principal del producto; falta validar WebKitGTK y empaquetar | P0 | L |
| Versión portable | 🟡 | El `.exe` release funciona; no hay paquete portable formal | P2 | S |
| Versión web | 🚫 | No es objetivo del MVP | — | — |
| Nube, Enterprise, licencias y servidores | 🚫 | SMath comercial; IngeCalc es GPL-3.0 y sin cuentas | — | — |
| SMath Writer / Viewer | 🚫 | Productos separados | — | — |
| Licencia GPL-3.0 y cabeceras SPDX | ✅ | `LICENSE` y cabeceras en el código Rust | — | — |
| CI multi-OS | 🟡 | Workflow configurado para Ubuntu y Windows; falta verificación Linux real | P1 | M |
| Documentación de usuario | 🟡 | README, ADRs y este análisis; falta manual de usuario | P2 | M |
| Autoactualización | 🚫 | No prioritaria; puede añadirse al final | — | — |

---

## 4. Brechas críticas, en orden de dependencia

1. **Modelo de valores.** `Value` cubre reales, racionales, complejos, unidades, matrices y texto; todavía faltan booleanos tipados, listas heterogéneas y resultados simbólicos como valor.
2. **Entorno de evaluación.** Sin variables, funciones de usuario, ámbitos, dependencias y recálculo en cascada no se puede hablar de hoja de cálculo.
3. **Unidades.** Es la característica que más diferencia a SMath en ingeniería; requiere registro, dimensiones, conversión, offsets y formato.
4. **Matrices y vectores.** Sin ellos no hay álgebra lineal, sistemas, gráficos por matrices ni buena parte de las funciones.
5. **Simbólico y cálculo.** Ya hay simplificación, derivadas, integrales numéricas y resolución; faltan un motor simbólico más profundo, cálculo simbólico avanzado e integración simbólica amplia.
6. **Gráficos.** La región 2D es la siguiente gran brecha visual; el 3D y las animaciones pueden esperar.
7. **Programación.** `if`, `for`, `sum`, `product`, `while`, `line`, iteración de matrices y `error`/`trace` ya están; falta un depurador visual y E/S de archivos.
8. **Editor.** Undo/redo, autosave, varias páginas, imagen, multiselección, zoom, atajos e inline math son necesidades de uso diario.
9. **Ecosistema.** Plugins, snippets, asistencia dinámica y descripciones multiplican el valor, pero deben llegar después del motor.
10. **Producto.** i18n, exportación vectorial, accesibilidad, empaquetado Linux y documentación.

---

## 5. Plan de desarrollo propuesto

### Fase 0 — Calidad y fundamentos (1 semana)

- Añadir Vitest para geometría, store y utilidades; Playwright o `tauri-driver` para smoke E2E.
- Validar el JSON contra `schemas/document.schema.json` al cargar y al guardar.
- Añadir *error boundaries* y un registro de errores de UI.
- Completar la evaluación inicial y el estado de error de las regiones.
- Congelar el contrato del documento antes de añadir campos nuevos.

**Salida:** CI verde, pruebas de regresión para drag/resize y exportación, carga inválida rechazada.

### Fase 1 — Editor de hoja completo (3–4 semanas)

- Evaluación inicial al montar y recálculo global (F9).
- Undo/redo con transacciones y agrupación por edición.
- Autosave, journal y recuperación al abrir.
- Varias páginas: añadir, borrar, reordenar, tamaño A4/Letter/custom.
- Inserción de imágenes con copia a assets y exportación.
- Multiselección por rectángulo y Ctrl+clic; copiar, pegar, duplicar y orden Z.
- Zoom 50–200 %, ajustar ancho/página.
- Atajos: Ctrl+N/O/S/Z/Y/C/V, Tab entre regiones, Ctrl+Enter, F9.
- Inline math en TipTap con nodo propio, MathLive en edición y KaTeX en reposo.
- Barra de formato para texto: tamaño, color, fondo, negrita/cursiva/subrayado.
- Diálogo de propiedades del documento.

**Salida:** documento multipágina con texto enriquecido, matemática inline, imágenes, undo/redo y autosave.

### Fase 2 — Núcleo numérico y variables (4–5 semanas)

- `math-numeric`: `Value` con real, racional, complejo, booleano, matriz, cantidad y texto; errores tipados.
- AST ampliado: asignación `:=`, definición de función, rango, unidad, matriz, comparación, bloque.
- `math-eval`: entorno de ámbitos, funciones de usuario, grafo de dependencias, detección de ciclos, caché y cancelación.
- Parser: cubrir LaTeX/ASCII de MathLive para asignaciones, matrices, unidades y rangos.
- Biblioteca de funciones: trigonométricas, hiperbólicas, logaritmos, combinatoria, mínimo/máximo, `mod`, redondeos.
- Formato numérico: fracción/decimal, precisión y cifras significativas; usar `settings.precision`.
- CLI: evaluar un documento completo y devolver resultados por región.

**Salida:** hoja con variables, funciones, fracciones, complejos y recálculo; 100+ vectores de prueba.

### Fase 3 — Unidades, matrices y gráficos 2D (5–6 semanas)

- Registro de unidades SI + ingeniería, conversión, offsets de temperatura, unidades propias y unidad de resultado.
- Matrices/vectores dinámicos: literales, `mat`, `matrix`, indexación 1-based, `row`, `col`, transpuesta, producto, determinante, inversa.
- Funciones `augment`, `stack`, `minor`, `cofactor`, `vminor`, `findrows`, `sorting`.
- Región de gráfico 2D: ecuaciones y matrices, múltiples series, dominios, unidades, zoom/paneo, puntos/líneas, colores y export SVG/PNG.
- `solve`, `roots`, `polyroots` y sistemas lineales.
- Formato y localización inicial de unidades.

**Salida:** hoja de ingeniería con unidades, matrices y gráfico 2D exportable.

### Fase 4 — Simbólico, cálculo y resolución (4–5 semanas)

- Motor simbólico: simplificar, expandir, sustituir, recoger, factorizar lo básico, `numden`.
- `diff` simbólico y numérico, parciales, orden superior y jacobiano.
- Integración numérica adaptativa y `eval()`.
- Resolución simbólica y numérica; sistemas no lineales.
- Interpolación `linterp`, `cinterp`, `ainterp`.
- Optimización numérica/simbólica por bloque y tiempo de cálculo.

**Salida:** documento con derivadas, integrales, resolución y simbólico validado con casos de ingeniería.

### Fase 5 — Programación, depuración y archivos (3–4 semanas)

- `if`, `for` (dos formas), `while`, bloque `line`, funciones/procedimientos y ámbitos locales.
- Ventana de resultados, `trace`, `error`, interrupción de cálculos e indicador de tiempo.
- `wfile`, `rfile`, `dfile` con carpeta de usuario, validación de rutas y permisos.
- Depuración básica: traza, paso a paso y detección de bucles infinitos.

**Salida:** ejemplos de Fibonacci, sumas de columnas y funciones reutilizables; E/S segura.

### Fase 6 — Ecosistema y acabado (5–6 semanas)

- Host JSON-RPC de plugins: spawn, `initialize`, `evaluate`, `render_region`, `shutdown`, capacidades, timeout y kill.
- Plugin de ejemplo (gráfico 2D o Modeller) y documentación del SDK.
- Administrador de extensiones local (carpeta/ZIP, activar/desactivar, manifiesto) y, más adelante, catálogo online.
- Snippets: crear desde selección, metadatos, atajo e inserción desde asistencia.
- Asistencia dinámica y descripciones multiidioma.
- i18n de UI, documento, unidades, funciones y metadatos (español/inglés primero).
- Barra de formato, paletas, menús contextuales y atajos completos.
- Exportación: PDF vectorial con LaTeX, PNG/SVG, tablas, inline math, saltos de página y encabezados.
- Empaquetado Linux (AppImage/deb/rpm), CI, accesibilidad y manual de usuario.
- 3D y animaciones como fase posterior o plugin.

**Salida:** MVP con paridad funcional para flujos de ingeniería típicos, paquetes Linux y ecosistema de plugins.

---

## 6. Backlog priorizado (top 20)

| # | Tarea | Área | Prio | Tamaño | Depende de |
|---|---|---|---|---|---|
| 1 | Evaluación inicial y recálculo global | Editor | P0 | S | — |
| 2 | Undo/redo con transacciones | Editor | P0 | M | — |
| 3 | Autosave y recuperación | Documento | P0 | M | — |
| 4 | Varias páginas y tamaño de página | Documento | P0 | M | — |
| 5 | Inserción de imágenes | Documento | P0 | S | Assets |
| 6 | Multiselección, copiar/pegar/duplicar | Editor | P0 | M | — |
| 7 | Zoom y ajuste | Editor | P0 | S | — |
| 8 | Atajos de teclado | Editor | P0 | S | — |
| 9 | Inline math en TipTap | Editor | P0 | L | Nodo TipTap |
| 10 | Modelo de valores (`Value`) | Motor | P0 | L | — |
| 11 | Variables, funciones y grafo de dependencias | Motor | P0 | L | #10 |
| 12 | Unidades | Motor | P0 | XL | #10, #11 |
| 13 | Matrices y álgebra lineal | Motor | P0 | L | #10 |
| 14 | Biblioteca de funciones | Motor | P1 | M | #10 |
| 15 | Gráfico 2D | UI/Motor | P1 | L | #11, #13 |
| 16 | Simbólico: simplificar y derivar | Motor | P1 | L | #10 |
| 17 | Resolución de ecuaciones | Motor | P1 | M | #16 |
| 18 | Programación `if/for/while/line` | Motor | P1 | L | #11 |
| 19 | Host de plugins + plugin de ejemplo | Plugins | P1 | L | #10 |
| 20 | i18n, formato y paquete Linux | Producto | P1 | L | — |

---

## 7. Quick wins implementados y siguientes

Implementados en la primera iteración:

1. **Evaluación inicial al montar**: `2+2` muestra `= 4` sin editar.
2. **Undo/redo** con historial transaccional; un arrastre es una sola operación.
3. **Autosave** en `localStorage` con recuperación al iniciar y descarte explícito.
4. **Varias páginas**: añadir y eliminar, con undo.
5. **Inserción de imagen** con selector nativo y comando `read_image`.
6. **Multiselección con Ctrl+clic, duplicado y borrado múltiple**.
7. **Zoom 40–200 %**, ajuste a ventana y atajos.
8. **Atajos** Ctrl+N/O/S/Z/Y/D, F9, zoom y Ctrl+C/X/V.
9. **Tests de frontend** con Vitest (geometría, historial/store, páginas/portapapeles y borradores).
10. **PDF vectorial con LaTeX**: comando `export_pdf_latex`, detección de motor y configuración manual; validado con MiKTeX.
11. **Tamaño y reordenado de páginas**: A4/Carta, flechas ↑/↓ y undo.
12. **Portapapeles de regiones y navegación Tab/Shift+Tab**.
13. **Selección por rectángulo y arrastre grupal** con límites de página.
14. **Matemática inline y barra de formato**: nodo KaTeX en TipTap; B/I/H2/listas; export LaTeX/PDF y fallback ODT.
15. **Journal de autosave en archivo** con recuperación/descarte y sin límite de 4 MB.
16. **PDF vectorial por CLI** con motor explícito o autodetección.
17. **Configuración de página e impresión**: márgenes, encabezado/pie con `{page}`, coherencia en LaTeX/ODT/PDF y CSS `@media print`.
18. **Paletas de funciones/unidades y formato ampliado**: catálogo con búsqueda/categorías, inserción en MathLive, color, tamaño de letra y subrayado con exportación LaTeX/ODT/PDF.
19. **Tablas y enlaces**: inserción 3×3, controles de filas/columnas, encabezados, enlaces en la toolbar y exportación LaTeX/ODT/PDF.
20. **Modelo de valores, variables, unidades y matrices**: `Value`, `Environment`, asignaciones `:=`, evaluación por documento, registro de unidades, literales de matriz, operaciones, `det`, `transpose` e `identity`.
21. **Funciones de usuario, dependencias y ciclos**: `f(x) := …`, llamadas, análisis de símbolos/funciones, orden topológico, detección de recursión y definiciones duplicadas.
22. **Indexación, inversa y sistemas**: `A[i,j]` 1-based, `A[i]`, `inv`, `A^-1`, `solve(A,b)` y potencias negativas.
23. **Racionales y complejos**: `Rational` exacto, `Complex`, `i`, aritmética, `sqrt(-1)`, `abs`, `re`, `im` y `conj`.
24. **Unidades propias y localización**: registro dinámico, `unit(nombre) := …`, diálogo **Unidad…**, unidades imperiales y alias español/inglés.
25. **Gráficos 2D**: región `plot`, series con colores, muestreo por documento, SVG, zoom/paneo y exportación SVG/PNG.
26. **Ribbon y modo oscuro**: pestañas Inicio/Insertar/Página/Fórmulas/Exportar, grupos de comandos y tema persistente.
27. **Gráficos paramétricos e implícitos**: `x(t), y(t)`, `F(x,y)=0`, marching squares y varias series.
28. **Cálculo simbólico y resolución**: `simplify`, `expand`, `factor`, `substitute`, `diff`, `integrate`, `integral`, `solve`, `roots` y `polyroots`.
29. **Matrices avanzadas**: LU, QR, autovalores simétricos, rango, RREF, rangos, `resize`/`reshape` y operaciones vectoriales.
30. **Gráficos 3D y unidades**: superficies `z=f(x,y)` con rotación, zoom y malla; etiquetas de unidades en los ejes.
31. **Programación completa**: `while`, `line`, `for` sobre matrices, `trace` y `error`.
32. **Disposición sin solapamientos**: reflujo vertical al crecer, rechazo de arrastres invasivos y traslado a la página siguiente.

Pendientes inmediatos:

1. Profundizar el motor simbólico: integración simbólica amplia, Jacobiano, simplificación avanzada y resolución simbólica general.
2. Matrices heterogéneas, redimensionado dinámico en la UI y descomposición SVD/mínimos cuadrados.
3. Depurador visual, E/S de archivos (`wfile`, `rfile`, `dfile`) e interrupción de cálculos.
4. Regiones anidadas y división de texto con indicador `…`.
5. Plugins, snippets, asistencia dinámica y localización profunda.

---

## 8. Recomendaciones de arquitectura

- **Valor canónico:** `math_numeric::Value` cubre reales, racionales, complejos, cantidades, matrices y texto. El siguiente paso es añadir `Boolean`, listas heterogéneas y expresiones simbólicas como valor.
- **Entorno:** `Environment` ya contiene variables, funciones de usuario y una pila de llamadas; la evaluación usa un grafo topológico. Faltan ámbitos anidados, redefiniciones controladas, caché y cancelación.
- **AST:** `Assignment`, `Matrix`, `FunctionDef`, comparaciones y llamadas a `if`/`for`/`sum`/`product`/`while`/`line` ya están en `math-ast`. `range` y `line` se representan como llamadas; faltan `Boolean`, bloques tipados y un parser de programa separado.
- **Parser:** MathLive sigue enviando ASCII Math; el parser ya acepta asignaciones, matrices y unidades entrecomilladas. Mantener el `source` LaTeX y ampliar el parser para rangos, comparaciones y funciones de usuario.
- **Unidades:** el registro es dinámico por documento y admite unidades propias lineales. El siguiente paso es localización completa, descripciones y unidades propias con offset.
- **Matrices:** las matrices actuales son `Vec<Vec<f64>>` con indexación, inversa, sistemas, LU, QR, autovalores simétricos, rango y utilidades vectoriales. Valorar `nalgebra::DMatrix` para SVD, mínimos cuadrados, matrices heterogéneas y redimensionado dinámico.
- **Simbólico:** motor de reglas propio con `simplify`, `expand`, `factor`, `substitute`, `diff` e `integrate`; `egg` es una alternativa si se necesita reescritura avanzada. Evitar dependencias con licencias incompatibles con GPL-3.0.
- **Gráficos:** la región 2D renderiza SVG desde React y exporta SVG/PNG; el siguiente paso es añadir ecuaciones implícitas/paramétricas y 3D en plugin o WebGL.
- **Plugins:** JSON-RPC sobre stdio con `tokio::process`, capacidades declaradas, timeout, límite de memoria y kill; WASM en fase 2. Mantener la API independiente de SMath.
- **Editor:** Zustand + Immer con historial por transacciones; TipTap con nodo inline de matemática; virtualización si el documento supera ~500 regiones.
- **Exportación:** pipeline único de documento → modelo intermedio → LaTeX/ODT/PDF/SVG; añadir pruebas golden. El PDF vectorial por LaTeX debe ser una opción, con el raster como fallback.
- **Testing:** Rust con `insta`/`proptest` para parser/evaluador; Vitest para geometría/store; Playwright/`tauri-driver` para flujos de región y exportación.

---

## 9. Fuera de alcance

No forman parte del MVP de IngeCalc:

- licenciamiento, cuentas, suscripciones, servidores de licencias, nube y Enterprise de SMath;
- SMath Viewer y SMath Writer;
- compatibilidad con `.sm`, `.smz`, `.smw` y plugins binarios .NET de SMath;
- bloques de resolución, geometría constructiva y simulación mecánica de Mathcad;
- colaboración en tiempo real, asistente AI y sincronización de archivos;
- paridad con las 46 traducciones de SMath; empezar por español e inglés;
- WaterSteamPro/CoolProp integrados; pueden ser plugins de dominio.

---

## 10. Fuentes consultadas

- Portal: <https://docs.smath.com/es/>
- Guía del usuario: <https://docs.smath.com/es/user-guide/>
- Matrices: <https://docs.smath.com/es/matrices/>
- Unidades: <https://docs.smath.com/es/units/>
- Gráficos: <https://docs.smath.com/es/graphs/>
- Animaciones: <https://docs.smath.com/es/animations/>
- Programación: <https://docs.smath.com/es/programming/>
- `if`: <https://docs.smath.com/es/if-condition/>
- `for`: <https://docs.smath.com/es/for-loop/>
- `while`: <https://docs.smath.com/es/while-loop/>
- Bloque `line`: <https://docs.smath.com/es/line-block/>
- Resolución de ecuaciones: <https://docs.smath.com/es/solving-equations/>
- `eval`: <https://docs.smath.com/es/fn-eval/>
- `diff`: <https://docs.smath.com/es/fn-diff/>
- Splines: <https://docs.smath.com/es/fn-splines/>
- `solve`: <https://docs.smath.com/es/fn-solve/>
- Asistencia dinámica: <https://docs.smath.com/es/dynamic-assistance/>
- Descripciones: <https://docs.smath.com/es/descriptions/>
- Formato de regiones: <https://docs.smath.com/es/formatting-text-math/>
- Atajos: <https://docs.smath.com/es/keyboard-shortcuts/>
- Selección: <https://docs.smath.com/es/selecting/>
- Snippets: <https://docs.smath.com/es/snippets/>
- Complementos: <https://docs.smath.com/es/plugins/>
- Administrador de extensiones: <https://docs.smath.com/es/extensions-manager/>
- Creación de extensiones: <https://docs.smath.com/es/creating-extensions/>
- Modeller Region: <https://docs.smath.com/es/modeller-region/>
- Ventana de resultados: <https://docs.smath.com/es/output-window/>
- Manipulación de archivos: <https://docs.smath.com/es/file-manipulation/>
- Soporte multilingüe: <https://docs.smath.com/es/multilanguage-support/>
- Integración: <https://docs.smath.com/es/integration/>
- SMath Studio vs Mathcad: <https://docs.smath.com/es/smath-studio-vs-mathcad/>
- SMath Studio Drawing: <https://docs.smath.com/es/smath-studio-drawing/>
- SMath Studio: <https://docs.smath.com/es/smath-studio/>

---

## 11. Conclusión

SMath Studio no es “una calculadora con una hoja bonita”: es un **sistema de cálculo de ingeniería** con un lenguaje de documento, un motor numérico/simbólico, unidades, matrices, gráficos, programación y un ecosistema de extensiones. IngeCalc ya tiene la carcasa y la salida a formatos abiertos; la siguiente inversión debe ser el **motor y el modelo de documento**, no añadir botones aislados.

El orden propuesto evita el error clásico de implementar funciones de nicho antes de tener valores, variables, unidades y matrices. Si se completa la Fase 3, IngeCalc cubrirá el núcleo de una hoja de ingeniería; las fases 4–6 lo acercarán a la experiencia SMath sin copiar su producto comercial ni su formato propietario.