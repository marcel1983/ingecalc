export type ManualBlock =
  | { type: "paragraph"; text: string }
  | { type: "list"; items: string[] }
  | { type: "code"; title?: string; text: string }
  | { type: "table"; headers: string[]; rows: string[][] }
  | { type: "note"; title?: string; text: string };

export type ManualSection = {
  id: string;
  icon: string;
  title: string;
  summary: string;
  keywords: string[];
  blocks: ManualBlock[];
};

export type ManualContent = {
  title: string;
  subtitle: string;
  version: string;
  sections: ManualSection[];
};

const es: ManualContent = {
  title: "Manual de IngeCalc",
  subtitle: "Guía de uso y referencia rápida",
  version: "0.1.2",
  sections: [
    {
      id: "inicio",
      icon: "🧭",
      title: "Primeros pasos",
      summary: "Crear, abrir y guardar una hoja de cálculo técnica.",
      keywords: ["introducción", "empezar", "nuevo", "abrir", "guardar"],
      blocks: [
        {
          type: "paragraph",
          text: "IngeCalc organiza el trabajo en páginas y regiones. Una región puede contener matemática, texto enriquecido, una imagen o un gráfico."
        },
        {
          type: "list",
          items: [
            "Abre un documento `.smj` con **Abrir** (`Ctrl+O`) o crea uno nuevo con **Nuevo** (`Ctrl+N`).",
            "Haz clic en una zona vacía de la página para crear una región matemática.",
            "Escribe una expresión y pulsa **Recalcular** (`F9`) para actualizar los resultados.",
            "Guarda con `Ctrl+S`. El autosave conserva un borrador recuperable."
          ]
        },
        {
          type: "code",
          title: "Primera fórmula",
          text: "x := 3\nf(t) := t^2\nf(x) + 1"
        },
        {
          type: "note",
          title: "Ayuda siempre disponible",
          text: "Pulsa `F1` o el botón `?` de la barra superior para abrir este manual desde cualquier parte de la aplicación."
        }
      ]
    },
    {
      id: "interfaz",
      icon: "🪟",
      title: "Interfaz",
      summary: "Ribbon, panel de fórmulas, páginas y barra de estado.",
      keywords: ["ribbon", "barra", "panel", "pestañas", "estado"],
      blocks: [
        {
          type: "paragraph",
          text: "La barra superior agrupa los comandos en pestañas: **Inicio**, **Insertar**, **Página**, **Fórmulas**, **Exportar** y **Ayuda**."
        },
        {
          type: "list",
          items: [
            "**Inicio:** portapapeles, regiones, documento y recálculo.",
            "**Insertar:** regiones, encabezado/pie, funciones, unidades y constantes.",
            "**Página:** tamaño, márgenes, unidades de resultado, números, zoom y vista.",
            "**Fórmulas:** asignación `:=`, panel lateral y catálogos matemáticos.",
            "**Exportar:** LaTeX, ODT, PDF raster y PDF vectorial.",
            "**Ayuda:** manual, primeros pasos, atajos, problemas y acerca de IngeCalc."
          ]
        },
        {
          type: "paragraph",
          text: "El panel lateral de fórmulas permite buscar e insertar funciones, operadores, símbolos, unidades y gráficos en la región activa. Se puede ocultar con el botón de panel de la barra superior."
        },
        {
          type: "note",
          title: "Barra de estado",
          text: "A la derecha de la barra superior se muestran el estado del documento, el tema, el idioma de la interfaz y el acceso a la ayuda."
        }
      ]
    },
    {
      id: "documento",
      icon: "📄",
      title: "Documento y regiones",
      summary: "Páginas, regiones, reflujo, márgenes, imágenes y tablas.",
      keywords: ["página", "región", "texto", "imagen", "tabla", "márgenes", "reflujo"],
      blocks: [
        {
          type: "list",
          items: [
            "Cada documento tiene páginas A4 o Carta con márgenes, encabezado y pie editables.",
            "Las regiones matemáticas y de texto se ajustan a su contenido. Las imágenes y los gráficos se redimensionan con la manija inferior derecha.",
            "Al crecer una caja, las cajas situadas debajo se reflotan hacia abajo. Si no caben, pasan a la página siguiente.",
            "Las cajas respetan los márgenes imprimibles. Las regiones de encabezado y pie son la excepción.",
            "Las cajas vacías se eliminan al perder el foco. `Ctrl+Z` permite recuperarlas.",
            "`Ctrl+clic` añade o quita regiones de la selección; arrastrar sobre el fondo crea una selección rectangular.",
            "`Ctrl+D` duplica la selección; `Ctrl+C`, `Ctrl+X` y `Ctrl+V` gestionan el portapapeles de regiones.",
            "El botón de fijar bloquea las posiciones. Una caja fijada no se mueve, pero su contenido sigue siendo editable.",
            "**Insertar → Imagen** añade una imagen redimensionable. **Insertar → Tabla** crea una tabla 3×3 editable.",
            "El botón **ƒx** de una región de texto inserta matemática inline con KaTeX."
          ]
        },
        {
          type: "note",
          title: "Enter en matemática",
          text: "Dentro de una región matemática, `Enter` crea otra región matemática justo debajo y mueve el cursor a ella, sin solaparse."
        },
        {
          type: "paragraph",
          text: "Las páginas se añaden, reordenan y eliminan desde la pestaña **Página**. El token `{page}` del encabezado o pie se sustituye por el número de página."
        }
      ]
    },
    {
      id: "matematica",
      icon: "∑",
      title: "Matemática y variables",
      summary: "Variables, funciones, operadores y evaluación por dependencias.",
      keywords: ["variable", "función", "definición", "fórmula", "evaluar"],
      blocks: [
        {
          type: "code",
          title: "Definiciones",
          text: "x := 3\nf(t) := t^2\nf(x) + 1"
        },
        {
          type: "list",
          items: [
            "`:=` define una variable o una función.",
            "Las definiciones se evalúan por dependencias, no por el orden en que aparecen en la página.",
            "Cada nombre admite una sola definición. No hay redefiniciones, ámbitos anidados ni funciones recursivas.",
            "`=` al final de una región funciona como marcador de evaluación y muestra el resultado.",
            "Los operadores disponibles incluyen `+`, `-`, `*`, `/`, `^`, comparaciones y `:=`."
          ]
        },
        {
          type: "note",
          title: "Errores de nombre",
          text: "Si aparece «variable o símbolo no definido», crea el nombre con `nombre := valor` o selecciona la sugerencia de definición del autocompletado."
        }
      ]
    },
    {
      id: "autocompletado",
      icon: "⌨",
      title: "Autocompletado y entrada literal",
      summary: "Sugerencias, atajos y escritura sin autocorrección.",
      keywords: ["sugerencia", "completar", "autocorrección", "literal", "tab"],
      blocks: [
        {
          type: "paragraph",
          text: "Al escribir el comienzo de una variable, función, constante u operador aparece un menú agrupado por tipo."
        },
        {
          type: "list",
          items: [
            "Si el nombre no existe, se sugiere `nombre :=` para crear la variable.",
            "Si la variable ya existe, solo se ofrece para usarla; no se permite redefinirla.",
            "Las funciones definidas sugieren primero `f(...) =` para evaluarlas.",
            "`↑`/`↓` navegan por las sugerencias.",
            "`Tab` o un clic insertan la sugerencia seleccionada.",
            "`Enter` sin navegación solo cierra el menú, sin autocorregir.",
            "`Ctrl+Espacio` abre el catálogo completo y `Esc` cierra el menú."
          ]
        },
        {
          type: "note",
          title: "Entrada literal",
          text: "No se aplican conversiones automáticas como `Re → ℜ`, `alpha → α` o `<= → ≤`. Los símbolos se insertan desde la paleta o el autocompletado. La barra `/` sigue creando una fracción como estructura matemática."
        }
      ]
    },
    {
      id: "unidades",
      icon: "📐",
      title: "Unidades",
      summary: "Conversión, unidades localizadas, propias y sistema de resultados.",
      keywords: ["unidad", "conversión", "metro", "pulgada", "temperatura", "SI"],
      blocks: [
        {
          type: "code",
          title: "Conversiones",
          text: "d := 2\"m\" + 30\"cm\"\nt := 2\"s\"\nv := d/t\nT := 20\"°C\" + 5\"K\""
        },
        {
          type: "list",
          items: [
            "La suma y la resta convierten a la unidad de la izquierda.",
            "La multiplicación y la división combinan y simplifican dimensiones.",
            "Se admiten unidades SI, imperiales y localizadas.",
            "**Página → Unidades…** permite elegir el sistema de resultados, personalizar magnitudes o conservar las unidades originales.",
            "Las unidades propias se definen con `unit(\"nombre\") := expresión`."
          ]
        },
        {
          type: "code",
          title: "Unidad propia",
          text: "unit(\"mi_pulgada\") := 2.54 cm\nunit(\"mi_pie\") := 12 mi_pulgada\n0\"cm\" + 1\"mi_pie\""
        },
        {
          type: "note",
          title: "Limitación",
          text: "Las unidades propias deben ser lineales y no pueden redefinir una unidad existente. Los offsets solo están disponibles en las unidades integradas."
        }
      ]
    },
    {
      id: "matrices",
      icon: "▦",
      title: "Matrices y vectores",
      summary: "Construcción, operaciones, inversa, LU, QR y autovalores.",
      keywords: ["matriz", "vector", "determinante", "inversa", "lu", "qr", "autovalores"],
      blocks: [
        {
          type: "code",
          title: "Operaciones básicas",
          text: "A := [1,2;3,4]\nB := [2,0;1,2]\nA*B\ndet(A)\nA^-1\nA[2,1]\nidentity(3)\nsolve([2,1;1,3],[1;2])"
        },
        {
          type: "paragraph",
          text: "La indexación es 1-based: `A[2,1]` es la fila 2, columna 1; `A[1]` devuelve la primera fila."
        },
        {
          type: "code",
          title: "Matrices avanzadas",
          text: "lu(A)\nqr(A)\neigenvalues([2,1;1,2])\nrank(A)\nrref(A)\nrange(1,5)\nreshape(range(1,6), 2, 3)\ndot([1,2,3],[4,5,6])\ncross([1,0,0],[0,1,0])\nnorm([3,4])"
        },
        {
          type: "note",
          title: "Limitación",
          text: "Las matrices son numéricas (`f64`). No admiten cantidades heterogéneas, redimensionado dinámico ni sistemas no lineales."
        }
      ]
    },
    {
      id: "simbolico",
      icon: "∫",
      title: "Cálculo simbólico",
      summary: "Simplificar, expandir, factorizar, derivar, integrar y resolver.",
      keywords: ["simbólico", "derivada", "integral", "factorizar", "resolver", "raíces"],
      blocks: [
        {
          type: "code",
          title: "Ejemplos",
          text: "simplify(x + 0 + x)\nexpand((x+1)^2)\nfactor(x^2 - 4)\ndiff(x^3 + 2*x, x)\nintegrate(x^2, x)\nintegral(sin(x), x, 0, pi)\nsolve(x^2 - 5*x + 6, x)\npolyroots([1, 0, 1])"
        },
        {
          type: "paragraph",
          text: "El motor cubre simplificación, expansión, factorización, sustitución, derivadas, primitivas elementales, integrales definidas numéricas, ecuaciones y raíces."
        },
        {
          type: "note",
          title: "Alcance",
          text: "No es un CAS completo. Las transformaciones avanzadas o las ecuaciones no soportadas pueden devolver una expresión simbólica o un error."
        }
      ]
    },
    {
      id: "flujo",
      icon: "↻",
      title: "Control de flujo",
      summary: "Condicionales, bucles, sumatorias, productos y variables locales.",
      keywords: ["if", "for", "while", "sum", "product", "bucle", "condicional"],
      blocks: [
        {
          type: "code",
          title: "Ejemplos",
          text: "if(x > 0, 1, 0)\nfor(i, 1, 10, i^2)\nfor(v, [1,2,3], v^2)\nsum(i, 1, 100, i)\nproduct(i, 1, 5, i)\nline(x := 0, while(x < 5, x := x + 1), x)\ntrace(\"valor\", x^2)\nerror(\"mensaje\")"
        },
        {
          type: "list",
          items: [
            "`if(condición, valor_si_verdadero, valor_si_falso)`.",
            "`for` recorre un rango o los elementos de una matriz.",
            "`sum` y `product` acumulan una expresión sobre un rango.",
            "`while` repite mientras se cumpla una condición.",
            "`line` evalúa una secuencia de expresiones con variables locales.",
            "`trace` devuelve un valor para depuración y `error` detiene el cálculo."
          ]
        },
        {
          type: "note",
          title: "Límite",
          text: "Los bucles tienen un límite de 100 000 iteraciones para evitar bloqueos."
        }
      ]
    },
    {
      id: "graficos",
      icon: "📈",
      title: "Gráficos",
      summary: "Series 2D, paramétricas, implícitas y superficies 3D.",
      keywords: ["gráfico", "plot", "2d", "3d", "superficie", "eje", "unidades"],
      blocks: [
        {
          type: "list",
          items: [
            "**Explícita:** `sin(x)`.",
            "**Paramétrica:** `x(t)=cos(t)`, `y(t)=sin(t)`, con rango de `t`.",
            "**Implícita:** `x^2+y^2-4` para representar una circunferencia.",
            "**Superficie 3D:** `z=f(x,y)` con dominios X/Y, rotación, zoom y malla.",
            "**Con unidades:** `x * 1 m` conserva la unidad en el eje Y."
          ]
        },
        {
          type: "note",
          title: "Exportación",
          text: "Los gráficos se recalculan al abrirlos y se pueden exportar como SVG o PNG. En LaTeX y ODT se exporta una nota en lugar del gráfico."
        }
      ]
    },
    {
      id: "constantes",
      icon: "π",
      title: "Constantes",
      summary: "Constantes físicas, de materiales y propias del documento.",
      keywords: ["constante", "acero", "agua", "gravedad", "material"],
      blocks: [
        {
          type: "paragraph",
          text: "**Fórmulas → Constantes…** abre el catálogo de constantes integradas: gravedad, constantes físicas, propiedades de acero, aluminio, hormigón, agua, presión atmosférica y número áureo."
        },
        {
          type: "list",
          items: [
            "Pulsa una constante integrada para insertarla en la región activa.",
            "Las constantes propias se añaden desde el mismo diálogo.",
            "Se guardan en el documento y se evalúan antes que las regiones.",
            "Las constantes se muestran con una fuente y color propios."
          ]
        }
      ]
    },
    {
      id: "idiomas",
      icon: "🌐",
      title: "Idiomas y ortografía",
      summary: "Interfaz multilingüe y corrector Hunspell.",
      keywords: ["idioma", "ortografía", "hunspell", "diccionario", "árabe", "chino"],
      blocks: [
        {
          type: "paragraph",
          text: "La interfaz está disponible en español, inglés, francés, hindi, árabe y chino simplificado. El árabe activa la dirección RTL."
        },
        {
          type: "list",
          items: [
            "El idioma del documento se elige en **Página → Página…** o desde la barra de la región de texto.",
            "El corrector ortográfico usa diccionarios Hunspell para español, inglés, francés, hindi y árabe.",
            "Las palabras dudosas se subrayan; el clic derecho ofrece sugerencias, omitir o añadir al diccionario.",
            "El diccionario personalizado se guarda en el documento.",
            "Las marcas del corrector no se imprimen ni se incluyen en el HTML guardado."
          ]
        }
      ]
    },
    {
      id: "exportar",
      icon: "📤",
      title: "Exportación",
      summary: "LaTeX, ODT, PDF raster, PDF vectorial y gráficos.",
      keywords: ["exportar", "latex", "odt", "pdf", "vectorial", "miktex"],
      blocks: [
        {
          type: "list",
          items: [
            "**LaTeX:** genera un `.tex` con los resultados calculados.",
            "**ODT:** fórmulas como objetos Math embebidos, tablas, imágenes y encabezado/pie.",
            "**PDF raster:** usa KaTeX, html2canvas y jsPDF; el texto no es seleccionable.",
            "**PDF vectorial:** compila con XeLaTeX, LuaLaTeX o pdfLaTeX; requiere MiKTeX o TeX Live.",
            "**SVG/PNG:** exporta gráficos 2D."
          ]
        },
        {
          type: "paragraph",
          text: "El motor LaTeX se busca primero en `INGECALC_LATEX_ENGINE`, después en `PATH` y finalmente en las rutas habituales de MiKTeX en Windows."
        },
        {
          type: "note",
          title: "Seguridad",
          text: "La compilación usa `-no-shell-escape`, pero el motor LaTeX puede leer archivos locales. No compiles documentos de terceros sin revisarlos."
        }
      ]
    },
    {
      id: "atajos",
      icon: "⌘",
      title: "Atajos de teclado",
      summary: "Los atajos más útiles para documento, edición y ayuda.",
      keywords: ["atajo", "teclado", "ctrl", "f1", "tab", "enter"],
      blocks: [
        {
          type: "table",
          headers: ["Atajo", "Acción"],
          rows: [
            ["Ctrl+N", "Nuevo documento"],
            ["Ctrl+O", "Abrir"],
            ["Ctrl+S", "Guardar"],
            ["Ctrl+P", "Imprimir"],
            ["Ctrl+Z / Ctrl+Y", "Deshacer / rehacer"],
            ["Ctrl+D", "Duplicar selección"],
            ["F9", "Recalcular"],
            ["F1", "Abrir la ayuda"],
            ["Ctrl+Espacio", "Abrir catálogo de autocompletado"],
            ["Tab / Shift+Tab", "Navegar entre regiones o insertar sugerencia"],
            ["Esc", "Cerrar menú o diálogo"],
            ["Ctrl+= / Ctrl+- / Ctrl+0", "Zoom"]
          ]
        },
        {
          type: "note",
          title: "En matemática",
          text: "`Tab` inserta la sugerencia seleccionada; `Enter` sin navegación solo cierra el menú. Así el autocompletado nunca cambia el texto por accidente."
        }
      ]
    },
    {
      id: "problemas",
      icon: "🛠",
      title: "Solución de problemas",
      summary: "Errores frecuentes y cómo resolverlos.",
      keywords: ["problema", "error", "latex", "borrador", "pdf", "no funciona"],
      blocks: [
        {
          type: "list",
          items: [
            "**No aparece el motor LaTeX:** instala MiKTeX/TeX Live o define `INGECALC_LATEX_ENGINE` con la ruta del ejecutable.",
            "**Una fórmula no evalúa:** comprueba que todas las variables estén definidas y que ningún nombre se defina dos veces.",
            "**Una caja no cabe:** se traslada a la página siguiente. Si hay cajas fijadas, pueden bloquear el reflujo.",
            "**No veo el borrador anterior:** el identificador de la aplicación cambió a `org.ingecalc.app` y crea un directorio de datos nuevo.",
            "**El PDF raster no permite seleccionar texto:** usa **PDF LaTeX** para obtener texto vectorial.",
            "**El autocompletado no inserta:** navega con `↑`/`↓` y pulsa `Tab`, o haz clic en la sugerencia."
          ]
        },
        {
          type: "note",
          title: "¿Sigue fallando?",
          text: "Abre una incidencia con la versión, el sistema operativo, los pasos para reproducirlo y el mensaje de error. Para problemas de seguridad, usa el aviso privado de GitHub."
        }
      ]
    },
    {
      id: "formato",
      icon: "{}",
      title: "Formato y compatibilidad",
      summary: "Documentos `.smj`, formato nuevo y archivos antiguos.",
      keywords: ["formato", "smj", "json", "compatibilidad", "ingecalc-json", "mathdoc-json"],
      blocks: [
        {
          type: "paragraph",
          text: "Los documentos usan JSON con extensión `.smj`. El formato actual es `ingecalc-json` y la versión del esquema es `1`."
        },
        {
          type: "code",
          title: "Cabecera de un documento",
          text: "{\n  \"format\": \"ingecalc-json\",\n  \"version\": 1\n}"
        },
        {
          type: "note",
          title: "Compatibilidad",
          text: "El cargador acepta documentos antiguos con `mathdoc-json` y los migra al formato actual al guardarlos. Las variables de entorno `MATHDOC_*` se aceptan como respaldo."
        }
      ]
    },
    {
      id: "acerca",
      icon: "ℹ",
      title: "Acerca de IngeCalc",
      summary: "Versión, licencia y tecnologías.",
      keywords: ["acerca", "versión", "licencia", "rust", "tauri", "react"],
      blocks: [
        {
          type: "paragraph",
          text: "IngeCalc es un editor open source de hojas de cálculo técnico para ingeniería civil, eléctrica, mecánica, hidráulica y otras disciplinas."
        },
        {
          type: "list",
          items: [
            "Núcleo matemático y documental en Rust.",
            "Escritorio con Tauri 2 y React.",
            "Edición matemática con MathLive y renderizado con KaTeX.",
            "Texto enriquecido con TipTap/ProseMirror.",
            "Licencia GPL-3.0-or-later."
          ]
        },
        {
          type: "note",
          title: "Versión",
          text: "Versión 0.1.2. Repositorio: github.com/marcel1983/ingecalc."
        }
      ]
    }
  ]
};

