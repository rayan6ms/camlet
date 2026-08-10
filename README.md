<!-- markdownlint-disable MD033 MD041 -->
<h1 align="center">Camlet</h1>

<p align="center">
  <img src="./assets/icons/256x256.png" alt="Camlet icon" width="128" height="128">
</p>

<p align="center">
  <a href="https://github.com/rayan6ms/camlet/releases">
    <img src="https://img.shields.io/github/downloads/rayan6ms/camlet/total?style=flat-square" alt="Total downloads">
  </a>
  <a href="https://github.com/rayan6ms/camlet/stargazers">
    <img src="https://img.shields.io/github/stars/rayan6ms/camlet?style=flat-square" alt="GitHub stars">
  </a>
  <a href="https://github.com/rayan6ms/camlet/releases/latest">
    <img src="https://img.shields.io/github/v/release/rayan6ms/camlet?display_name=tag&sort=semver&style=flat-square" alt="Latest release">
  </a>
</p>

<p align="center">
  Camlet is a lightweight floating webcam app for desktop. It stays on top, keeps a clean transparent shell, and lets you quickly place your camera overlay where you want it by dragging it with the left mouse button.
</p>
<p align="center">
  Use the right mouse button anywhere inside it to show more options.
</p>
<!-- markdownlint-enable MD033 MD041 -->

## Choose the right Camlet

This `master` branch contains the **legacy Electron implementation**. The
recommended, actively developed Camlet is the **native Rust implementation**.

| Version | Technology | Download or source |
| --- | --- | --- |
| **Camlet 0.2.x native (recommended)** | Rust, Iced, and WGPU; no browser or webview | Download a release asset whose name starts with `camlet-rust`, or visit the [`rewrite/iced` branch](https://github.com/rayan6ms/camlet/tree/rewrite/iced) |
| Camlet 0.1.x legacy | Electron and TypeScript | Download a [0.1.x release](https://github.com/rayan6ms/camlet/releases?q=v0.1), or use this branch |

The icon shown above belongs to the legacy Electron version. The native Rust
README and packages use the separate `camlet-rust` icon and artifact names so
users can tell exactly which implementation they are installing.

## Legacy Electron features

- always-on-top floating camera window
- frameless transparent overlay
- saved window position and size
- camera selection and persistence
- live language switching: `en` and `pt-BR`
- keyboard shortcuts for moving and resizing the overlay

## Shortcuts

These only work while the Camlet window is focused.

- `Arrow keys`: move the overlay by 1px
- `Shift + Arrow keys`: move the overlay by 24px
- `-` or `Numpad -`: decrease overlay size
- `=` or `Numpad +`: increase overlay size

## Running the legacy version from source

Requirements:

- Node.js 20+
- pnpm 9+

Install dependencies:

```bash
pnpm install
```

Start the app in development:

```bash
pnpm dev
```

## Build

Create a production build:

```bash
pnpm build
```

Package executables:

```bash
pnpm package:linux
pnpm package:win
```

Build artifacts are written to `release/`.

## Releases

The current release is the native Rust version and provides explicitly named
`camlet-rust` packages. This legacy workflow builds:

- Linux AppImage
- Windows NSIS installer

## License

GPL-3.0-only
