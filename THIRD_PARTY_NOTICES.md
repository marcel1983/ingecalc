# Third-party notices

IngeCalc is licensed under GPL-3.0-or-later. It depends on third-party software
and assets with their own licenses. This file is a summary; the complete texts
are distributed by the upstream packages and must be included in release
artifacts.

## Frontend

| Component | License |
|---|---|
| React / React DOM | MIT |
| Vite | MIT |
| TypeScript | Apache-2.0 |
| Tauri JS API and CLI | MIT or Apache-2.0 |
| MathLive | MIT |
| KaTeX | MIT |
| TipTap / ProseMirror | MIT |
| Zustand | MIT |
| Immer | MIT |
| jsPDF | MIT |
| html2canvas | MIT |
| DOMPurify | MPL-2.0 OR Apache-2.0 |

## Fonts

The MathLive and KaTeX font files included in the application are distributed
under the **SIL Open Font License 1.1** (KaTeX fonts). Keep the upstream
copyright and license notices when redistributing the fonts.

Do not use the SMath Studio font, icons or other proprietary assets.

## Rust crates

The Rust workspace uses crates such as `serde`, `serde_json`, `thiserror`,
`uuid`, `clap`, `tracing`, `zip`, and `base64`. Most are dual-licensed
MIT/Apache-2.0; `zip` is MIT.

Before a release, generate a complete inventory with:

```bash
cargo about generate about.hbs > THIRD_PARTY_LICENSES.html
cargo deny check licenses
```

For the frontend:

```bash
npm exec --yes license-checker --summary
```

## Release checklist

- [ ] Include this file and the upstream license texts.
- [ ] Include the GPL-3.0 text in `LICENSE`.
- [ ] Publish the complete corresponding source for each binary release.
- [ ] Do not redistribute proprietary SMath Studio assets.
- [ ] Review plugin licenses separately.