const en: ManualContent = {
  title: "IngeCalc Manual",
  subtitle: "User guide and quick reference",
  version: "0.1.2",
  sections: [
    {
      id: "inicio",
      icon: "🧭",
      title: "Getting started",
      summary: "Create, open and save an engineering worksheet.",
      keywords: ["introduction", "start", "new", "open", "save"],
      blocks: [
        {
          type: "paragraph",
          text: "IngeCalc organizes work into pages and regions. A region can contain math, rich text, an image or a plot."
        },
        {
          type: "list",
          items: [
            "Open a `.smj` document with **Open** (`Ctrl+O`) or create one with **New** (`Ctrl+N`).",
            "Click an empty area of the page to create a math region.",
            "Type an expression and press **Recalculate** (`F9`) to refresh results.",
            "Save with `Ctrl+S`. Autosave keeps a recoverable draft."
          ]
        },
        {
          type: "code",
          title: "First formula",
          text: "x := 3\nf(t) := t^2\nf(x) + 1"
        },
        {
          type: "note",
          title: "Help anywhere",
          text: "Press `F1` or click the `?` button in the top bar to open this manual from anywhere in the app."
        }
      ]
    },
    {
      id: "interfaz",
      icon: "🪟",
      title: "Interface",
      summary: "Ribbon, formula panel, pages and status bar.",
      keywords: ["ribbon", "toolbar", "panel", "tabs", "status"],
      blocks: [
        {
          type: "paragraph",
          text: "The top bar groups commands in tabs: **Home**, **Insert**, **Page**, **Formulas**, **Export** and **Help**."
        },
        {
          type: "list",
          items: [
            "**Home:** clipboard, regions, document and recalculation.",
            "**Insert:** regions, header/footer, functions, units and constants.",
            "**Page:** size, margins, result units, numbers, zoom and view.",
            "**Formulas:** assignment `:=`, side panel and math catalogs.",
            "**Export:** LaTeX, ODT, raster PDF and vector PDF.",
            "**Help:** manual, quick start, shortcuts, troubleshooting and about IngeCalc."
          ]
        },
        {
          type: "paragraph",
          text: "The formula side panel lets you search and insert functions, operators, symbols, units and plots into the active region. Hide it with the panel button in the top bar."
        },
        {
          type: "note",
          title: "Status bar",
          text: "The right side of the top bar shows the document status, theme, interface language and help access."
        }
      ]
    },
    {
      id: "documento",
      icon: "📄",
      title: "Document and regions",
      summary: "Pages, regions, reflow, margins, images and tables.",
      keywords: ["page", "region", "text", "image", "table", "margins", "reflow"],
      blocks: [
        {
          type: "list",
          items: [
            "Each document has A4 or Letter pages with editable margins, header and footer.",
            "Math and text regions fit their content. Images and plots can be resized with the bottom-right handle.",
            "When a box grows, boxes below reflow downward. If they do not fit, they move to the next page.",
            "Boxes stay inside printable margins. Header and footer regions are the exception.",
            "Empty boxes are removed on blur. `Ctrl+Z` restores them.",
            "`Ctrl+click` adds or removes regions from the selection; dragging on the background creates a rectangular selection.",
            "`Ctrl+D` duplicates the selection; `Ctrl+C`, `Ctrl+X` and `Ctrl+V` manage the region clipboard.",
            "The pin button locks positions. A pinned box cannot move, but its content remains editable.",
            "**Insert → Image** adds a resizable image. **Insert → Table** creates an editable 3×3 table.",
            "The **ƒx** button in a text region inserts inline KaTeX math."
          ]
        },
        {
          type: "note",
          title: "Enter in math",
          text: "Inside a math region, `Enter` creates another math region directly below and moves the cursor there without overlapping."
        },
        {
          type: "paragraph",
          text: "Add, reorder and delete pages from the **Page** tab. The `{page}` token in the header or footer is replaced by the page number."
        }
      ]
    },
    {
      id: "matematica",
      icon: "∑",
      title: "Math and variables",
      summary: "Variables, functions, operators and dependency evaluation.",
      keywords: ["variable", "function", "definition", "formula", "evaluate"],
      blocks: [
        {
          type: "code",
          title: "Definitions",
          text: "x := 3\nf(t) := t^2\nf(x) + 1"
        },
        {
          type: "list",
          items: [
            "`:=` defines a variable or a function.",
            "Definitions are evaluated by dependency, not by their order on the page.",
            "Each name has a single definition. There are no redefinitions, nested scopes or recursion.",
            "A trailing `=` in a region marks evaluation and shows the result.",
            "Operators include `+`, `-`, `*`, `/`, `^`, comparisons and `:=`."
          ]
        },
        {
          type: "note",
          title: "Name errors",
          text: "If you see “unknown symbol”, create the name with `name := value` or select the definition suggestion in autocomplete."
        }
      ]
    },
    {
      id: "autocompletado",
      icon: "⌨",
      title: "Autocomplete and literal input",
      summary: "Suggestions, shortcuts and writing without autocorrection.",
      keywords: ["suggestion", "complete", "autocorrect", "literal", "tab"],
      blocks: [
        {
          type: "paragraph",
          text: "Typing the start of a variable, function, constant or operator opens a menu grouped by kind."
        },
        {
          type: "list",
          items: [
            "If the name does not exist, it suggests `name :=` to create the variable.",
            "If the variable already exists, it is only offered for use; redefining it is not allowed.",
            "Defined functions suggest `f(...) =` first so they can be evaluated.",
            "`↑`/`↓` navigate suggestions.",
            "`Tab` or a click inserts the selected suggestion.",
            "`Enter` without navigation only closes the menu, without autocorrecting.",
            "`Ctrl+Space` opens the full catalog and `Esc` closes the menu."
          ]
        },
        {
          type: "note",
          title: "Literal input",
          text: "No automatic conversions such as `Re → ℜ`, `alpha → α` or `<= → ≤` are applied. Symbols are inserted from the palette or autocomplete. `/` still creates a fraction as a math structure."
        }
      ]
    },
    {
      id: "unidades",
      icon: "📐",
      title: "Units",
      summary: "Conversion, localized units, custom units and result systems.",
      keywords: ["unit", "conversion", "meter", "inch", "temperature", "SI"],
      blocks: [
        {
          type: "code",
          title: "Conversions",
          text: "d := 2\"m\" + 30\"cm\"\nt := 2\"s\"\nv := d/t\nT := 20\"°C\" + 5\"K\""
        },
        {
          type: "list",
          items: [
            "Addition and subtraction convert to the left-hand unit.",
            "Multiplication and division combine and simplify dimensions.",
            "SI, imperial and localized units are supported.",
            "**Page → Units…** chooses the result system, customizes quantities or keeps the original units.",
            "Custom units are defined with `unit(\"name\") := expression`."
          ]
        },
        {
          type: "code",
          title: "Custom unit",
          text: "unit(\"my_inch\") := 2.54 cm\nunit(\"my_foot\") := 12 my_inch\n0\"cm\" + 1\"my_foot\""
        },
        {
          type: "note",
          title: "Limitation",
          text: "Custom units must be linear and cannot redefine an existing unit. Offsets are only available for built-in units."
        }
      ]
    },
    {
      id: "matrices",
      icon: "▦",
      title: "Matrices and vectors",
      summary: "Construction, operations, inverse, LU, QR and eigenvalues.",
      keywords: ["matrix", "vector", "determinant", "inverse", "lu", "qr", "eigenvalues"],
      blocks: [
        {
          type: "code",
          title: "Basic operations",
          text: "A := [1,2;3,4]\nB := [2,0;1,2]\nA*B\ndet(A)\nA^-1\nA[2,1]\nidentity(3)\nsolve([2,1;1,3],[1;2])"
        },
        {
          type: "paragraph",
          text: "Indexing is 1-based: `A[2,1]` is row 2, column 1; `A[1]` returns the first row."
        },
        {
          type: "code",
          title: "Advanced matrices",
          text: "lu(A)\nqr(A)\neigenvalues([2,1;1,2])\nrank(A)\nrref(A)\nrange(1,5)\nreshape(range(1,6), 2, 3)\ndot([1,2,3],[4,5,6])\ncross([1,0,0],[0,1,0])\nnorm([3,4])"
        },
        {
          type: "note",
          title: "Limitation",
          text: "Matrices are numeric (`f64`). They do not support heterogeneous quantities, dynamic resizing or nonlinear systems."
        }
      ]
    },
    {
      id: "simbolico",
      icon: "∫",
      title: "Symbolic math",
      summary: "Simplify, expand, factor, differentiate, integrate and solve.",
      keywords: ["symbolic", "derivative", "integral", "factor", "solve", "roots"],
      blocks: [
        {
          type: "code",
          title: "Examples",
          text: "simplify(x + 0 + x)\nexpand((x+1)^2)\nfactor(x^2 - 4)\ndiff(x^3 + 2*x, x)\nintegrate(x^2, x)\nintegral(sin(x), x, 0, pi)\nsolve(x^2 - 5*x + 6, x)\npolyroots([1, 0, 1])"
        },
        {
          type: "paragraph",
          text: "The engine covers simplification, expansion, factoring, substitution, derivatives, elementary antiderivatives, numeric definite integrals, equations and roots."
        },
        {
          type: "note",
          title: "Scope",
          text: "It is not a full CAS. Advanced transformations or unsupported equations may return a symbolic expression or an error."
        }
      ]
    },
    {
      id: "flujo",
      icon: "↻",
      title: "Control flow",
      summary: "Conditionals, loops, sums, products and local variables.",
      keywords: ["if", "for", "while", "sum", "product", "loop", "conditional"],
      blocks: [
        {
          type: "code",
          title: "Examples",
          text: "if(x > 0, 1, 0)\nfor(i, 1, 10, i^2)\nfor(v, [1,2,3], v^2)\nsum(i, 1, 100, i)\nproduct(i, 1, 5, i)\nline(x := 0, while(x < 5, x := x + 1), x)\ntrace(\"value\", x^2)\nerror(\"message\")"
        },
        {
          type: "list",
          items: [
            "`if(condition, true_value, false_value)`.",
            "`for` iterates over a range or matrix elements.",
            "`sum` and `product` accumulate an expression over a range.",
            "`while` repeats while a condition holds.",
            "`line` evaluates a sequence of expressions with local variables.",
            "`trace` returns a value for debugging and `error` stops evaluation."
          ]
        },
        {
          type: "note",
          title: "Limit",
          text: "Loops are limited to 100,000 iterations to avoid blocking."
        }
      ]
    },
    {
      id: "graficos",
      icon: "📈",
      title: "Plots",
      summary: "2D, parametric, implicit and 3D surface plots.",
      keywords: ["plot", "2d", "3d", "surface", "axis", "units"],
      blocks: [
        {
          type: "list",
          items: [
            "**Explicit:** `sin(x)`.",
            "**Parametric:** `x(t)=cos(t)`, `y(t)=sin(t)`, with a `t` range.",
            "**Implicit:** `x^2+y^2-4` for a circle.",
            "**3D surface:** `z=f(x,y)` with X/Y domains, rotation, zoom and mesh.",
            "**With units:** `x * 1 m` keeps the unit on the Y axis."
          ]
        },
        {
          type: "note",
          title: "Export",
          text: "Plots are recalculated when opened and can be exported as SVG or PNG. LaTeX and ODT export a note instead of the plot."
        }
      ]
    },
    {
      id: "constantes",
      icon: "π",
      title: "Constants",
      summary: "Physical, material and document-specific constants.",
      keywords: ["constant", "steel", "water", "gravity", "material"],
      blocks: [
        {
          type: "paragraph",
          text: "**Formulas → Constants…** opens the built-in catalog: gravity, physical constants, steel, aluminum, concrete and water properties, atmospheric pressure and the golden ratio."
        },
        {
          type: "list",
          items: [
            "Click a built-in constant to insert it into the active region.",
            "Custom constants are added from the same dialog.",
            "They are stored in the document and evaluated before regions.",
            "Constants use their own font and color."
          ]
        }
      ]
    },
    {
      id: "idiomas",
      icon: "🌐",
      title: "Languages and spelling",
      summary: "Multilingual interface and Hunspell spellcheck.",
      keywords: ["language", "spelling", "hunspell", "dictionary", "arabic", "chinese"],
      blocks: [
        {
          type: "paragraph",
          text: "The interface is available in Spanish, English, French, Hindi, Arabic and Simplified Chinese. Arabic enables RTL direction."
        },
        {
          type: "list",
          items: [
            "The document language is selected in **Page → Page…** or from the text region toolbar.",
            "Spellcheck uses Hunspell dictionaries for Spanish, English, French, Hindi and Arabic.",
            "Suspicious words are underlined; right-click offers suggestions, ignore or add to dictionary.",
            "The personal dictionary is stored in the document.",
            "Spellcheck marks are not printed or included in saved HTML."
          ]
        }
      ]
    },
    {
      id: "exportar",
      icon: "📤",
      title: "Export",
      summary: "LaTeX, ODT, raster PDF, vector PDF and plots.",
      keywords: ["export", "latex", "odt", "pdf", "vector", "miktex"],
      blocks: [
        {
          type: "list",
          items: [
            "**LaTeX:** generates a `.tex` file with calculated results.",
            "**ODT:** formulas as embedded Math objects, tables, images and header/footer.",
            "**Raster PDF:** uses KaTeX, html2canvas and jsPDF; text is not selectable.",
            "**Vector PDF:** compiles with XeLaTeX, LuaLaTeX or pdfLaTeX; requires MiKTeX or TeX Live.",
            "**SVG/PNG:** exports 2D plots."
          ]
        },
        {
          type: "paragraph",
          text: "The LaTeX engine is searched first in `INGECALC_LATEX_ENGINE`, then in `PATH`, and finally in common MiKTeX paths on Windows."
        },
        {
          type: "note",
          title: "Security",
          text: "Compilation uses `-no-shell-escape`, but the LaTeX engine may read local files. Do not compile third-party documents without reviewing them."
        }
      ]
    },
    {
      id: "atajos",
      icon: "⌘",
      title: "Keyboard shortcuts",
      summary: "The most useful shortcuts for documents, editing and help.",
      keywords: ["shortcut", "keyboard", "ctrl", "f1", "tab", "enter"],
      blocks: [
        {
          type: "table",
          headers: ["Shortcut", "Action"],
          rows: [
            ["Ctrl+N", "New document"],
            ["Ctrl+O", "Open"],
            ["Ctrl+S", "Save"],
            ["Ctrl+P", "Print"],
            ["Ctrl+Z / Ctrl+Y", "Undo / redo"],
            ["Ctrl+D", "Duplicate selection"],
            ["F9", "Recalculate"],
            ["F1", "Open help"],
            ["Ctrl+Space", "Open autocomplete catalog"],
            ["Tab / Shift+Tab", "Navigate regions or insert suggestion"],
            ["Esc", "Close menu or dialog"],
            ["Ctrl+= / Ctrl+- / Ctrl+0", "Zoom"]
          ]
        },
        {
          type: "note",
          title: "In math",
          text: "`Tab` inserts the selected suggestion; `Enter` without navigation only closes the menu. Autocomplete never changes text accidentally."
        }
      ]
    },
    {
      id: "problemas",
      icon: "🛠",
      title: "Troubleshooting",
      summary: "Common errors and how to solve them.",
      keywords: ["problem", "error", "latex", "draft", "pdf", "not working"],
      blocks: [
        {
          type: "list",
          items: [
            "**LaTeX engine not found:** install MiKTeX/TeX Live or set `INGECALC_LATEX_ENGINE` to the executable path.",
            "**A formula does not evaluate:** check that every variable is defined and that no name is defined twice.",
            "**A box does not fit:** it moves to the next page. Pinned boxes may block reflow.",
            "**I cannot see the previous draft:** the app identifier changed to `org.ingecalc.app`, so it uses a new data directory.",
            "**Raster PDF text cannot be selected:** use **LaTeX PDF** for vector text.",
            "**Autocomplete does not insert:** navigate with `↑`/`↓` and press `Tab`, or click the suggestion."
          ]
        },
        {
          type: "note",
          title: "Still failing?",
          text: "Open an issue with the version, operating system, steps to reproduce and the error message. For security issues, use GitHub private reporting."
        }
      ]
    },
    {
      id: "formato",
      icon: "{}",
      title: "Format and compatibility",
      summary: "`.smj` documents, new format and legacy files.",
      keywords: ["format", "smj", "json", "compatibility", "ingecalc-json", "mathdoc-json"],
      blocks: [
        {
          type: "paragraph",
          text: "Documents use JSON with a `.smj` extension. The current format is `ingecalc-json` and the schema version is `1`."
        },
        {
          type: "code",
          title: "Document header",
          text: "{\n  \"format\": \"ingecalc-json\",\n  \"version\": 1\n}"
        },
        {
          type: "note",
          title: "Compatibility",
          text: "The loader accepts legacy `mathdoc-json` documents and migrates them to the current format when saved. `MATHDOC_*` environment variables are accepted as fallback."
        }
      ]
    },
    {
      id: "acerca",
      icon: "ℹ",
      title: "About IngeCalc",
      summary: "Version, license and technologies.",
      keywords: ["about", "version", "license", "rust", "tauri", "react"],
      blocks: [
        {
          type: "paragraph",
          text: "IngeCalc is an open-source engineering worksheet editor for civil, electrical, mechanical, hydraulic and other engineering disciplines."
        },
        {
          type: "list",
          items: [
            "Math and document core in Rust.",
            "Desktop shell with Tauri 2 and React.",
            "Math editing with MathLive and rendering with KaTeX.",
            "Rich text with TipTap/ProseMirror.",
            "GPL-3.0-or-later license."
          ]
        },
        {
          type: "note",
          title: "Version",
          text: "Version 0.1.2. Repository: github.com/marcel1983/ingecalc."
        }
      ]
    }
  ]
};

export function getManualContent(language: string): ManualContent {
  return language.toLowerCase().startsWith("es") ? es : en;
}

export function normalizeManualSearch(value: string): string {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLocaleLowerCase();
}

function sectionSearchText(section: ManualSection): string {
  const blocks = section.blocks.flatMap((block) => {
    switch (block.type) {
      case "paragraph":
        return [block.text];
      case "code":
      case "note":
        return [block.text, block.title ?? ""];
      case "list":
        return block.items;
      case "table":
        return [...block.headers, ...block.rows.flat()];
    }
  });
  return normalizeManualSearch(
    [section.title, section.summary, ...section.keywords, ...blocks].join(" ")
  );
}

export function searchManual(
  content: ManualContent,
  query: string
): ManualSection[] {
  const needle = normalizeManualSearch(query.trim());
  if (!needle) {
    return content.sections;
  }
  return content.sections.filter((section) =>
    sectionSearchText(section).includes(needle)
  );
}
