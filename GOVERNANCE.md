# Gobernanza de IngeCalc

Este documento describe cómo se toman las decisiones en el proyecto, quién puede
fusionar cambios y cómo se incorporan nuevos mantenedores.

## Modelo inicial

IngeCalc comienza con un modelo **BDFL con consejo**: el fundador conserva la
decisión final y consulta a los mantenedores antes de cambios importantes. Cuando
existan al menos cinco mantenedores activos, el proyecto podrá pasar a un comité
técnico con votación por mayoría.

## Roles

| Rol | Cómo se obtiene | Permisos |
|---|---|---|
| Usuario | — | Abrir issues y participar en Discussions |
| Contribuidor | Un PR fusionado | Reconocimiento en `CONTRIBUTORS` |
| Triager | Participación constante | Clasificar issues y pedir información |
| Revisor | Cinco PRs de calidad en un área | Aprobar PRs de su área |
| Mantenedor | Propuesta de un mantenedor y votación | Fusionar, publicar y decidir |

Los equipos de GitHub son:

- `maintainers`: administración, releases y seguridad.
- `core`: crates Rust, parser, evaluador, formato y plugins.
- `gui`: React, Tauri y experiencia de usuario.
- `docs`: manual, guías, traducciones y ADRs.
- `engineering`: validación técnica y ejemplos.
- `triage`: clasificación de incidencias.

Mientras el repositorio esté en la cuenta personal `@marcel1983`, estos roles son
organizativos y `CODEOWNERS` apunta al mantenedor. Al trasladar el proyecto a una
organización, se crearán los equipos y se actualizará `CODEOWNERS`.

## Toma de decisiones

### Cambios ordinarios

Se resuelven en el PR con revisión normal. El mantenedor de área puede fusionar
cuando el CI esté en verde y exista la aprobación requerida.

### Cambios de arquitectura, formato o API

Requieren una **RFC**:

1. Abrir una Discussion con la etiqueta `rfc`.
2. Exponer el problema, las alternativas y el impacto.
3. Dejar un periodo mínimo de comentarios de 14 días.
4. Registrar la decisión final como ADR en `docs/adr/`.

### Cambios de gobernanza

Se proponen mediante PR a este documento. Requieren la aprobación de al menos dos
mantenedores y un periodo de comentarios de 7 días.

## Nombramiento de mantenedores

Un revisor puede ser propuesto como mantenedor cuando:

- haya realizado contribuciones sostenidas durante al menos tres meses;
- conozca una parte relevante del código o de la validación;
- haya revisado PRs con criterio técnico;
- acepte las responsabilidades de revisión y release.

La propuesta se discute en privado entre los mantenedores y se anuncia en
Discussions cuando se aprueba.

## Conflictos

Los desacuerdos técnicos se resuelven con argumentos, pruebas y referencias. Si no
hay consenso, decide el mantenedor del área y, en última instancia, el BDFL. Los
problemas de conducta se tratan según `CODE_OF_CONDUCT.md`.

## Contribuciones y DCO

No se requiere CLA. Cada commit debe incluir la firma del Developer Certificate of
Origin:

```bash
git commit -s -m "feat(units): add pressure offset conversion"
```

## Seguridad

Las vulnerabilidades se reportan de forma privada según `SECURITY.md`. No se
discuten detalles explotables en issues públicos.

## Modificaciones

Este documento puede cambiar con el tiempo. La versión vigente es la de la rama
`main`.
