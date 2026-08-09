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
  A lightweight native floating camera overlay built with Rust, Iced, and WGPU.
</p>
<p align="center">
  Drag with the left mouse button. Open settings with the right mouse button.
</p>
<!-- markdownlint-enable MD033 MD041 -->

## Features

- native Rust application with no browser or webview
- frameless, transparent, always-on-top camera window
- seamless anti-aliased camera edges and optional gradient ring
- original camera aspect ratio by default
- circle, rounded square, portrait, landscape, and diamond shapes
- cover and contain preview fitting
- camera selection, selectable 15/24/30/60 FPS capture, recovery states, and a visible retry placeholder
- saved position, size, appearance, camera, and language
- English and Brazilian Portuguese interfaces
- deterministic automation and visual regression coverage

## Download

The GitHub release provides:

- Linux x86_64 AppImage
- Debian/Ubuntu x86_64 deb package
- Windows x86_64 installer

Release builds are currently unsigned. macOS packages are not yet provided.

## Shortcuts

These shortcuts work while the Camlet window is focused:

- `Arrow keys`: move by one logical pixel
- `Shift + Arrow keys`: move by 24 logical pixels
- `-` or `Numpad -`: reduce the overlay size
- `=` or `Numpad +`: increase the overlay size
- `Escape`: close the active panel

## Build from source

Camlet requires Rust 1.88 or newer. Linux camera builds also need Clang/libclang,
Video4Linux headers, X11/Wayland development libraries, and a Vulkan or OpenGL
driver.

```bash
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo build --workspace --release --locked
target/release/camlet
```

Build installable packages with `cargo-packager` 0.11.8:

```bash
cargo install cargo-packager --version 0.11.8 --locked
cargo packager --config Packager.toml --formats appimage,deb
# Windows:
cargo packager --config Packager.toml --formats nsis
```

## Automated preview

The complete overlay path can be exercised without a camera or manual input:

```bash
target/release/camlet \
  --frame-source synthetic \
  --profile-dir /tmp/camlet-profile \
  --automation-script fixtures/automation/full-smoke.json \
  --automation-output /tmp/camlet-smoke
```

The scenario captures the original view and every custom shape from the real
WGPU compositor. Each capture is checked against the deterministic reference
renderer, including edge alpha and the four cardinal axes.

## Troubleshooting

### Camera error

Camlet displays camera failures inside a dark, shaped placeholder with a retry
button. Grant camera permission, close other applications using the camera, and
select **Retry camera**.

### Graphics initialization failure

Update the system graphics driver. Camlet requires a WGPU-compatible Vulkan,
Direct3D 12, or OpenGL implementation.

### Linux position or always-on-top restrictions

Native Wayland compositors restrict absolute positioning and z-order. Camlet
automatically uses X11/Xwayland when `DISPLAY` is available so dragging and the
arrow-key shortcuts remain functional. Set `WINIT_UNIX_BACKEND=wayland` to opt
into native Wayland and accept those compositor restrictions.

## Privacy

Camera frames remain in process memory and are neither logged nor saved. A
screenshot is written only when an explicit command-line automation capture is
requested. Diagnostics omit camera identifiers, frame contents, and profile
paths.

## License

GPL-3.0-only
