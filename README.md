# Camlet

Camlet is a lightweight Electron floating camera app for desktops.

The current scope includes:

- secure Electron main, preload, and renderer architecture
- frameless transparent always-on-top overlay shell
- persisted and clamped window size and position
- renderer-driven drag-to-move with main-process window control
- webcam preview with renderer-owned `getUserMedia`
- camera device enumeration and persisted selected camera device id
- compact in-app settings for camera, language, and appearance
- live language switching with persisted `system`, `en`, `pt-BR`, `es`, `fr`, `de`, `it`, and `ja` modes
- startup notices when saved settings were repaired or could not be persisted safely
- translated startup/bootstrap failure handling
- About/build info with packaged-runtime metadata
- development-only diagnostics window for runtime memory and size inspection
- deterministic locale structure checks against the English source locale
- safer camera fallback handling for missing labels, stale saved devices, busy cameras, and missing media APIs
- focused-window keyboard shortcuts for precise overlay movement and resizing
- Electron Builder packaging for Linux AppImage and prepared Windows NSIS output
- GitHub Actions workflows for CI, packaged artifacts, and tag-driven releases
- deterministic unit and integration coverage for shared logic and bootstrap/security paths

## Requirements

- Node.js 20 or newer
- pnpm 9 or newer

## Setup

```bash
pnpm install
```

## Scripts

- `pnpm dev`: runs the Vite renderer, watches Electron sources, and starts Electron
- `pnpm dev:renderer`: starts only the Vite renderer dev server
- `pnpm dev:electron`: starts Electron against an already running renderer dev server and built Electron files
- `pnpm build`: builds both Electron and renderer output
- `pnpm build:electron`: compiles Electron main, preload, and shared runtime modules
- `pnpm build:renderer`: builds the Vite renderer bundle
- `pnpm typecheck`: runs TypeScript project references for app code and tests
- `pnpm test`: runs the Vitest suite
- `pnpm test:unit`: runs the unit tests folder explicitly
- `pnpm test:integration`: runs deterministic integration smoke tests
- `pnpm verify`: runs typecheck, Biome, all tests, and a production build
- `pnpm package`: builds the app and creates packaged artifacts for the current host
- `pnpm package:linux`: builds a Linux AppImage into `release/`
- `pnpm package:win`: builds a Windows NSIS installer into `release/`
- `pnpm dist`: alias for `pnpm package`
- `pnpm check`: runs `biome check`
- `pnpm format`: runs `biome format --write`
- `pnpm clean`: removes generated build and cache output

## Shortcuts

These shortcuts only apply while the Camlet window itself is focused.

- `Arrow keys`: move the overlay by 1px
- `Shift + Arrow keys`: move the overlay by 24px
- `-` or `Numpad -`: decrease overlay size
- `=` or `Numpad +`: increase overlay size

## Project Structure

```text
assets/icons/               Application icons and packaging assets
src/main/                   Electron window bootstrap, IPC, and persistence glue
src/preload/                Sandboxed preload bridge
src/shared/                 Shared contracts, settings, appearance, release, and window-state helpers
src/renderer/               React renderer entry, overlay UI, diagnostics, media preview, locales, and styles
tests/unit/                 Deterministic shared and renderer helper tests
tests/integration/          Deterministic preload, bootstrap, security, and settings-store smoke tests
.github/workflows/          CI, packaging, and release workflows
scripts/                    Local helper scripts
LOCALIZATION.md             Translation structure, locale workflow, and maintenance guidance
```

## Electron Architecture

Camlet stays split into three layers:

1. `main`
   Creates the frameless transparent window, owns persisted settings, clamps restored bounds, performs actual window movement, and enforces navigation and permission policy for trusted local content only.
2. `preload`
   Exposes a minimal typed bridge with `contextIsolation: true`, `nodeIntegration: false`, and `sandbox: true`.
3. `renderer`
   Runs the React overlay shell, initializes i18n, manages the webcam preview directly with web media APIs, applies appearance changes live, and renders startup/build diagnostics.

Raw media streams stay in the renderer and are never sent over IPC.

## Localization Scope

- English remains the source-of-truth locale shape
- all shipped locales are bundled locally with the renderer
- locale structure is validated in tests against the English base locale
- system locale detection maps supported language families to the shipped locale set
- the settings language picker uses stable ordering with native language labels

## Manual Validation

Use [`QA.md`](./QA.md) for the release-candidate checklist covering:

- Ubuntu Wayland validation
- Ubuntu X11 validation
- Windows packaged-app smoke checks
- transparency, drag, persistence, and camera behavior
- startup/config recovery scenarios
- development diagnostics window for runtime memory and size inspection
- language switching and translated startup/settings/About coverage

Use [`LOCALIZATION.md`](./LOCALIZATION.md) for the translation structure, source-of-truth rules, and the safe process for adding a new locale.

Use [`RELEASING.md`](./RELEASING.md) for the candidate-to-release flow.

## Packaging and Distribution

Camlet uses Electron Builder with:

- Linux AppImage as the primary release artifact
- Windows NSIS installer preparation
- deterministic artifact naming under `release/`
- a dedicated [`electron-builder.yml`](./electron-builder.yml) config

Local packaging examples:

```bash
pnpm package:linux
pnpm package:win
```
