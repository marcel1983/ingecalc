# Ejemplos de IngeCalc

Colección de documentos `.smj` listos para abrir con **Abrir** en la aplicación o para validar desde la CLI.

| Archivo | Páginas | Tema | Resultados destacados |
|---|---:|---|---|
| [`fibonacci.smj`](fibonacci.smj) | 2 | Sucesión de Fibonacci, fórmula de Binet, identidad de Cassini y número áureo | `fib(50) = 12586269025`, `fib(10)² − fib(9)·fib(11) = −1`, `fib(21)/fib(20) ≈ φ` |
| [`ecuaciones.smj`](ecuaciones.smj) | 3 | Sistemas lineales 2×2 y 3×3, ecuación cuadrática y raíces complejas | `solve(A,b) = [0.2; 0.6]`, `solve(M,v) = [0.5; 0; 1.5]`, raíces `2` y `1`, raíces `i` y `−i` |
| [`matrices.smj`](matrices.smj) | 2 | Suma, producto, potencia, determinante, inversa, transpuesta, indexación e identidad | `A·B = [4,4;10,8]`, `B·A = [2,4;7,10]`, `det(A) = −2`, `A⁻¹·A = I` |
| [`unidades.smj`](unidades.smj) | 2 | Conversión automática, unidades localizadas, unidades propias y resultados en sistema métrico | `2 m + 30 cm = 2.3 m`, `d/t = 1.15 m/s`, `20 °C + 5 K = 25°C`, `4856 in + 45 m = 168.3424 m`, `0 cm + 1 mi_pie = 0.3048 m` |
| [`complejos.smj`](complejos.smj) | 2 | Fracciones exactas, complejos, conjugado y módulo | `1/3 + 1/6 = 1/2`, `(2+3i)/(1−i) = −0.5 + 2.5i`, `\|3+4i\| = 5` |
| [`graficos.smj`](graficos.smj) | 2 | Gráficos explícitos, paramétricos e implícitos | seno/coseno, circunferencia paramétrica, Lissajous 3:2, circunferencia implícita y corazón algebraico |
| [`control-flujo.smj`](control-flujo.smj) | 1 | Condicionales, comparaciones, bucles `for`, `sum`, `product`, `while`, `line`, iteración de matrices y `if` anidado | `if(2 < 3, 10, 20) = 10`, `for(i, 1, 5, i²) = [1, 4, 9, 16, 25]`, `sum(i, 1, 5, i²) = 55`, `product(i, 1, 5, i) = 120`, `line(x := 0, while(x < 5, x := x + 1), x) = 5`, `for(v, [1,2,3], v²) = [1, 4, 9]` |
| [`simbolico.smj`](simbolico.smj) | 1 | Simplificación, expansión, factorización, derivadas, integrales y resolución de ecuaciones | `simplify(x + 0 + x) = 2·x`, `diff(x³ + 2x, x) = 3x² + 2`, `integrate(x², x) = x³/3`, `integral(sin(x), 0, π) = 2`, `solve(x² − 5x + 6, x) = {2, 3}`, `polyroots([1,0,1]) = ±i` |
| [`matrices-avanzadas.smj`](matrices-avanzadas.smj) | 1 | LU, QR, autovalores, rango, rangos, vectores, producto escalar/vectorial y norma | `eigenvalues([2,1;1,2]) = [1, 3]`, `rank([1,2;3,4]) = 2`, `range(1,5) = [1,2,3,4,5]`, `dot([1,2,3],[4,5,6]) = 32`, `cross([1,0,0],[0,1,0]) = [0,0,1]` |
| [`graficos-3d.smj`](graficos-3d.smj) | 1 | Superficie 3D `z = sin(x)·cos(y)` y gráfico 2D con unidades en el eje Y | Superficie interactiva con rotación/zoom y serie `y = x·1 m` |
| [`avanzado.smj`](avanzado.smj) | 4 | Documento integrador: cálculo simbólico y resolución, matrices avanzadas, gráficos 3D con unidades y control de flujo | Resumen de las cuatro áreas en un solo archivo |

## Abrir en la aplicación

1. Ejecuta IngeCalc.
2. Pulsa **Abrir** en la barra de acceso rápido.
3. Selecciona uno de los archivos de esta carpeta.

Cada documento está pensado para verse a 100 % de zoom. Los gráficos se recalculan al abrirlos y se pueden mover, ampliar o exportar como SVG/PNG.

