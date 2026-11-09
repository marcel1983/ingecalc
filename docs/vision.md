# Visión de IngeCalc

## Qué es IngeCalc

IngeCalc es un editor open source de hojas de cálculo técnico para ingeniería. Su
objetivo es que un profesional pueda pasar de una expresión matemática a una
memoria de cálculo documentada sin cambiar de herramienta.

El documento `.smj` combina matemática, texto, unidades, gráficos y resultados en
un formato abierto y versionado.

## Para quién

- Ingenieros civiles, eléctricos, mecánicos, hidráulicos y de otras disciplinas.
- Estudiantes y docentes que necesitan cálculo reproducible.
- Oficinas técnicas que preparan memorias y comprobaciones.
- Desarrolladores que quieren extender el motor mediante plugins.

## Principios

1. **Local-first:** los documentos son del usuario.
2. **Formato abierto:** JSON legible y esquema publicado.
3. **Cálculo trazable:** dependencias explícitas y una definición por nombre.
4. **Sin autocorrección:** las sugerencias no cambian identificadores.
5. **Multidisciplinar:** una misma hoja sirve para estructuras, circuitos,
   máquinas y fluidos.
6. **Documentación integrada:** la ayuda está dentro de la aplicación.
7. **Exportación útil:** LaTeX, ODT y PDF, no solo una vista previa.

## Diferenciadores

- Núcleo matemático en Rust y escritorio Tauri.
- Edición matemática WYSIWYG con MathLive y KaTeX.
- Unidades de ingeniería con conversión dimensional.
- Cálculo simbólico y numérico en el mismo documento.
- Exportación a formatos abiertos y PDF vectorial.
- Compatibilidad con documentos de la etapa anterior del proyecto.

## Criterios de éxito

- Abrir, calcular y guardar una hoja técnica sin fricción.
- Exportar una memoria con resultados, unidades y formato consistente.
- Detectar errores de cálculo con pruebas de regresión y validación.
- Mantener el formato `.smj` compatible entre versiones menores.
- Tener una comunidad que pueda contribuir sin depender de una sola persona.

## Fuera del alcance inmediato

- Colaboración en la nube en tiempo real.
- CAS simbólico completo.
- Marketplace de plugins.
- Bibliotecas de normas integradas.
- Edición móvil.
