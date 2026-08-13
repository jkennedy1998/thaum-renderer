# renderer-assets

Canonical boot-owned proving asset root for early thaum-renderer development.

Current staged contents:

- `glyph-fonts/thaum-mono/`
  - `ThaumMono-W80.ttf`
  - `ThaumMono-W160.ttf`
  - `ThaumMono-W320.ttf`
  - `ThaumMono-W640.ttf`
- `cell-sprites/monothaum-atlas-v3/`
  - canonical glyph sprite-sheet batches used first for `CellGraphic::Glyph`
  - includes `sections.txt` plus `monothaum_*.png` batch sheets
- `cell-sprites/proofs/`
  - `grass.png`
  - `channel-bands.png`

Intent:

- keep canonical development assets under `orchestration/` because boot/dev proving ownership lives there
- let consumer apps choose this asset root during local development without forcing them to own the staged files
- preserve the stable renderer-facing folder expectations while app-side asset-root choice stays external