## Validar todos los ejemplos

Desde la raíz del repositorio:

```bash
for file in examples/fibonacci.smj examples/ecuaciones.smj examples/matrices.smj \
            examples/unidades.smj examples/complejos.smj examples/graficos.smj \
            examples/control-flujo.smj \
            examples/simbolico.smj examples/matrices-avanzadas.smj \
            examples/graficos-3d.smj examples/avanzado.smj; do
  cargo run -p ingecalc-cli -- validate "$file"
  cargo run -p ingecalc-cli -- eval-document "$file"
done
```

En PowerShell:

```powershell
Get-ChildItem examples/*.smj |
  Where-Object { $_.Name -ne 'sample.smj' } |
  ForEach-Object {
    cargo run -p ingecalc-cli -- validate $_.FullName
    cargo run -p ingecalc-cli -- eval-document $_.FullName
  }
```

`eval-document` devuelve un JSON con el valor o el error de cada región matemática, en orden de página y de arriba hacia abajo.

## Sintaxis que muestran los ejemplos

### Variables y funciones

```text
phi := (1+sqrt(5))/2
fib(n) := round((phi^n - psi^n)/sqrt(5))
fib(20)
```

Las definiciones se evalúan por dependencias, no por orden de aparición. Un nombre solo puede definirse una vez.

### Matrices

```text
A := [1,2;3,4]
B := [2,0;1,2]
A*B
det(A)
A^-1
A[2,1]
identity(3)
solve([2,1;1,3],[1;2])
```

La indexación es 1-based: `A[2,1]` es la fila 2, columna 1; `A[1]` devuelve la primera fila.

### Unidades

```text
d := 2"m" + 30"cm"
t := 2"s"
v := d/t
x := v*t
T := 20"°C" + 5"K"
```

Las unidades localizadas se escriben entre comillas: `2"metro"`, `1"pulgada"`, `1"grado Celsius"`. Las unidades visibles se muestran en azul, tanto si se insertan desde la paleta como si se escriben a mano.

Los resultados con unidades se presentan por defecto en el **sistema métrico**. En **Página → Unidades…** se puede elegir imperial, personalizar la unidad de cada magnitud o mantener las unidades originales del cálculo.

Las unidades propias se definen con `unit`:

```text
unit("mi_pulgada") := 2.54 cm
unit("mi_pie") := 12 mi_pulgada
0"cm" + 1"mi_pie"
```

### Gráficos

Cada serie de una región `plot` puede ser:

- explícita: `sin(x)`;
- paramétrica: `x(t)=cos(t)`, `y(t)=sin(t)`, con rango de `t`;
- implícita: `x^2+y^2-4` para representar `x² + y² = 4`;
- superficie 3D: `z=f(x,y)` con dominios X/Y editables, rotación y zoom;
- con unidades: `x * 1 m` conserva la unidad en el eje Y.

### Cálculo simbólico y resolución

```text
simplify(x + 0 + x)
expand((x+1)^2)
factor(x^2 - 4)
diff(x^3 + 2*x, x)
integrate(x^2, x)
integral(sin(x), x, 0, pi)
solve(x^2 - 5*x + 6, x)
polyroots([1, 0, 1])
```

### Control de flujo y programación

```text
if(x > 0, 1, 0)
for(i, 1, 10, i^2)
for(v, [1,2,3], v^2)
sum(i, 1, 100, i)
product(i, 1, 5, i)
line(x := 0, while(x < 5, x := x + 1), x)
trace("valor", x^2)
error("mensaje")
```

### Matrices y vectores avanzados

```text
lu(A)
qr(A)
eigenvalues(A)
rank(A)
rref(A)
range(1, 5)
reshape(range(1, 6), 2, 3)
dot([1,2,3], [4,5,6])
cross([1,0,0], [0,1,0])
norm([3,4])
```

## Notas

- Los documentos usan página de 850×1100 px con márgenes de 48 px y pie `Página {page}`.
- Los resultados usan por defecto **4 decimales**, notación automática y fracciones exactas cuando corresponde. **Página → Números…** permite cambiar notación, decimales, ceros finales y fracciones por documento.
- Las regiones de texto muestran su barra de formato solo al seleccionarlas; al imprimir nunca se muestra.
- Los ejemplos no cargan plugins ni definen unidades o funciones con nombres duplicados.
- Licencia: GPL-3.0-or-later, la misma del proyecto.
