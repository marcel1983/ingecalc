# IngeCalc — recursos de marca

**IngeCalc** es el nombre público del proyecto. El formato de documento mantiene
compatibilidad con la etapa anterior mediante el identificador legado
`mathdoc-json`; los documentos nuevos usan `ingecalc-json`.

## Concepto

IngeCalc no se limita a una especialidad. La marca usa un monograma **IC** rodeado
por cuatro nodos que representan las áreas principales:

| Nodo | Color | Referencia |
|---|---|---|
| Civil | Ámbar `#FFB24A` | Puente / estructuras |
| Eléctrica | Amarillo `#FACC15` | Rayo / circuitos |
| Mecánica | Cian `#47D7FF` | Engranaje / máquinas |
| Hidráulica | Azul `#2F7DFF` | Gota / fluidos |

La órbita punteada sugiere que todas las disciplinas comparten el mismo entorno
de cálculo. El banner desarrolla la idea con cuatro esquemas técnicos: un puente,
un circuito RLC, un par de engranajes y un circuito hidráulico con bomba y válvula.

## Archivos

| Archivo | Uso |
|---|---|
| `ingecalc-avatar.png` | Avatar cuadrado para GitHub (1024 × 1024), con fondo completo. |
| `ingecalc-logo.png` | Logo cuadrado con esquinas redondeadas y fondo transparente (1024 × 1024). |
| `ingecalc-banner.png` | Banner social para GitHub (1280 × 640). |
| `ingecalc-banner@2x.png` | Versión 2× para pantallas de alta densidad (2560 × 1280). |
| `*.svg` | Fuentes vectoriales editables. |

## Paleta

| Color | Hex | Uso |
|---|---|---|
| Azul noche | `#07111F` | Fondo principal. |
| Azul acero | `#0B1A2E` | Superficies y tarjetas. |
| Cian | `#47D7FF` | Acento de marca y mecánica. |
| Azul eléctrico | `#2F7DFF` | Trazos, enlaces e hidráulica. |
| Ámbar | `#FFB24A` | Civil y resultados. |
| Amarillo | `#FACC15` | Eléctrica. |
| Blanco | `#F8FBFF` | Texto principal. |
| Gris azulado | `#B7C6DA` | Texto secundario. |

## Uso en GitHub

1. Subir `ingecalc-avatar.png` como avatar del repositorio u organización.
2. Subir `ingecalc-banner.png` en **Settings → General → Social preview**.
3. Para el README:

```html
<p align="center">
  <img src="docs/assets/branding/ingecalc-banner.png" alt="IngeCalc — Cálculo para todas las ingenierías" width="100%">
</p>
```

Los iconos de la aplicación Tauri se regeneran desde el logo:

```bash
cd apps/desktop
npx tauri icon ../../docs/assets/branding/ingecalc-logo.png
```
